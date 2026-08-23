import { render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import { DevelopView } from '../../src/screens/develop/DevelopView'

describe('Develop file tree', () => {
  it('identifies changed-file state in the project tree', () => {
    const { getByTestId } = render(() => <DevelopView />)
    expect(getByTestId('dev-tree').getAttribute('data-changed-file-state')).toBe('visible')
  })
})
