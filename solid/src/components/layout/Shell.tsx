import { Component, JSX, Show } from 'solid-js';

interface ShellProps {
  children: JSX.Element;
  ambientBackdropUrl?: string;
  crtShaderEnabled?: boolean;
}

export const Shell: Component<ShellProps> = (props) => {
  return (
    <div class="emubox-1080p-root">
      <div class="emubox-1080p-stage">
        {/* Dynamic Ambient Background Canvas */}
        <div class="ambient-backdrop-layer">
          <Show when={props.ambientBackdropUrl}>
            <div
              class="ambient-backdrop-image"
              style={{
                "background-image": `url(${props.ambientBackdropUrl})`
              }}
            />
          </Show>
          <div class="ambient-vignette-overlay" />
          <div class="ambient-grid-mesh" />
        </div>

        {/* Console Stage Content */}
        <div class="stage-content-wrapper">
          {props.children}
        </div>
      </div>
    </div>
  );
};
