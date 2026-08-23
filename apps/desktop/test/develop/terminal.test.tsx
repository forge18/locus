import { render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import { DevelopView } from '../../src/screens/develop/DevelopView'

describe('Develop terminal', () => {
  it('renders a linked-repo terminal session explanation', () => {
    const { getByTestId } = render(() => <DevelopView />)
    expect(getByTestId('develop-terminal').textContent).toContain('linked repo')
  })
})
