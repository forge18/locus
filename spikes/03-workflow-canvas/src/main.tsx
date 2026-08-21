// WebKit has no requestIdleCallback, and solid-flow calls it unguarded in
// requestUpdateNodeInternals. Without this the canvas renders NOTHING on
// WKWebView (macOS) and WebKitGTK (Linux) — two of Tauri's three webviews —
// with one ReferenceError and no other symptom. Measured: see
// scripts/webkit-check.mjs and screenshots/webkit.png.
//
// Three lines, and it must load before solid-flow does.
if (typeof globalThis.requestIdleCallback !== 'function') {
  globalThis.requestIdleCallback = ((cb: (d: IdleDeadline) => void) =>
    setTimeout(() => cb({ didTimeout: false, timeRemaining: () => 1 } as IdleDeadline), 1)) as never;
  globalThis.cancelIdleCallback = ((id: number) => clearTimeout(id)) as never;
}

import { render } from 'solid-js/web';
import '@dschz/solid-flow/styles';
import './tokens.css';
import { Canvas } from './Canvas';
import { fixture } from './fixture';

render(() => <Canvas graph={fixture()} />, document.getElementById('root')!);
