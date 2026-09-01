#!/usr/bin/env node
import { execFileSync, spawn } from "node:child_process";
import { existsSync, readFileSync, readdirSync } from "node:fs";
import { join, resolve } from "node:path";
import { setTimeout as sleep } from "node:timers/promises";
import { remote } from "webdriverio";

const repo = resolve(import.meta.dirname, "../../..");
const desktop = join(repo, "apps/desktop");
const migrations = join(repo, "migrations");
const postgresImage = process.env.LOCUS_DESKTOP_TEST_POSTGRES_IMAGE ?? "pgvector/pgvector:pg17";
const database = "locus";
const password = `locus-desktop-integration-${process.pid}`;
const container = `locus-desktop-integration-${process.pid}`;

function unsupported(reason) {
  console.error(`DESKTOP_INTEGRATION_UNSUPPORTED: ${reason}`);
  process.exit(0);
}

function commandExists(command) {
  try {
    execFileSync("sh", ["-c", `command -v ${command}`], { stdio: "ignore" });
    return true;
  } catch {
    return false;
  }
}

for (const command of ["docker", "cargo", "pnpm"]) {
  if (!commandExists(command)) unsupported(`${command} is not installed`);
}

function run(command, args, options = {}) {
  const output = execFileSync(command, args, {
    encoding: "utf8",
    stdio: "pipe",
    ...options,
  });
  return typeof output === "string" ? output.trim() : "";
}

function runInherit(command, args) {
  execFileSync(command, args, { cwd: repo, stdio: "inherit" });
}

function docker(args, options = {}) {
  return run("docker", args, options);
}

function removeContainer() {
  try {
    docker(["rm", "--force", container], { stdio: "ignore" });
  } catch {
    // The container may not have been created.
  }
}

function migrationSql() {
  return readdirSync(migrations)
    .filter((file) => file.endsWith(".up.sql"))
    .sort()
    .map((file) => `-- ${file}\n${readFileSync(join(migrations, file), "utf8")}`)
    .join("\n");
}

function psql(sql) {
  execFileSync(
    "docker",
    [
      "exec",
      "-i",
      container,
      "env",
      `PGPASSWORD=${password}`,
      "psql",
      "--quiet",
      "--host",
      "127.0.0.1",
      "--username",
      "locus",
      "--dbname",
      database,
      "--set",
      "ON_ERROR_STOP=1",
    ],
    { input: sql, cwd: repo, stdio: ["pipe", "inherit", "inherit"] },
  );
}

async function waitForDatabase() {
  for (let attempt = 0; attempt < 120; attempt += 1) {
    try {
      docker(
        [
          "exec",
          container,
          "pg_isready",
          "--host",
          "127.0.0.1",
          "--username",
          "locus",
          "--dbname",
          database,
        ],
        { stdio: "ignore" },
      );
      return;
    } catch {
      await sleep(500);
    }
  }
  throw new Error("the disposable Postgres store did not become ready");
}

function startDatabase() {
  removeContainer();
  docker([
    "run",
    "--detach",
    "--name",
    container,
    "--publish",
    "127.0.0.1::5432",
    "--env",
    "POSTGRES_USER=locus",
    "--env",
    `POSTGRES_PASSWORD=${password}`,
    "--env",
    `POSTGRES_DB=${database}`,
    postgresImage,
  ]);
  const published = docker(["port", container, "5432/tcp"]);
  const port = Number(published.slice(published.lastIndexOf(":") + 1));
  if (!Number.isInteger(port) || port < 1) throw new Error(`invalid Postgres port: ${published}`);
  return `postgres://locus:${password}@127.0.0.1:${port}/${database}?sslmode=disable`;
}

function seedDatabase() {
  psql(migrationSql());
  psql(`
INSERT INTO core.projects (id, name) VALUES
  ('00000000-0000-0000-0000-000000000101', 'tapestry'),
  ('00000000-0000-0000-0000-000000000102', 'loom-db');
INSERT INTO core.repos (id, project_id, name, working_copy_path) VALUES
  ('00000000-0000-0000-0000-000000000201', '00000000-0000-0000-0000-000000000101', 'locus', '/tmp/locus-tapestry'),
  ('00000000-0000-0000-0000-000000000202', '00000000-0000-0000-0000-000000000102', 'loom', '/tmp/locus-loom-db');
`);
}

async function waitForElement(browser, selector, timeout = 30_000) {
  return browser.waitUntil(
    async () => (await browser.$(selector)).isExisting(),
    { timeout, interval: 250, timeoutMsg: `element did not appear: ${selector}` },
  ).then(() => browser.$(selector));
}

async function waitForText(browser, selector, expected, timeout = 30_000) {
  await browser.waitUntil(
    async () => {
      const element = await browser.$(selector);
      return (await element.getText()).includes(expected);
    },
    { timeout, interval: 250, timeoutMsg: `text did not appear in ${selector}: ${expected}` },
  );
}

async function clickSelector(browser, selector) {
  const result = await browser.execute((target) => {
    const element = document.querySelector(target);
    if (!(element instanceof HTMLElement)) return false;
    element.click();
    return true;
  }, selector);
  if (!result) throw new Error(`element not found: ${selector}`);
}

async function clickExact(browser, selector, expected) {
  const result = await browser.execute((target, text) => {
    const element = [...document.querySelectorAll(target)].find(
      (candidate) => candidate.textContent?.trim() === text,
    );
    if (!(element instanceof HTMLElement)) return false;
    element.click();
    return true;
  }, selector, expected);
  if (!result) throw new Error(`button not found: ${expected}`);
}

async function installStreamProbe(browser) {
  await browser.executeAsync((done) => {
    const internals = window.__TAURI_INTERNALS__;
    window.__locusIntegrationEvents = [];
    const callbackId = internals.transformCallback((rawMessage) => {
      if (rawMessage && typeof rawMessage === "object" && "message" in rawMessage) {
        window.__locusIntegrationEvents.push(rawMessage.message);
      }
    });
    window.__locusIntegrationChannelId = callbackId;
    internals
      .invoke("telemetry_subscribe", { channel: `__CHANNEL__:${callbackId}` })
      .then(() => done(null))
      .catch((error) => done(String(error)));
  });
}

async function emitIntegrationEvent(browser, runId, text) {
  await browser.executeAsync((id, message, done) => {
    window.__TAURI_INTERNALS__
      .invoke("desktop_integration_emit_event", { runId: id, text: message })
      .then(() => done(null))
      .catch((error) => done(String(error)));
  }, runId, text);
}

async function streamMessages(browser) {
  const result = await browser.execute(() => window.__locusIntegrationEvents ?? []);
  return result ?? [];
}

function cargoMetadata() {
  try {
    return JSON.parse(run("cargo", ["metadata", "--no-deps", "--format-version", "1"], { cwd: repo }));
  } catch (error) {
    throw new Error("cargo metadata did not return valid JSON", { cause: error });
  }
}

function startHost(binary, databaseUrl, webdriverPort) {
  return spawn(binary, [], {
    cwd: repo,
    env: {
      ...process.env,
      DATABASE_URL: databaseUrl,
      TAURI_WEBDRIVER_PORT: String(webdriverPort),
    },
    stdio: ["ignore", "ignore", "ignore"],
  });
}

async function buildApplication() {
  runInherit("pnpm", ["-C", desktop, "exec", "tauri", "build", "--debug", "--no-bundle", "--features", "webdriver"]);
  const metadata = cargoMetadata();
  const candidates = [
    join(metadata.target_directory, "debug", process.platform === "win32" ? "locus-tauri.exe" : "locus-tauri"),
    join(desktop, "src-tauri", "target", "debug", process.platform === "win32" ? "locus-tauri.exe" : "locus-tauri"),
  ];
  const binary = candidates.find(existsSync);
  if (!binary) throw new Error(`Tauri binary not found; checked ${candidates.join(", ")}`);
  return binary;
}

async function waitForWebdriver(port) {
  for (let attempt = 0; attempt < 120; attempt += 1) {
    try {
      const response = await fetch(`http://127.0.0.1:${port}/status`);
      if (response.ok) return;
    } catch {
      // The app has not started its embedded WebDriver server yet.
    }
    await sleep(250);
  }
  throw new Error(`embedded WebDriver did not start on port ${port}`);
}

let app;
let browser;
let databaseUrl;

async function stopHost() {
  if (!app || app.exitCode !== null || app.signalCode !== null) {
    app = undefined;
    return;
  }
  const exited = new Promise((resolve) => app.once("exit", resolve));
  app.kill("SIGTERM");
  await Promise.race([exited, sleep(5_000)]);
  app = undefined;
}

try {
  databaseUrl = startDatabase();
  await waitForDatabase();
  seedDatabase();
  const binary = await buildApplication();
  const webdriverPort = 4445 + (process.pid % 1000);
  app = startHost(binary, databaseUrl, webdriverPort);

  await waitForWebdriver(webdriverPort);
  browser = await remote({
    hostname: "127.0.0.1",
    port: webdriverPort,
    logLevel: "error",
    capabilities: { browserName: "tauri" },
  });
  await waitForElement(browser, '[data-testid="project-rail"]');
  await clickExact(browser, '[data-testid="project-rail"] button', "Projects");
  await waitForElement(browser, '[data-testid="projects-view"]');
  await waitForText(browser, '[data-testid="project-state-list"]', "#tapestry");
  await clickExact(browser, '[data-testid="project-state-list"] .project-list-item', "#tapestry");
  await waitForText(browser, '[data-testid="project-repos"]', "locus");
  await waitForText(browser, '[data-testid="project-harnesses"]', "Harnesses");

  await clickExact(browser, '[data-testid="project-state-list"] .project-list-item', "#loom-db");
  await waitForText(browser, '[data-testid="project-repos"]', "loom");

  await clickSelector(browser, '[data-testid="dispatch-pill"]');
  await clickExact(browser, '[data-testid="dispatch-popover"] button', "Stop all");
  await waitForText(browser, "body", "Dispatch stopped");

  await installStreamProbe(browser);
  const streamRunId = "00000000-0000-0000-0000-000000000301";
  const streamText = "real Tauri stream update";
  await emitIntegrationEvent(browser, streamRunId, streamText);
  await browser.waitUntil(
    async () => (await streamMessages(browser)).some((event) => event.text === streamText),
    { timeout: 30_000, interval: 250, timeoutMsg: "telemetry stream update was not observed" },
  );

  await clickExact(browser, '[data-testid="project-state-list"] .project-list-item', "#loom-db");
  await waitForText(browser, '[data-testid="project-repos"]', "loom");

  await browser.deleteSession();
  browser = undefined;
  await stopHost();
  docker(["stop", "--time", "1", container], { stdio: "ignore" });
  await sleep(1_000);
  app = startHost(binary, databaseUrl, webdriverPort);
  await waitForWebdriver(webdriverPort);
  browser = await remote({
    hostname: "127.0.0.1",
    port: webdriverPort,
    logLevel: "error",
    capabilities: { browserName: "tauri" },
  });
  await waitForElement(browser, '[data-testid="project-rail"]');
  await clickExact(browser, '[data-testid="project-rail"] button', "Projects");
  const error = await waitForElement(
    browser,
    '[data-testid="project-state-list"] [role="alert"], [data-testid="store-health"][data-status="unavailable"]',
  );
  if (!(await error.getText()).trim()) throw new Error("backend error state was empty");
  process.stdout.write("desktop integration passed: live setup, project scope, stop all, stream, and backend error\n");
} catch (error) {
  throw error;
} finally {
  if (browser) await browser.deleteSession().catch(() => undefined);
  await stopHost();
  removeContainer();
}
