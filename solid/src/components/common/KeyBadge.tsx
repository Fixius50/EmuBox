import { Component, Show } from 'solid-js';
import type { KeyBadgeProps } from '@contracts/common.types';

export const KeyBadge: Component<KeyBadgeProps> = (props) => {
  const gamepadVariant = () =>
    props.variant && props.variant !== 'default' ? props.variant : '';

  return (
    <>
      <Show when={props.gamepad}>
        <span
          class={`xmb-key-badge gamepad ${gamepadVariant()} ${props.class || ''}`.trim()}
        >
          {props.gamepad}
        </span>
      </Show>
      <Show when={props.kbd}>
        <span class={`xmb-key-badge kbd ${props.class || ''}`.trim()}>
          {props.kbd}
        </span>
      </Show>
    </>
  );
};

