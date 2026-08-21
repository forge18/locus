import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { For } from 'solid-js'
import { SkeletonRows } from '../src/ui/SkeletonRows'

const ROW_HEIGHT = 26
const COLUMNS = ['1.4fr', '60px', '80px']

/** Stand-in for a real table row, so the two can be measured against each other. */
function DataRows(props: { count: number }) {
  return (
    <div data-testid="data-rows">
      <For each={Array.from({ length: props.count })}>
        {() => (
          <div
            class="data-row"
            style={{
              height: `${ROW_HEIGHT}px`,
              display: 'grid',
              'grid-template-columns': COLUMNS.join(' '),
            }}
          />
        )}
      </For>
    </div>
  )
}

describe('skeleton-no-reflow', () => {
  it('draws a row per count', () => {
    const { getByTestId } = render(() => <SkeletonRows count={5} rowHeight={ROW_HEIGHT} />)
    expect(getByTestId('skeleton-rows').querySelectorAll('.skeleton-row').length).toBe(5)
  })

  it('matches the real row height exactly, so the table does not jump', () => {
    const skeleton = render(() => (
      <SkeletonRows count={3} rowHeight={ROW_HEIGHT} columns={COLUMNS} />
    ))
    const data = render(() => <DataRows count={3} />)

    const skeletonHeights = [...skeleton.getByTestId('skeleton-rows').querySelectorAll('.skeleton-row')]
      .map((el) => (el as HTMLElement).style.height)
    const dataHeights = [...data.getByTestId('data-rows').querySelectorAll('.data-row')]
      .map((el) => (el as HTMLElement).style.height)

    expect(skeletonHeights).toEqual(dataHeights)
  })

  it('lays the bars out on the same columns the data will use', () => {
    const { getByTestId } = render(() => (
      <SkeletonRows count={1} rowHeight={ROW_HEIGHT} columns={COLUMNS} />
    ))
    const row = getByTestId('skeleton-rows').querySelector('.skeleton-row') as HTMLElement
    expect(row.style.gridTemplateColumns).toBe(COLUMNS.join(' '))
    expect(row.querySelectorAll('.skeleton-bar').length).toBe(COLUMNS.length)
  })

  it('is hidden from readers — a placeholder is not content', () => {
    const { getByTestId } = render(() => <SkeletonRows count={2} rowHeight={ROW_HEIGHT} />)
    expect(getByTestId('skeleton-rows').getAttribute('aria-hidden')).toBe('true')
  })
})
