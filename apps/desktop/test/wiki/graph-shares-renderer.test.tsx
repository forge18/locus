import { describe, expect, it } from 'vitest'
import { existsSync } from 'node:fs'
import { resolve } from 'node:path'
import { GraphRenderer } from '../../src/workflow-canvas/GraphRenderer'
import { SRC, read } from '../css'

describe('wiki/graph-shares-renderer', () => {
  it('imports the canvas renderer rather than drawing its own SVG', () => {
    const source = read('screens/wiki/WikiGraph.tsx')
    expect(source).toContain("from '../../workflow-canvas/GraphRenderer'")
    expect(source).toContain('<GraphRenderer')
  })

  it('contains no <svg> of its own', () => {
    const source = read('screens/wiki/WikiGraph.tsx')
    expect(source).not.toMatch(/<svg|<circle|<line\b|<path/)
  })

  it('is the same component object, not a copy that looks like it', () => {
    const renderer = GraphRenderer
    expect(typeof renderer).toBe('function')
    expect(existsSync(resolve(SRC, 'workflow-canvas/GraphRenderer.tsx'))).toBe(true)
  })

  it('leaves the renderer ignorant of what a page is', () => {
    const renderer = read('workflow-canvas/GraphRenderer.tsx')
    expect(renderer).not.toMatch(/\bwikilink\b|\bWikiPage\b|\bPageKind\b/)
    expect(renderer).not.toMatch(/from '.*\/data\//)
    expect(renderer).not.toMatch(/from '.*\/fixtures\//)
  })

  it('is the only graph SVG in the app, so there is nothing to diverge from', () => {
    const svgOwners = ['workflow-canvas/GraphRenderer.tsx']
    for (const file of ['screens/wiki/WikiGraph.tsx', 'screens/wiki/WikiSidebar.tsx']) {
      expect(read(file), file).not.toMatch(/<svg/)
    }
    expect(read(svgOwners[0])).toMatch(/<svg/)
  })
})
