import type { JSX } from 'solid-js';

export type BadgeVariant =
  | 'default'
  | 'highlight'
  | 'boost'
  | 'chip'
  | 'success'
  | 'warning'
  | 'danger'
  | 'interactive';

export interface BadgeProps {
  variant?: BadgeVariant;
  class?: string;
  style?: JSX.CSSProperties | string;
  children: JSX.Element;
  onClick?: (e: MouseEvent) => void;
}

export type ButtonVariant = 'primary' | 'secondary' | 'danger' | 'action';

export interface ConsoleButtonProps {
  label: string;
  buttonKey?: string;
  variant?: ButtonVariant;
  isFocused?: boolean;
  disabled?: boolean;
  style?: JSX.CSSProperties | string;
  class?: string;
  onClick?: (e: MouseEvent) => void;
  ref?: HTMLButtonElement | ((el: HTMLButtonElement) => void);
}

export interface SettingCardRowProps {
  title: string;
  description: string;
  isFocused?: boolean;
  tag?: string;
  actionBadge?: string;
  children?: JSX.Element;
  onClick?: () => void;
  style?: JSX.CSSProperties | string;
  class?: string;
}

export interface SettingSwitchProps {
  title: string;
  description: string;
  checked: boolean;
  isFocused?: boolean;
  onChange: (checked: boolean) => void;
}

export interface SettingSliderProps {
  title: string;
  description: string;
  value: number;
  min: number;
  max: number;
  unit?: string;
  isFocused?: boolean;
  onChange: (val: number) => void;
}
