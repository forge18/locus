import { render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import { AgentsView } from '../../src/screens/automate/AgentsView'

describe('Automate agents', () => {
  it('renders run status and transcript', () => {
    const { getByTestId } = render(() => <AgentsView />)
    expect(getByTestId('transcript-pane').textContent).not.toBe('')
  })
})
