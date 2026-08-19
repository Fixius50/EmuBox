import { Component } from 'solid-js';
import type { BadgeProps } from '@contracts/common.types';

export const Badge: Component<BadgeProps> = (props) => {
  const getVariantClass = (): string => {
    switch (props.variant) {
      case 'highlight':
        return 'readonly-badge highlight';
      case 'boost':
        return 'readonly-badge boost';
      case 'chip':
        return 'blade-platform-chip';
      case 'success':
        return 'readonly-badge highlight';
      case 'warning':
        return 'readonly-badge boost';
      case 'danger':
        return 'readonly-badge';
      case 'interactive':
        return 'readonly-badge interactive-badge';
      case 'default':
      default:
        return 'readonly-badge';
    }
  };

  return (
    <span
      class={`${getVariantClass()} ${props.class ? props.class : ''}`}
      style={props.style}
      onClick={props.onClick}
    >
      {props.children}
    </span>
  );
};
