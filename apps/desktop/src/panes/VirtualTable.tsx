import { For, Show } from 'solid-js'
import { VirtualRows } from './VirtualRows'
import type { Column } from '../ui/Table'

/**
 * A `Table` that only renders the rows you can see, and asks for the next page
 * as you approach the end of what is loaded.
 *
 * It composes the chrome `Column` definitions rather than replacing them, so a
 * screen swapping a `Table` for a `VirtualTable` keeps its column types, its
 * mono numerics and its alignment for free.
 */
export interface VirtualTableProps<T> {
  columns: Column<T>[]
  /** The rows loaded so far. */
  rows: T[]
  rowKey: (row: T) => string
  /** How many rows exist. Larger than `rows.length` while pages are still coming. */
  total: number
  rowHeight: number
  /** Height of the scrolling body. */
  height: number
  onLoadMore?: () => void
  onRowClick?: (row: T) => void
  testId?: string
}

export function VirtualTable<T>(props: VirtualTableProps<T>) {
  const cls = (c: Column<T>) => `col-${c.type ?? 'text'}`
  const loading = () => props.rows.length < props.total

  return (
    <div class="table-wrap virtual-table" data-testid={props.testId ?? 'table'}>
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
      </table>

      <VirtualRows
        items={props.rows}
        total={props.total}
        rowHeight={props.rowHeight}
        height={props.height}
        onLoadMore={props.onLoadMore}
        testId={`${props.testId ?? 'table'}-rows`}
      >
        {(row) => (
          <table class="table" style={{ 'table-layout': 'fixed' }}>
            <tbody>
              <tr
                style={{ height: `${props.rowHeight}px` }}
                data-row-key={props.rowKey(row)}
                onClick={props.onRowClick ? () => props.onRowClick!(row) : undefined}
              >
                <For each={props.columns}>{(c) => <td class={cls(c)}>{c.cell(row)}</td>}</For>
              </tr>
            </tbody>
          </table>
        )}
      </VirtualRows>

      <Show when={loading()}>
        <div class="virtual-loading" data-testid={`${props.testId ?? 'table'}-loading`}>
          {props.rows.length.toLocaleString('en-US')} of{' '}
          {props.total.toLocaleString('en-US')} loaded — scroll for more
        </div>
      </Show>
    </div>
  )
}
