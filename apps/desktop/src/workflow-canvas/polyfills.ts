// WebKit does not expose requestIdleCallback, but @dschz/solid-flow uses it
// while recalculating node internals. Install the smallest browser-compatible
// fallback before a canvas is mounted in a Tauri webview.
const scope = globalThis as typeof globalThis & {
  requestIdleCallback?: (callback: IdleRequestCallback) => number;
  cancelIdleCallback?: (handle: number) => void;
};

if (typeof scope.requestIdleCallback !== "function") {
  scope.requestIdleCallback = (callback) =>
    setTimeout(
      () => callback({ didTimeout: false, timeRemaining: () => 1 }),
      1,
    ) as unknown as number;
  scope.cancelIdleCallback = (handle) => clearTimeout(handle);
}

if (typeof window !== "undefined" && typeof window.matchMedia !== "function") {
  window.matchMedia = ((query: string) => ({
    matches: false,
    media: query,
    onchange: null,
    addListener: () => undefined,
    removeListener: () => undefined,
    addEventListener: () => undefined,
    removeEventListener: () => undefined,
    dispatchEvent: () => false,
  })) as typeof window.matchMedia;
}

if (typeof globalThis.ResizeObserver !== "function") {
  globalThis.ResizeObserver = class {
    observe() {}
    unobserve() {}
    disconnect() {}
  } as typeof ResizeObserver;
}

if (
  typeof window !== "undefined" &&
  typeof window.DOMMatrixReadOnly !== "function"
) {
  window.DOMMatrixReadOnly = class {
    m22: number;

    constructor(transform: string) {
      const scale = transform.match(/scale\(([-+]?\d*\.?\d+)/);
      this.m22 = scale ? Number(scale[1]) : 1;
    }
  } as typeof DOMMatrixReadOnly;
}
