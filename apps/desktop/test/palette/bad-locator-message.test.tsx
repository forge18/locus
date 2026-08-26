import { describe, expect, it } from 'vitest'
import { resolve } from '../../src/nav/locator'

describe('palette/bad-locator-message', () => {
  it('names the invalid locator segment', () => {
    expect(() => resolve('locus://tapestry/nope/item')).toThrow(/kind/)
  })
})
