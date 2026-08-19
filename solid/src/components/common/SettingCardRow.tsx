import { Component, Show } from 'solid-js';
import type { SettingCardRowProps } from '@contracts/common.types';
import { Badge } from './Badge';

export const SettingCardRow: Component<SettingCardRowProps> = (props) => {
  return (
    <div
      class={`setting-card-row ${props.isFocused ? 'focused' : ''} ${props.class ? props.class : ''}`}
      style={props.style}
      onClick={props.onClick}
    >
      <div class="setting-info">
        <div style={{ display: 'flex', 'align-items': 'center', gap: '0.5rem' }}>
          <Show when={props.tag}>
            <Badge variant="default" style={{ 'font-size': '0.5625rem', padding: '0.125rem 0.375rem' }}>
              {props.tag}
            </Badge>
          </Show>
          <span class="setting-title">{props.title}</span>
        </div>
        <span class="setting-desc">{props.description}</span>
      </div>

      <Show when={props.children} fallback={
        <Show when={props.actionBadge}>
          <Badge variant="interactive" class={props.isFocused ? 'highlight' : ''}>
            {props.actionBadge}
          </Badge>
        </Show>
      }>
        {props.children}
      </Show>
    </div>
  );
};
