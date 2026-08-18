import { Component } from 'solid-js';
import { Switch } from '@kobalte/core/switch';

interface SettingSwitchProps {
  id: string;
  title: string;
  description: string;
  checked: boolean;
  onChange: (checked: boolean) => void;
}

export const SettingSwitch: Component<SettingSwitchProps> = (props) => {
  return (
    <div class="setting-item-row" id={props.id}>
      <div class="setting-text-pane">
        <div class="setting-title">{props.title}</div>
        <div class="setting-desc">{props.description}</div>
      </div>

      <Switch checked={props.checked} onChange={props.onChange}>
        <Switch.Input />
        <Switch.Control class={`switch-control ${props.checked ? 'checked' : ''}`}>
          <Switch.Thumb class={`switch-thumb ${props.checked ? 'checked' : ''}`} />
        </Switch.Control>
      </Switch>
    </div>
  );
};
