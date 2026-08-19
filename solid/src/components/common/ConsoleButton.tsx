import { Component, Show } from 'solid-js';
import type { ConsoleButtonProps } from '@contracts/common.types';

export const ConsoleButton: Component<ConsoleButtonProps> = (props) => {
  const getButtonClass = (): string => {
    const focusedClass = props.isFocused ? 'focused' : '';
    switch (props.variant) {
      case 'danger':
        return `crud-btn-delete ${focusedClass}`;
      case 'secondary':
        return `crud-btn-cancel ${focusedClass}`;
      case 'primary':
        return `crud-btn-save ${focusedClass}`;
      case 'action':
      default:
        return `settings-action-btn ${focusedClass}`;
    }
  };

  return (
    <button
      ref={props.ref}
      class={`${getButtonClass()} ${props.class ? props.class : ''}`}
      style={props.style}
      onClick={props.onClick}
      disabled={props.disabled}
    >
      <Show when={props.buttonKey}>
        <span style={{ "margin-right": "0.375rem", opacity: "0.9" }}>[{props.buttonKey}]</span>
      </Show>
      <span>{props.label}</span>
    </button>
  );
};
