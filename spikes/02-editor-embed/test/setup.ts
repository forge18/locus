// CodeMirror measures. jsdom does not lay out, so anything that depends on real
// geometry (tooltip placement, scroll position) is out of scope here and is
// answered by the screenshots instead.
if (typeof Range !== 'undefined' && !Range.prototype.getBoundingClientRect) {
  Range.prototype.getBoundingClientRect = () => ({ x: 0, y: 0, top: 0, left: 0, right: 0,
    bottom: 0, width: 0, height: 0, toJSON: () => ({}) }) as DOMRect;
  Range.prototype.getClientRects = () => Object.assign([], { item: () => null }) as never;
}
