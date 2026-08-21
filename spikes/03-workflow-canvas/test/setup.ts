import '@testing-library/jest-dom/vitest';

// jsdom gaps solid-flow needs filled. Each is a real browser API the library
// reaches for; none is a workaround for a bug in it, and all three exist in
// every webview Tauri ships (WKWebView, WebView2, WebKitGTK).
//
//  * ResizeObserver — nodes are measured, not assumed
//  * matchMedia     — @solid-primitives/media, for the dark/light and pointer queries
//  * DOMMatrixReadOnly — the pan/zoom transform in @xyflow/system
class RO { observe() {} unobserve() {} disconnect() {} }
(globalThis as any).ResizeObserver ??= RO;

(globalThis as any).DOMMatrixReadOnly ??= class {
  m22 = 1; a = 1; b = 0; c = 0; d = 1; e = 0; f = 0;
  constructor(_?: string) {}
};
(globalThis as any).DOMMatrix ??= (globalThis as any).DOMMatrixReadOnly;

if (typeof window !== 'undefined' && !window.matchMedia) {
  window.matchMedia = ((query: string) => ({
    matches: false, media: query, onchange: null,
    addListener() {}, removeListener() {},
    addEventListener() {}, removeEventListener() {},
    dispatchEvent: () => false,
  })) as unknown as typeof window.matchMedia;
}

// jsdom reports every element as 0x0. solid-flow hides a node until it has been
// measured, so without this the canvas renders empty and every assertion below
// would fail for a reason that has nothing to do with the library.
if (typeof Element !== 'undefined') {
  const NODE_W = 188, NODE_H = 86;
  Element.prototype.getBoundingClientRect = function () {
    const isNode = (this as HTMLElement).classList?.contains('solid-flow__node');
    const w = isNode ? NODE_W : 1200;
    const h = isNode ? NODE_H : 800;
    return { x: 0, y: 0, top: 0, left: 0, right: w, bottom: h, width: w, height: h,
             toJSON: () => ({}) } as DOMRect;
  };
}

// requestIdleCallback. jsdom has none — and neither, historically, does WebKit,
// which is the engine behind BOTH of Tauri's non-Windows webviews (WKWebView on
// macOS, WebKitGTK on Linux). solid-flow calls it unguarded in
// requestUpdateNodeInternals, so this is not only a test-environment gap: it is
// a portability question Spike 2 is in a position to answer against a real
// webview. Recorded in FINDINGS.md as such.
(globalThis as any).requestIdleCallback ??= (cb: (d: unknown) => void) =>
  setTimeout(() => cb({ didTimeout: false, timeRemaining: () => 1 }), 0);
(globalThis as any).cancelIdleCallback ??= (id: number) => clearTimeout(id);
