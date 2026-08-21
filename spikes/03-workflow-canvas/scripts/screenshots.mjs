// Task 4's evidence: the canvas, rendered in a real browser, at the handoff's
// density. A screenshot is not a test — it is the only way to answer "does this
// reach the fidelity screen 12 draws" without a person squinting at a test run.
//
// Uses the globally installed playwright rather than adding it to this spike's
// dependencies: a 100MB browser download does not belong in a package whose
// whole job is to answer four questions.
import { createServer } from 'node:http';
import { readFile } from 'node:fs/promises';
import { extname, join } from 'node:path';
import { createRequire } from 'node:module';
import { execSync } from 'node:child_process';

const globalRoot = execSync('npm root -g').toString().trim();
const require = createRequire(join(globalRoot, 'x.js'));
const { chromium } = require('playwright');

const TYPES = { '.html': 'text/html', '.js': 'text/javascript', '.css': 'text/css' };
const root = new URL('../dist/', import.meta.url).pathname;

const server = createServer(async (req, res) => {
  const path = req.url === '/' ? '/index.html' : req.url.split('?')[0];
  try {
    const body = await readFile(join(root, path));
    res.writeHead(200, { 'content-type': TYPES[extname(path)] ?? 'application/octet-stream' });
    res.end(body);
  } catch {
    res.writeHead(404); res.end('not found');
  }
});
await new Promise((r) => server.listen(4173, r));

const browser = await chromium.launch();
const page = await browser.newPage({ viewportSize: { width: 1440, height: 900 },
                                     deviceScaleFactor: 2 });
const errors = [];
page.on('pageerror', (e) => errors.push(String(e)));
page.on('console', (m) => { if (m.type() === 'error') errors.push(m.text()); });

await page.goto('http://127.0.0.1:4173/', { waitUntil: 'networkidle' });
await page.waitForSelector('.wf-node', { timeout: 10_000 });

// What the screenshot is worth depends on what actually rendered, so count it.
const counts = await page.evaluate(() => ({
  nodes: document.querySelectorAll('.wf-node').length,
  kinds: [...new Set([...document.querySelectorAll('.wf-node')]
           .map((n) => n.getAttribute('data-kind')))].sort(),
  edgePaths: document.querySelectorAll('.solid-flow__edge path, .solid-flow__edges path').length,
  markers: document.querySelectorAll('marker').length,
  loopGroups: document.querySelectorAll('.wf-loop-group').length,
  hasRequestIdleCallback: typeof window.requestIdleCallback === 'function',
}));

await page.screenshot({ path: new URL('../screenshots/canvas.png', import.meta.url).pathname });
await browser.close();
server.close();

console.log(JSON.stringify({ ...counts, pageErrors: errors }, null, 2));
if (errors.length) process.exit(1);
if (counts.nodes === 0) { console.error('nothing rendered'); process.exit(1); }
