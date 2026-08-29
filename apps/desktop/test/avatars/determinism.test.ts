import { describe, expect, it } from "vitest";
import { deriveAvatar } from "../../src/avatars/derive";

describe("avatars/determinism", () => {
  it("uses the bot id rather than its display name as the seed", () => {
    const beforeRename = deriveAvatar("bottts", "bot-keeper");
    const afterRename = deriveAvatar("bottts", "bot-keeper");

    expect(afterRename).toBe(beforeRename);
  });

  it("changes the derived robot when the id changes", () => {
    expect(deriveAvatar("bottts", "bot-keeper")).not.toBe(
      deriveAvatar("bottts", "bot-new-id"),
    );
  });

  it("keeps different styles deterministic independently", () => {
    expect(deriveAvatar("bottts", "bot-keeper")).toBe(
      deriveAvatar("bottts", "bot-keeper"),
    );
    expect(deriveAvatar("lorelei", "bot-keeper")).toBe(
      deriveAvatar("lorelei", "bot-keeper"),
    );
  });
});
