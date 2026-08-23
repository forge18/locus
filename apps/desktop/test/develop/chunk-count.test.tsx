import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { DevelopView } from '../../src/screens/develop/DevelopView'
import { useHunks } from '../../src/data/develop'
import { read, rules } from '../css'

const mount = () => render(() => <DevelopView />)

describe('develop/chunk-count', () => {
  it('offers the collapseUnchanged toggle', () => {
    const { getByTestId } = mount()
    expect(getByTestId('collapse-unchanged').textContent).toBe('collapseUnchanged')
    expect(getByTestId('collapse-unchanged').getAttribute('aria-pressed')).toBe('true')
  })

  it('toggles it', () => {
    const { getByTestId } = mount()
    getByTestId('collapse-unchanged').click()
    expect(getByTestId('collapse-unchanged').getAttribute('aria-pressed')).toBe('false')
  })

  it('counts the chunks from the diff, not from the label', () => {
    const { getByTestId } = mount()
    expect(getByTestId('dev-chunks').textContent).toBe(`${useHunks().length} chunks`)
    expect(useHunks().length).toBe(2)
  })

  it('sets the count in accent', () => {
    const { getByTestId } = mount()
    expect(getByTestId('dev-chunks').className).toContain('dev-chunks')
    expect(
      rules(read('screens/screens.css')).find((r) => r.selector === '.dev-chunks')!.body,
    ).toContain('color: var(--action-attention)')
  })

  it('sits on the right of the tab strip', () => {
    expect(
      rules(read('screens/screens.css')).find((r) => r.selector === '.dev-tabs-right')!.body,
    ).toContain('margin-left: auto')
  })
})
