import { Component } from 'solid-js';
import { Slider } from '@kobalte/core/slider';
import type { SettingSliderProps } from '@contracts/common.types';

export const SettingSlider: Component<SettingSliderProps> = (props) => {
  return (
    <div class={`setting-card-row vertical ${props.isFocused ? 'focused' : ''}`}>
      <div class="setting-header-pane" style={{ display: 'flex', 'justify-content': 'space-between', 'align-items': 'center', width: '100%' }}>
        <div>
          <div class="setting-title">{props.title}</div>
          <div class="setting-desc">{props.description}</div>
        </div>
        <div class="setting-value-badge">
          {props.value}{props.unit || ''}
        </div>
      </div>

      <Slider
        value={[props.value]}
        onChange={(vals) => props.onChange(vals[0])}
        minValue={props.min}
        maxValue={props.max}
        step={1}
        class="kobalte-slider-root"
        style={{ width: '100%', 'margin-top': '0.75rem' }}
      >
        <Slider.Track class="slider-track">
          <Slider.Fill class="slider-fill" />
          <Slider.Thumb class="slider-thumb">
            <Slider.Input />
          </Slider.Thumb>
        </Slider.Track>
      </Slider>
    </div>
  );
};
