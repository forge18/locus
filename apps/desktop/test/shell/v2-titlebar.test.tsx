import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { AppTitleBar } from '../../src/shell/AppTitleBar'
import { read, rules } from '../css'

const rule = (selector: string) => rules(read('shell/shell.css')).find((candidate) => candidate.selector === selector)

describe('shell/v2-titlebar', () => {
  it('renders the fixed-height desktop title-bar foundation', () => {
    const { getByTestId } = render(() => <AppTitleBar />)

    expect(getByTestId('app-titlebar').className).toContain('titlebar')
    expect(getByTestId('traffic-lights').querySelectorAll('span')).toHaveLength(3)
    expect(getByTestId('wordmark').textContent).toBe('Locus')
    expect(rule('.titlebar')?.body).toContain('height: 42px')
  })
})
