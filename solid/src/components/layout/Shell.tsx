import { Component, JSX } from 'solid-js';

interface ShellProps {
  children: JSX.Element;
  ambientBackdropUrl?: string;
  crtShaderEnabled?: boolean;
}

export const Shell: Component<ShellProps> = (props) => {
  return (
    <div class="emubox-1080p-root console-app-root">
      <div class="emubox-1080p-stage console-app-stage">
        <div class="stage-content-wrapper">{props.children}</div>
      </div>
    </div>
  );
};
