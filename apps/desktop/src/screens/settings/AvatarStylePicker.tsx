import { Select } from "@kobalte/core/select";
import { createMemo } from "solid-js";
import {
  AVATAR_STYLES,
  avatarStyleOption,
  type AvatarStyleOption,
} from "../../avatars/derive";
import {
  setAvatarStylePreference,
  useAvatarStyle,
} from "../../avatars/preferences";
import { DEFAULT_BOT_AVATAR_STYLE } from "../../avatars/setting";
import "./avatar-style-picker.css";

export function AvatarStylePicker() {
  const currentStyle = useAvatarStyle();
  const activeStyle = createMemo(() => avatarStyleOption(currentStyle()));

  const choose = (option: AvatarStyleOption | null) => {
    if (option) setAvatarStylePreference(option.id);
  };

  return (
    <section class="settings-section" data-testid="bot-avatar-style-setting">
      <h3>Bot avatars</h3>
      <div class="settings-row">
        <div class="settings-copy">
          <span>Avatar style</span>
          <p>
            One install-wide style for every bot. The default is Bottts; avatars
            are derived from bot ids and are never stored.
          </p>
          <p class="avatar-style-attribution" data-testid="avatar-attribution">
            {activeStyle().label} by {activeStyle().creator} ·{" "}
            <a href={activeStyle().licenseUrl} rel="noreferrer" target="_blank">
              {activeStyle().license}
            </a>
          </p>
        </div>
        <div class="settings-control">
          <Select<AvatarStyleOption>
            options={[...AVATAR_STYLES]}
            value={activeStyle()}
            onChange={choose}
            optionValue="id"
            optionTextValue="label"
            class="avatar-style-select"
            placeholder={DEFAULT_BOT_AVATAR_STYLE}
            itemComponent={(itemProps) => {
              const option = itemProps.item.rawValue;
              return (
                <Select.Item
                  item={itemProps.item}
                  class="avatar-style-option"
                  data-testid={`avatar-style-option-${option.id}`}
                >
                  <div class="avatar-style-option-copy">
                    <Select.ItemLabel>{option.label}</Select.ItemLabel>
                    <span class="avatar-style-option-meta">
                      {option.creator} · {option.license}
                    </span>
                  </div>
                  <Select.ItemIndicator
                    class="avatar-style-option-indicator"
                    aria-hidden="true"
                  >
                    ✓
                  </Select.ItemIndicator>
                </Select.Item>
              );
            }}
          >
            <Select.HiddenSelect />
            <Select.Label class="avatar-style-label">
              Bot avatar style
            </Select.Label>
            <Select.Trigger
              class="settings-select avatar-style-trigger"
              data-testid="avatar-style-trigger"
            >
              <Select.Value<AvatarStyleOption>>
                {(state) => state.selectedOption()?.label ?? "Bottts"}
              </Select.Value>
              <Select.Icon aria-hidden="true">⌄</Select.Icon>
            </Select.Trigger>
            <Select.Portal>
              <Select.Content class="avatar-style-content">
                <Select.Listbox class="avatar-style-list" />
              </Select.Content>
            </Select.Portal>
          </Select>
        </div>
      </div>
    </section>
  );
}
