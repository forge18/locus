import { fireEvent, render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import ManageView from '../../src/screens/manage/ManageView'

describe('automate/no-agent-list', () => {
  it('keeps List rows task-centric rather than exposing an agent queue', async () => {
    const { getByText, getByTestId, queryByTestId } = render(() => <ManageView />)
    await fireEvent.click(getByText('List'))
    expect(getByTestId('automate-list-tasks')).toBeTruthy()
    expect(queryByTestId('manage-session-detail')).toBeNull()
  })
})
