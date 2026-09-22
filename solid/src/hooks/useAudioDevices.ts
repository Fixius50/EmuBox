import { createSignal, onCleanup, onMount } from "solid-js";
import type { AudioTabProps } from "@contracts/settings.types";
import type { AudioDevice } from "@contracts/system.types";

export function useAudioDevices(props: AudioTabProps) {
  const [devices, setDevices] = createSignal<AudioDevice[]>([]);
  const [loading, setLoading] = createSignal(false);
  const [error, setError] = createSignal("");
  const [limited, setLimited] = createSignal(false);
  let request = 0;
  const media =
    typeof navigator === "undefined" ? undefined : navigator.mediaDevices;
  const refresh = async () => {
    const current = ++request;
    setLoading(true);
    setError("");
    try {
      if (!props.backend) throw new Error("Servicio de audio no disponible.");
      const info = await props.backend.getAudioInfo();
      const available = info.devices;
      if (current !== request) return;
      setDevices(available);
      setLimited(info.deviceNamesLimited ?? false);
      const settings = props.settings;
      if (
        settings &&
        (["inputDevice", "outputDevice"] as const).some((key, index) => {
          const selected = settings.audio[key];
          return (
            selected &&
            selected !== "default" &&
            !available.some(
              (device) =>
                device.id === selected &&
                device.type === (index === 0 ? "source" : "sink"),
            )
          );
        })
      ) {
        props.onUpdateSettings((value) => {
          for (const [key, type] of [
            ["inputDevice", "source"],
            ["outputDevice", "sink"],
          ] as const) {
            const selected = value.audio[key];
            if (
              selected &&
              !available.some(
                (device) => device.id === selected && device.type === type,
              )
            )
              value.audio[key] = "default";
          }
        });
      }
    } catch (cause) {
      if (current === request)
        setError(
          cause instanceof Error
            ? cause.message
            : "No se pudieron detectar los dispositivos de audio.",
        );
    } finally {
      if (current === request) setLoading(false);
    }
  };
  const deviceChanged = () => {
    void refresh();
  };
  onMount(() => {
    void refresh();
    media?.addEventListener("devicechange", deviceChanged);
  });
  onCleanup(() => {
    request++;
    media?.removeEventListener("devicechange", deviceChanged);
  });
  return { devices, loading, error, limited, refresh };
}
