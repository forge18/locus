import { For, Show } from "solid-js";
import { VirtualRows } from "./VirtualRows";
import { EmptyPane } from "../ui/EmptyPane";
import { InlineError } from "../ui/InlineError";
import { SkeletonRows } from "../ui/SkeletonRows";
import type { Column } from "../ui/Table";

export type VirtualTableState = "loading" | "empty" | "error" | "loaded";

/**
 * A `Table` that only renders the rows you can see, and asks for the next page
 * as you approach the end of what is loaded.
 *
 * It composes the chrome `Column` definitions rather than replacing them, so a
 * screen swapping a `Table` for a `VirtualTable` keeps its column types, its
 * mono numerics and its alignment for free.
 */
export interface VirtualTableProps<T> {
    columns: Column<T>[];
    /** The rows loaded so far. */
    rows: T[];
    rowKey: (row: T) => string;
    /** How many rows exist. Larger than `rows.length` while pages are still coming. */
    total: number;
    rowHeight: number;
    /** Height of the scrolling body. */
    height: number;
    /** Explicitly marks the first page as pending; pagination remains inferred from `total`. */
    loading?: boolean;
    /** A backend failure. Errors take precedence over loading and empty states. */
    error?: string | Error;
    /** Why the table is empty. It is shown only when the loaded total is zero. */
    emptyMessage?: string;
    /** Optional recovery action for a failed load. */
    onRetry?: () => void;
    onLoadMore?: () => void;
    onRowClick?: (row: T) => void;
    testId?: string;
}

export function VirtualTable<T>(props: VirtualTableProps<T>) {
    const testId = () => props.testId ?? "table";
    const cls = (c: Column<T>) => `col-${c.type ?? "text"}`;
    // Keep the old total-based pagination behavior while allowing an empty first
    // page to be explicitly loading. A count of zero is otherwise indistinguishable
    // from a request that has not started yet. A partial page remains loading even
    // when a caller forgets to set the flag, so it can never look fully loaded.
    const loading = () =>
        props.loading === true || props.rows.length < props.total;
    const hasError = () => props.error !== undefined;
    const empty = () =>
        !hasError() &&
        !loading() &&
        props.rows.length === 0 &&
        props.total === 0;
    const state = (): VirtualTableState => {
        if (hasError()) return "error";
        if (loading()) return "loading";
        if (empty()) return "empty";
        return "loaded";
    };
    const initialLoading = () =>
        state() === "loading" && props.rows.length === 0;
    const errorMessage = () =>
        props.error instanceof Error ? props.error.message : props.error;

    return (
        <div
            class="table-wrap virtual-table"
            data-testid={testId()}
            data-state={state()}
            data-table-state={state()}
            role="table"
            /* One header row plus every row in the store, not only the loaded window. */
            aria-rowcount={props.total + 1}
        >
            <table class="table">
                <thead role="rowgroup">
                    <tr role="row">
                        <For each={props.columns}>
                            {(c) => (
                                <th
                                    role="columnheader"
                                    class={cls(c)}
                                    style={
                                        c.width ? { width: c.width } : undefined
                                    }
                                >
                                    {c.header}
                                </th>
                            )}
                        </For>
                    </tr>
                </thead>
            </table>

            <Show when={hasError()}>
                <div
                    data-testid={`${testId()}-error`}
                    style={{ padding: "var(--g-3) var(--g-4)" }}
                >
                    <InlineError
                        cause={
                            errorMessage() || "The table could not be loaded."
                        }
                        next={
                            props.onRetry
                                ? "Retry loading this table."
                                : "Refresh to try again."
                        }
                        action={
                            props.onRetry ? (
                                <button
                                    type="button"
                                    onClick={() => props.onRetry?.()}
                                >
                                    Retry
                                </button>
                            ) : undefined
                        }
                    />
                </div>
            </Show>

            <Show when={initialLoading()}>
                <div
                    class="virtual-loading"
                    data-testid={`${testId()}-loading`}
                    role="status"
                    aria-label="Loading table"
                >
                    Loading…
                </div>
                <SkeletonRows
                    count={6}
                    rowHeight={props.rowHeight}
                    columns={props.columns.map((c) => c.width ?? "1fr")}
                />
            </Show>

            <Show when={empty()}>
                <div
                    data-testid={`${testId()}-empty`}
                    style={{ "min-height": `${props.height}px` }}
                >
                    <EmptyPane
                        reason={props.emptyMessage ?? "No rows to display."}
                    />
                </div>
            </Show>

            <Show when={props.rows.length > 0}>
                <VirtualRows
                    items={props.rows}
                    total={props.total}
                    rowHeight={props.rowHeight}
                    height={props.height}
                    onLoadMore={hasError() ? undefined : props.onLoadMore}
                    testId={`${testId()}-rows`}
                >
                    {(row, index) => (
                        <table
                            class="table"
                            style={{ "table-layout": "fixed" }}
                        >
                            <tbody role="rowgroup">
                                <tr
                                    style={{ height: `${props.rowHeight}px` }}
                                    data-row-key={props.rowKey(row)}
                                    role="row"
                                    /* Global position in the sparse table: header is row 1. */
                                    aria-rowindex={index + 2}
                                    tabIndex={props.onRowClick ? 0 : undefined}
                                    onClick={
                                        props.onRowClick
                                            ? () => props.onRowClick!(row)
                                            : undefined
                                    }
                                    onKeyDown={
                                        props.onRowClick
                                            ? (e: KeyboardEvent) => {
                                                  if (
                                                      e.key !== "Enter" &&
                                                      e.key !== " "
                                                  )
                                                      return;
                                                  e.preventDefault();
                                                  props.onRowClick!(row);
                                              }
                                            : undefined
                                    }
                                >
                                    <For each={props.columns}>
                                        {(c) => (
                                            <td role="cell" class={cls(c)}>
                                                {c.cell(row)}
                                            </td>
                                        )}
                                    </For>
                                </tr>
                            </tbody>
                        </table>
                    )}
                </VirtualRows>
            </Show>

            <Show
                when={
                    !hasError() &&
                    loading() &&
                    props.rows.length > 0 &&
                    props.rows.length < props.total
                }
            >
                <div
                    class="virtual-loading"
                    data-testid={`${testId()}-loading`}
                    role="status"
                >
                    {props.rows.length.toLocaleString("en-US")} of{" "}
                    {props.total.toLocaleString("en-US")} loaded — scroll for
                    more
                </div>
            </Show>
        </div>
    );
}
