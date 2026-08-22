export const terminalOptions = { macOptionIsMeta: true, convertEol: true } as const
/** Browser menus must not consume app chords; ordinary terminal keys including vim and IME pass through. */
export const reachesTerminal = (event: KeyboardEvent) => !(event.metaKey && event.type === 'keydown')
