import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { AgentsView } from '../../src/screens/automate/AgentsView'
import { useSessionDetails } from '../../src/data/sessions'

const sessions = useSessionDetails()

describe('agents/minimize-to-strip', () => {
  it('reports the session id to the caller, which owns the strip', () => {
    const minimized: string[] = []
    const { getByTestId } = render(() => <AgentsView onMinimize={(id) => minimized.push(id)} />)
    getByTestId('transcript-minimize').click()
    expect(minimized).toEqual([sessions[0].id])
  })

  it('leaves the session in the list — minimizing is not ending', () => {
    const { getByTestId } = render(() => <AgentsView />)
    getByTestId('transcript-minimize').click()
    expect(getByTestId(`session-card-${sessions[0].id}`)).toBeTruthy()
    expect(getByTestId(`session-chip-${sessions[0].id}`).textContent).toBe(sessions[0].status)
  })

  it('leaves the transcript up — the session is still the selected one', () => {
    const { getByTestId } = render(() => <AgentsView />)
    getByTestId('transcript-minimize').click()
    expect(getByTestId('transcript')).toBeTruthy()
    expect(getByTestId('run-id').textContent).toBe(sessions[0].runId)
  })

  it('minimizes whichever session is selected', () => {
    const minimized: string[] = []
    const { getByTestId } = render(() => <AgentsView onMinimize={(id) => minimized.push(id)} />)
    getByTestId(`session-card-${sessions[2].id}`).click()
    getByTestId('transcript-minimize').click()
    expect(minimized).toEqual([sessions[2].id])
  })

  it('records what has been minimized so the strip can hold it', () => {
    const { getByTestId } = render(() => <AgentsView />)
    getByTestId('transcript-minimize').click()
    expect(getByTestId('minimized-ids').textContent).toBe(sessions[0].id)
  })
})
