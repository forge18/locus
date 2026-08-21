// PLAN.md §Risks names WebKitGTK the weakest of the three webviews. WebKit is
// also the engine behind WKWebView, which is what Tauri uses on macOS — so a
// WebKit failure is not a Linux-only problem.
//
// solid-flow calls requestIdleCallback unguarded in requestUpdateNodeInternals,
// and that API has historically been absent from WebKit. This runs the real
// canvas in playwright's WebKit build and reports what happens.
//
// It is not WebKitGTK and does not claim to be: playwright's webkit is a WebKit
// build with its own embedding layer. What it answers is the engine question,
// which is the one that matters for an unguarded platform API.
import { createServer } from 'node:http';
import { readFile } from 'node:fs/promises';
import { extname, join } from 'node:path';
import { createRequire } from 'node:module';
import { execSync } from 'node:child_process';

const require = createRequire(join(execSync('npm root -g').toString().trim(), 'x.js'));
const { webkit, chromium } = require('playwright');
const TYPES = { '.html': 'text/html', '.js': 'text/javascript', '.css': 'text/css' };
const root = new URL('../dist/', import.meta.url).pathname;

const server = createServer(async (req, res) => {
  const p = req.url === '/' ? '/index.html' : req.url.split('?')[0];
  try {
    res.writeHead(200, { 'content-type': TYPES[extname(p)] ?? 'application/octet-stream' });
    res.end(await readFile(join(root, p)));
  } catch { res.writeHead(404); res.end(); }
});
await new Promise((r) => server.listen(4175, r));

const report = {};
for (const [name, engine] of [['webkit', webkit], ['chromium', chromium]]) {
  const browser = await engine.launch();
  const page = await browser.newPage({ viewportSize: { width: 1440, height: 900 } });
  const errors = [];
  page.on('pageerror', (e) => errors.push(String(e)));
  page.on('console', (m) => { if (m.type() === 'error') errors.push(m.text()); });
  await page.goto('http://127.0.0.1:4175/', { waitUntil: 'networkidle' });
  let rendered = { nodes: 0, edgePaths: 0, loopGroups: 0 };
  try {
    await page.waitForSelector('.wf-node', { timeout: 8000 });
    rendered = await page.evaluate(() => ({
      nodes: document.querySelectorAll('.wf-node').length,
      edgePaths: document.querySelectorAll('.solid-flow__edges path').length,
      loopGroups: document.querySelectorAll('.wf-loop-group').length,
    }));
  } catch { /* recorded as zero, with the errors below */ }
  const apis = await page.evaluate(() => ({
    requestIdleCallback: typeof window.requestIdleCallback,
    ResizeObserver: typeof window.ResizeObserver,
    DOMMatrix: typeof window.DOMMatrix,
  }));
  if (name === 'webkit') {
    await page.screenshot({ path: new URL('../screenshots/webkit.png', import.meta.url).pathname });
  }
  report[name] = { version: browser.version(), apis, rendered, pageErrors: errors };
  await browser.close();
}
server.close();
console.log(JSON.stringify(report, null, 2));
