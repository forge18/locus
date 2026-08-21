// Tab sets per category. Automate is Kanban-then-Agents, in that order.
//
// `agents` is deliberately absent from Workshop: agent definitions are a
// drill-down of Extensions, and Automate's "Agents" tab is a different thing
// entirely — the live session list.
import { categoryOf } from './views'
import type { Category, View } from './views'

export interface CategoryTab {
  view: View
  label: string
}

/**
 * Tabs per category, in the order they are drawn. Plan, Develop and Wiki have
 * none — a single-view category shows no tab bar entries at all.
 *
 * `agents` is deliberately absent: agent definitions are a drill-down of
 * Extensions, and Automate's "Agents" tab is a different thing entirely.
 */
const CATEGORY_TABS: Record<Category, CategoryTab[]> = {
  dashboard: [
    { view: 'inbox', label: 'Inbox' },
    { view: 'status', label: 'Status' },
  ],
  plan: [],
  develop: [],
  automate: [
    { view: 'board', label: 'Kanban' },
    { view: 'sessions', label: 'Agents' },
  ],
  review: [
    { view: 'telemetry', label: 'Telemetry' },
    { view: 'runs', label: 'Runs' },
    { view: 'artifact', label: 'Artifacts' },
  ],
  workshop: [
    { view: 'extensions', label: 'Extensions' },
    { view: 'canvas', label: 'Workflow' },
    { view: 'harnesses', label: 'Harnesses' },
  ],
  wiki: [],
}

export function tabsFor(category: Category): CategoryTab[] {
  return CATEGORY_TABS[category]
}

/**
 * Which tab reads as active for a view. Agent definitions keep **Extensions** lit,
 * because that is where you came from and where the back link goes.
 */
export function activeTabFor(view: View): View | null {
  if (view === 'agents') return 'extensions'
  const tabs = tabsFor(categoryOf(view))
  return tabs.some((t) => t.view === view) ? view : null
}

/**
 * The view a drill-down came from, or null. Agent definitions are entered from
 * the `agents` card on Extensions and go back there — which is also why the
 * Extensions tab stays lit while they are open.
 */
export function drilldownParent(view: View): View | null {
  return view === 'agents' ? 'extensions' : null
}
