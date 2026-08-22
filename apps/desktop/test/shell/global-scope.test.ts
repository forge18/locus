import { describe, expect, it } from 'vitest'
import { createRouteScope } from '../../src/nav/route-scope'

describe('shell/global-scope', () => {
  it('makes global scope explicit rather than carrying a project implicitly', () => {
    const routeScope = createRouteScope('tapestry')

    routeScope.showGlobal()
    expect(routeScope.scope()).toEqual({ kind: 'global' })
  })

  it('restores an explicit selected-project scope', () => {
    const routeScope = createRouteScope('tapestry')

    routeScope.showProject('loom-db')
    expect(routeScope.scope()).toEqual({ kind: 'project', project: 'loom-db' })
  })
})
