// The view/category/label table, as one exported constant and the lookups over it.
//
// The category list is closed at seven. A new surface joins one of them rather
// than adding an eighth: a rail that grows is a rail nobody reads.

export const VIEWS = Object.freeze([
  'inbox',
  'status',
  'plan',
  'wiki',
  'develop',
  'board',
  'sessions',
  'telemetry',
  'runs',
  'artifact',
  'extensions',
  'agents',
  'canvas',
  'harnesses',
] as const)

export type View = (typeof VIEWS)[number]

/**
 * Seven, and closed. Frozen at runtime as well as in the type, because "the list
 * is closed" is only a rule if something enforces it: a rail that grows is a rail
 * nobody reads, and the count was six until the Wiki earned its own entry.
 */
export const CATEGORIES = Object.freeze([
  'dashboard',
  'plan',
  'develop',
  'automate',
  'review',
  'workshop',
  'wiki',
] as const)

export type Category = (typeof CATEGORIES)[number]

/** Which category owns each view. A drill-down keeps its parent's category. */
const VIEW_CATEGORY: Record<View, Category> = {
  inbox: 'dashboard',
  status: 'dashboard',
  plan: 'plan',
  develop: 'develop',
  board: 'automate',
  sessions: 'automate',
  telemetry: 'review',
  runs: 'review',
  artifact: 'review',
  extensions: 'workshop',
  agents: 'workshop',
  canvas: 'workshop',
  harnesses: 'workshop',
  wiki: 'wiki',
}

export function categoryOf(view: View): Category {
  return VIEW_CATEGORY[view]
}

export interface RailItem {
  category: Category
  label: string
  /** Phosphor icon name. */
  icon: string
  /** Where a rail click lands — the category's first view, not the last one open. */
  firstView: View
}

/** Seven items, in rail order. Frozen for the same reason CATEGORIES is. */
export const RAIL_ITEMS: readonly RailItem[] = Object.freeze([
  { category: 'dashboard', label: 'Inbox', icon: 'tray', firstView: 'inbox' },
  { category: 'plan', label: 'Plan', icon: 'compass', firstView: 'plan' },
  { category: 'develop', label: 'Develop', icon: 'code', firstView: 'develop' },
  { category: 'automate', label: 'Automate', icon: 'lightning', firstView: 'board' },
  { category: 'review', label: 'Review', icon: 'chart-bar', firstView: 'telemetry' },
  { category: 'workshop', label: 'Workshop', icon: 'wrench', firstView: 'extensions' },
  { category: 'wiki', label: 'Wiki', icon: 'book-bookmark', firstView: 'wiki' },
].map((item) => Object.freeze(item)) as RailItem[])

/** What the rail and the tab bar call each category. */
export const CATEGORY_LABELS: Record<Category, string> = {
  dashboard: 'Inbox',
  plan: 'Plan',
  develop: 'Develop',
  automate: 'Automate',
  review: 'Review',
  workshop: 'Workshop',
  wiki: 'Wiki',
}
