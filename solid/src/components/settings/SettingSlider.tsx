import { Component } from 'solid-js';
import { Slider } from '@kobalte/core/slider';

interface SettingSliderProps {
  id: string;
  title: string;
  description: string;
  value: number;
  min: number;
  max: number;
  step: number;
  valueSuffix?: string;
  onChange: (val: number) => void;
}

export const SettingSlider: Component<SettingSliderProps> = (props) => {
  return (
    <div class="setting-item-row vertical" id={props.id}>
      <div class="setting-header-pane">
        <div>
          <div class="setting-title">{props.title}</div>
          <div class="setting-desc">{props.description}</div>
        </div>
        <div class="setting-value-badge">
          {props.value}{props.valueSuffix || ''}
        </div>
      </div>

      <Slider
        value={[props.value]}
        onChange={(vals) => props.onChange(vals[0])}
        minValue={props.min}
        maxValue={props.max}
        step={props.step}
        class="kobalte-slider-root"
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
