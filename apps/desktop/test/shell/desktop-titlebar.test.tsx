import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { AppTitleBar } from '../../src/shell/AppTitleBar'
import { read, rules } from '../css'

const rule = (selector: string) => rules(read('shell/shell.css')).find((candidate) => candidate.selector === selector)

describe('shell/desktop-titlebar', () => {
  it('renders the fixed-height desktop title-bar foundation', () => {
    const { getByTestId } = render(() => <AppTitleBar categoryLabel="Plan" viewLabel="Spec" running={8} needsYou={1} />)

    expect(getByTestId('app-titlebar').className).toContain('titlebar')
    expect(getByTestId('traffic-lights').querySelectorAll('span')).toHaveLength(3)
    expect(getByTestId('wordmark').textContent).toBe('Locus')
    expect(getByTestId('running-pill').textContent).toContain('8 running')
    expect(rule('.titlebar')?.body).toContain('height: 42px')
  })
})
