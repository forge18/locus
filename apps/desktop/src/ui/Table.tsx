import { For, Show } from 'solid-js'
import type { JSX } from 'solid-js'
import { SkeletonRows } from './SkeletonRows'

/**
 * A column's type decides how its cells are set, once, here. `numeric` is mono and
 * right-aligned; `mono` is mono and left-aligned (paths, branches, ids); `text` is
 * body type. No screen restates this per cell.
 */
export type ColumnType = 'text' | 'numeric' | 'mono'

export interface Column<T> {
  key: string
  header: string
  type?: ColumnType
  /** Track width for the skeleton state; also applied to the header cell. */
  width?: string
  cell: (row: T) => JSX.Element
}

export interface TableProps<T> {
  columns: Column<T>[]
  rows: T[]
  /** Row identity, so a re-sort does not re-create every row. */
  rowKey: (row: T) => string
  /** True while the rows are still arriving. Draws skeletons at the real height. */
  loading?: boolean
  /** How many skeleton rows to draw, and at what height. */
  skeletonRows?: number
  rowHeight?: number
  onRowClick?: (row: T) => void
}

const DEFAULT_ROW_HEIGHT = 26

export function Table<T>(props: TableProps<T>) {
  const rowHeight = () => props.rowHeight ?? DEFAULT_ROW_HEIGHT
  const cls = (c: Column<T>) => `col-${c.type ?? 'text'}`

  return (
    <div class="table-wrap" data-testid="table">
      <table class="table">
        <thead>
          <tr>
            <For each={props.columns}>
              {(c) => (
                <th class={cls(c)} style={c.width ? { width: c.width } : undefined}>
                  {c.header}
                </th>
              )}
            </For>
          </tr>
        </thead>
        <Show when={!props.loading}>
          <tbody>
            <For each={props.rows}>
              {(row) => (
                <tr
                  style={{ height: `${rowHeight()}px` }}
                  onClick={props.onRowClick ? () => props.onRowClick!(row) : undefined}
                  data-row-key={props.rowKey(row)}
                >
                  <For each={props.columns}>{(c) => <td class={cls(c)}>{c.cell(row)}</td>}</For>
                </tr>
              )}
            </For>
          </tbody>
        </Show>
      </table>
      <Show when={props.loading}>
        <SkeletonRows
          count={props.skeletonRows ?? 6}
          rowHeight={rowHeight()}
          columns={props.columns.map((c) => c.width ?? '1fr')}
        />
      </Show>
    </div>
  )
}
