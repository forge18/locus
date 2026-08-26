import { render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import { STRIP_CARDS } from '../../src/fixtures/strip'
import { Strip } from '../../src/shell/Strip'

describe('shell/running-strip-task-link', () => {
  it('links an active agent card to its owning task', () => {
    const { getByTestId } = render(() => <Strip cards={STRIP_CARDS} />)
    const link = getByTestId('strip-task-t-004')
    expect(link.getAttribute('href')).toBe('locus://tapestry/task/t-004')
    expect(getByTestId('strip-card-st-1').getAttribute('data-task-locator')).toBe('locus://tapestry/task/t-004')
  })
})
