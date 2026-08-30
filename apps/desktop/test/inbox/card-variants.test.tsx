import { describe, expect, it } from "vitest";
import { render } from "@solidjs/testing-library";
import { InboxCard } from "../../src/screens/inbox/InboxCard";
import { PENDING } from './deliveries';
const [gate, ask, guardrail] = PENDING;

describe('inbox/card-variants', () => {
  it('renders each delivery with its own subject', () => {
    for (const item of [gate, ask, guardrail]) {
      const { getByTestId, unmount } = render(() => <InboxCard item={item} selected={false} onSelect={() => {}} />)
      expect(getByTestId(`inbox-card-${item.id}`).textContent).toContain(item.subject)
      unmount()
    }
  })

  it('renders each delivery with its own project', () => {
    const { getByTestId } = render(() => <InboxCard item={guardrail} selected={false} onSelect={() => {}} />)
    expect(getByTestId('inbox-card-sub').textContent).toContain('loom-db')
  })
})
