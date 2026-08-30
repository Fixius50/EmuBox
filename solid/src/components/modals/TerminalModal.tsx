import { Component, createSignal, onMount, For, Show } from 'solid-js';
import { Dialog } from '@kobalte/core/dialog';
import type { IEmuBoxBackend } from '@contracts/backend.types';

interface TerminalModalProps {
  isOpen: boolean;
  onClose: () => void;
  backend: IEmuBoxBackend;
}

export const TerminalModal: Component<TerminalModalProps> = (props) => {
  const [outputHistory, setOutputHistory] = createSignal<Array<{ text: string; type: 'cmd' | 'out' | 'err' | 'sys' }>>([
    { text: 'EmuBox OS - Consola de Diagnóstico, Red y Mantenimiento Integrada', type: 'sys' },
    { text: 'Presiona [ESC] o el botón Cerrar para volver a la interfaz de juegos.', type: 'sys' }
  ]);
  const [commandInput, setCommandInput] = createSignal<string>('');
  const [isRunning, setIsRunning] = createSignal<boolean>(false);
  const [ipSummary, setIpSummary] = createSignal<string>('Consultando IP...');
  const [sshSummary, setSshSummary] = createSignal<string>('Consultando SSH...');

  let terminalOutputRef: HTMLDivElement | undefined;
  let inputRef: HTMLInputElement | undefined;

  const scrollToBottom = () => {
    setTimeout(() => {
      if (terminalOutputRef) {
        terminalOutputRef.scrollTop = terminalOutputRef.scrollHeight;
      }
    }, 40);
  };

  const runCommand = async (cmd: string) => {
    if (!cmd.trim() || isRunning()) return;

    const trimmed = cmd.trim();
    setOutputHistory((prev) => [...prev, { text: `$ ${trimmed}`, type: 'cmd' }]);
    setIsRunning(true);
    scrollToBottom();

    try {
      const res = await props.backend.executeCommand(trimmed);
      setOutputHistory((prev) => [...prev, { text: res, type: 'out' }]);
    } catch (err: any) {
      setOutputHistory((prev) => [...prev, { text: `[ERROR]: ${err?.message || err}`, type: 'err' }]);
    } finally {
      setIsRunning(false);
      setCommandInput('');
      scrollToBottom();
      setTimeout(() => inputRef?.focus(), 50);
    }
  };

  const refreshNetworkInfo = async () => {
    try {
      const ipRes = await props.backend.executeCommand("ip -br a 2>/dev/null || ip a 2>/dev/null | grep 'inet ' | grep -v '127.0.0.1' || hostname -I");
      setIpSummary(ipRes.trim().replace(/\n/g, '  |  ') || 'Sin IP detectada');
    } catch {
      setIpSummary('Error consultando IP');
    }

    try {
      const sshRes = await props.backend.executeCommand("systemctl is-active sshd 2>/dev/null || echo 'inactivo'");
      setSshSummary(sshRes.trim() === 'active' ? 'SSHD ACTIVO (Puerto 22)' : `SSHD: ${sshRes.trim()}`);
    } catch {
      setSshSummary('SSHD no disponible');
    }
  };

  onMount(() => {
    if (props.isOpen) {
      refreshNetworkInfo();
      setTimeout(() => inputRef?.focus(), 150);
    }
  });

  return (
    <Dialog open={props.isOpen} onOpenChange={(open) => { if (!open) props.onClose(); }}>
      <Dialog.Portal>
        <Dialog.Overlay
          class="console-modal-backdrop"
          style={{
            "background": "rgba(3, 7, 18, 0.94)",
            "backdrop-filter": "blur(1.5rem)",
            "z-index": "10000"
          }}
        />
        <div
          class="console-modal-center-container"
          style={{ "z-index": "10001", width: "94vw", "max-width": "68rem" }}
        >
          <Dialog.Content
            class="crud-modal-box"
            style={{
              width: "100%",
              "background": "#050811",
              "border-color": "#00f0ff",
              "box-shadow": "0 0 4rem rgba(0, 240, 255, 0.35), 0 2rem 6rem rgba(0, 0, 0, 0.95)",
              padding: "1.25rem",
              display: "flex",
              "flex-direction": "column",
              gap: "0.75rem",
              height: "82vh",
              "max-height": "48rem"
            }}
          >
            {/* 1. Header Bar */}
            <div style={{ display: "flex", "align-items": "center", "justify-content": "space-between", "border-bottom": "0.0625rem solid rgba(0, 240, 255, 0.2)", "padding-bottom": "0.75rem" }}>
              <div style={{ display: "flex", "align-items": "center", gap: "0.75rem" }}>
                <span style={{ "font-size": "1.25rem" }}>💻</span>
                <div>
                  <Dialog.Title style={{ color: "#00f0ff", "font-size": "1.125rem", "font-weight": "800", "letter-spacing": "0.0625rem" }}>
                    TERMINAL DE DIAGNÓSTICO, RED Y COMANDOS
                  </Dialog.Title>
                  <Dialog.Description style={{ "font-size": "0.75rem", color: "var(--text-secondary)" }}>
                    Ejecuta comandos de Arch Linux directamente desde la interfaz gráfica de EmuBox
                  </Dialog.Description>
                </div>
              </div>

              <button
                onClick={props.onClose}
                style={{
                  background: "rgba(239, 68, 68, 0.2)",
                  border: "1px solid #ef4444",
                  color: "#ffffff",
                  padding: "0.4rem 0.875rem",
                  "border-radius": "0.375rem",
                  "font-size": "0.75rem",
                  "font-weight": "700",
                  cursor: "pointer"
                }}
              >
                [ESC / B] CERRAR
              </button>
            </div>

            {/* 2. Real-time Telemetry Banner */}
            <div
              style={{
                display: "flex",
                "flex-wrap": "wrap",
                gap: "1rem",
                background: "rgba(15, 23, 42, 0.8)",
                padding: "0.625rem 1rem",
                "border-radius": "0.375rem",
                border: "1px solid rgba(255, 255, 255, 0.08)",
                "font-size": "0.8125rem"
              }}
            >
              <div>
                <span style={{ color: "var(--text-secondary)" }}>🌐 IP de la Máquina: </span>
                <strong style={{ color: "#10b981", "font-family": "var(--font-mono)" }}>{ipSummary()}</strong>
              </div>
              <div>
                <span style={{ color: "var(--text-secondary)" }}>🔒 Servicio SSH: </span>
                <strong style={{ color: "#38bdf8", "font-family": "var(--font-mono)" }}>{sshSummary()}</strong>
              </div>
            </div>

            {/* 3. Quick Action Buttons */}
            <div style={{ display: "flex", "flex-wrap": "wrap", gap: "0.5rem" }}>
              <button
                onClick={() => runCommand("ip addr || hostname -I")}
                style={{ background: "rgba(30, 41, 59, 0.9)", border: "1px solid rgba(0, 240, 255, 0.4)", color: "#00f0ff", padding: "0.35rem 0.75rem", "border-radius": "0.375rem", "font-size": "0.75rem", cursor: "pointer", "font-weight": "600" }}
              >
                🔍 Ver IP Detallada (ip a)
              </button>
              <button
                onClick={() => runCommand("sudo systemctl status sshd --no-pager")}
                style={{ background: "rgba(30, 41, 59, 0.9)", border: "1px solid rgba(0, 240, 255, 0.4)", color: "#00f0ff", padding: "0.35rem 0.75rem", "border-radius": "0.375rem", "font-size": "0.75rem", cursor: "pointer", "font-weight": "600" }}
              >
                🔒 Estado SSH (systemctl)
              </button>
              <button
                onClick={() => runCommand("sudo systemctl restart sshd && sudo systemctl status sshd --no-pager")}
                style={{ background: "rgba(30, 41, 59, 0.9)", border: "1px solid rgba(245, 158, 11, 0.4)", color: "#f59e0b", padding: "0.35rem 0.75rem", "border-radius": "0.375rem", "font-size": "0.75rem", cursor: "pointer", "font-weight": "600" }}
              >
                ⚡ Reiniciar SSH
              </button>
              <button
                onClick={() => runCommand("cat /var/log/emubox/session.log | tail -n 25")}
                style={{ background: "rgba(30, 41, 59, 0.9)", border: "1px solid rgba(255, 255, 255, 0.15)", color: "#e2e8f0", padding: "0.35rem 0.75rem", "border-radius": "0.375rem", "font-size": "0.75rem", cursor: "pointer", "font-weight": "600" }}
              >
                📋 Logs de Sesión (session.log)
              </button>
              <button
                onClick={() => runCommand("wlr-randr 2>/dev/null || echo 'wlr-randr no disponible'")}
                style={{ background: "rgba(30, 41, 59, 0.9)", border: "1px solid rgba(255, 255, 255, 0.15)", color: "#e2e8f0", padding: "0.35rem 0.75rem", "border-radius": "0.375rem", "font-size": "0.75rem", cursor: "pointer", "font-weight": "600" }}
              >
                🖥️ Salida Wayland (wlr-randr)
              </button>
            </div>

            {/* 4. Console Log Screen */}
            <div
              ref={terminalOutputRef}
              style={{
                flex: "1",
                background: "#020408",
                border: "1px solid rgba(0, 240, 255, 0.25)",
                "border-radius": "0.375rem",
                padding: "0.875rem",
                "overflow-y": "auto",
                "font-family": "var(--font-mono)",
                "font-size": "0.8125rem",
                "line-height": "1.45",
                display: "flex",
                "flex-direction": "column",
                gap: "0.35rem"
              }}
            >
              <For each={outputHistory()}>
                {(line) => {
                  let textColor = '#cbd5e1';
                  if (line.type === 'cmd') textColor = '#00f0ff';
                  if (line.type === 'err') textColor = '#ef4444';
                  if (line.type === 'sys') textColor = '#94a3b8';

                  return (
                    <div style={{ color: textColor, "white-space": "pre-wrap", "word-break": "break-word" }}>
                      {line.text}
                    </div>
                  );
                }}
              </For>
              <Show when={isRunning()}>
                <div style={{ color: "#f59e0b", "font-weight": "bold" }}>
                  ⏳ Ejecutando comando...
                </div>
              </Show>
            </div>

            {/* 5. Command Input Form */}
            <form
              onSubmit={(e) => {
                e.preventDefault();
                runCommand(commandInput());
              }}
              style={{ display: "flex", gap: "0.5rem", "align-items": "center" }}
            >
              <span style={{ color: "#00f0ff", "font-family": "var(--font-mono)", "font-weight": "700", "font-size": "0.875rem" }}>
                emubox@arch:~$
              </span>
              <input
                ref={inputRef}
                type="text"
                value={commandInput()}
                onInput={(e) => setCommandInput(e.currentTarget.value)}
                placeholder="Escribe un comando (ej: ip a, systemctl status sshd, ./script.sh) y pulsa Enter..."
                disabled={isRunning()}
                style={{
                  flex: "1",
                  background: "rgba(15, 23, 42, 0.9)",
                  border: "1px solid rgba(0, 240, 255, 0.4)",
                  color: "#ffffff",
                  padding: "0.625rem 0.875rem",
                  "border-radius": "0.375rem",
                  "font-family": "var(--font-mono)",
                  "font-size": "0.875rem",
                  outline: "none"
                }}
              />
              <button
                type="submit"
                disabled={isRunning() || !commandInput().trim()}
                style={{
                  background: "#00f0ff",
                  color: "#05070a",
                  border: "none",
                  padding: "0.625rem 1.25rem",
                  "border-radius": "0.375rem",
                  "font-weight": "800",
                  "font-size": "0.8125rem",
                  cursor: isRunning() ? "not-allowed" : "pointer"
                }}
              >
                EJECUTAR [Enter]
              </button>
            </form>
          </Dialog.Content>
        </div>
      </Dialog.Portal>
    </Dialog>
  );
};
