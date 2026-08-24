// The nav store: the only thing that sets `view`.
//
// A component that flips the view itself is a navigation path that will drift
// from the other six. Everything goes through `go` or `open`, and
// scripts/check-single-resolver.sh keeps it that way.

import { createMemo, createSignal } from 'solid-js'
import type { Accessor } from 'solid-js'
import { CATEGORY_LABELS, categoryOf } from './views'
import type { Category, View } from './views'
import { tabsFor } from './tabs'
import type { CategoryTab } from './tabs'
import { format, resolve } from './locator'
import type { NavTarget, ViewParams } from './locator'

export interface NavStore {
  view: Accessor<View>
  params: Accessor<ViewParams>
  category: Accessor<Category>
  categoryLabel: Accessor<string>
  /** The full locator for where you are. */
  locator: Accessor<string>
  /** The same, without the scheme — what the title bar and tab bar show. */
  locatorPath: Accessor<string>
  tabs: Accessor<CategoryTab[]>

  /** Navigate to a view. The one way `view` changes. */
  go: (view: View, params?: Partial<ViewParams>) => void
  /** Navigate by locator — ⌘K, a deep link, an inbox item, a board-card link. */
  open: (locator: string) => NavTarget

  /** Detail opens in place, as a sheet over the current category. */
  detail: Accessor<NavTarget | null>
  openDetail: (locator: string) => void
  closeDetail: () => void

  canBack: Accessor<boolean>
  canForward: Accessor<boolean>
  back: () => void
  forward: () => void
  /** The history stack, as locators. Per window. */
  history: Accessor<string[]>
}

export interface NavStoreOptions {
  view?: View
  project?: string
}

export function createNavStore(options: NavStoreOptions = {}): NavStore {
  const project = options.project ?? 'tapestry'
  const start: NavTarget = { view: options.view ?? 'inbox', params: { project } }

  const [target, setTarget] = createSignal<NavTarget>(start)
  const [stack, setStack] = createSignal<string[]>([format(start.view, start.params)])
  const [cursor, setCursor] = createSignal(0)
  const [detail, setDetail] = createSignal<NavTarget | null>(null)

  const view = createMemo(() => target().view)
  const params = createMemo(() => target().params)
  const locator = createMemo(() => format(target().view, target().params))

  /**
   * Push onto the stack, discarding anything forward of the cursor.
   *
   * The target is set from `resolve(locator)` rather than from what the caller
   * passed, so the store's params are always exactly what the locator encodes.
   * Anything the grammar does not carry cannot survive a back button, and this is
   * where that becomes true instead of merely intended.
   */
  const push = (next: NavTarget) => {
    const at = format(next.view, next.params)
    setTarget(resolve(at))
    if (at === stack()[cursor()]) return
    setStack([...stack().slice(0, cursor() + 1), at])
    setCursor(cursor() + 1)
  }

  const go: NavStore['go'] = (nextView, nextParams) => {
    // A named project scopes a target. `project: undefined` deliberately carries
    // global scope instead of re-adding the last selected project.
    const params = nextParams && 'project' in nextParams
      ? nextParams.project === undefined ? {} : nextParams
      : { project, ...nextParams }
    push({ view: nextView, params })
  }

  const open: NavStore['open'] = (at) => {
    const resolved = resolve(at)
    push(resolved)
    return resolved
  }

  const step = (delta: number) => {
    const to = cursor() + delta
    if (to < 0 || to >= stack().length) return
    setCursor(to)
    const at = stack()[to]
    setTarget(resolve(at))
  }

  return {
    view,
    params,
    category: createMemo(() => categoryOf(view())),
    categoryLabel: createMemo(() => CATEGORY_LABELS[categoryOf(view())]),
    locator,
    locatorPath: createMemo(() => locator().replace('locus://', '')),
    tabs: createMemo(() => tabsFor(categoryOf(view()))),
    go,
    open,
    detail,
    // Detail is a sheet over the current category. It does not touch `view`, which
    // is why the rail does not move when one opens.
    openDetail: (at) => setDetail(resolve(at)),
    closeDetail: () => setDetail(null),
    canBack: createMemo(() => cursor() > 0),
    canForward: createMemo(() => cursor() < stack().length - 1),
    back: () => step(-1),
    forward: () => step(1),
    history: stack,
  }
}
