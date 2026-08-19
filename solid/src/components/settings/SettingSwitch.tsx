import { Component } from 'solid-js';
import { Switch } from '@kobalte/core/switch';
import type { SettingSwitchProps } from '@contracts/common.types';

export const SettingSwitch: Component<SettingSwitchProps> = (props) => {
  return (
    <div
      class={`setting-card-row ${props.isFocused ? 'focused' : ''}`}
      onClick={() => props.onChange(!props.checked)}
    >
      <div class="setting-info">
        <span class="setting-title">{props.title}</span>
        <span class="setting-desc">{props.description}</span>
      </div>

      <Switch checked={props.checked} onChange={props.onChange} class="console-switch">
        <Switch.Input class="switch-input" />
        <Switch.Control class="switch-control">
          <Switch.Thumb class="switch-thumb" />
        </Switch.Control>
      </Switch>
    </div>
  );
};
