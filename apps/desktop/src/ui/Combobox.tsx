import { Combobox as KCombobox } from '@kobalte/core/combobox'
import type { ComboboxRootItemComponentProps } from '@kobalte/core/combobox'

export interface ComboboxOption {
  value: string
  label: string
}

export interface ComboboxProps {
  options: ComboboxOption[]
  value: ComboboxOption | null
  onChange: (value: ComboboxOption | null) => void
  placeholder?: string
  label: string
}

export function Combobox(props: ComboboxProps) {
  return (
    <KCombobox<ComboboxOption>
      options={props.options}
      optionValue="value"
      optionTextValue="label"
      optionLabel="label"
      value={props.value}
      onChange={props.onChange}
      placeholder={props.placeholder}
      itemComponent={(itemProps: ComboboxRootItemComponentProps<ComboboxOption>) => (
        <KCombobox.Item class="menu-item" item={itemProps.item}>
          <KCombobox.ItemLabel>{itemProps.item.rawValue.label}</KCombobox.ItemLabel>
        </KCombobox.Item>
      )}
    >
      <KCombobox.Control class="combobox-control" aria-label={props.label}>
        <KCombobox.Input class="combobox-input" data-testid="combobox-input" />
      </KCombobox.Control>
      <KCombobox.Portal>
        <KCombobox.Content class="menu" data-testid="combobox-content">
          <KCombobox.Listbox />
        </KCombobox.Content>
      </KCombobox.Portal>
    </KCombobox>
  )
}
