#!/usr/bin/env bash
# ==============================================================================
#  EMUBOX SETUP - UDEV RULES, USER GROUPS & REALTIME AUDIO PERMISSIONS
# ==============================================================================

set -euo pipefail

CURRENT_USER="$(whoami)"

echo "  -> Configurando permisos de usuario y reglas udev para mandos..."

# 1. Add current user to input, video, render and audio groups
for grp in input video render audio; do
  if getent group "$grp" >/dev/null 2>&1; then
    sudo usermod -aG "$grp" "${CURRENT_USER}" || true
  fi
done

# 2. Udev rules for controllers (Xbox, PlayStation DualSense/DualShock, Nintendo Pro, 8BitDo)
UDEV_RULE_FILE="/etc/udev/rules.d/99-emubox-gamepads.rules"

if [[ ! -f "${UDEV_RULE_FILE}" ]]; then
  echo "  -> Creando reglas udev para acceso directo a mandos USB/Bluetooth..."
  sudo tee "${UDEV_RULE_FILE}" > /dev/null << 'EOF'
# EmuBox OS - Gamepad direct access without root privileges
# Xbox 360 / Xbox One / Series X
KERNEL=="uinput", MODE="0660", GROUP="input", OPTIONS+="static_node=uinput"
SUBSYSTEM=="input", ATTRS{name}=="*Xbox*", MODE="0666"
# Sony DualShock 4 / DualSense
SUBSYSTEM=="input", ATTRS{name}=="*Wireless Controller*", MODE="0666"
SUBSYSTEM=="input", ATTRS{name}=="*DualSense*", MODE="0666"
# Nintendo Switch Pro Controller
SUBSYSTEM=="input", ATTRS{name}=="*Pro Controller*", MODE="0666"
# 8BitDo Controllers
SUBSYSTEM=="input", ATTRS{name}=="*8BitDo*", MODE="0666"
# Generic DirectInput / XInput USB Gamepads
KERNEL=="js*", MODE="0666"
KERNEL=="event*", ENV{ID_INPUT_JOYSTICK}=="1", MODE="0666"
EOF
  sudo udevadm control --reload-rules || true
  sudo udevadm trigger || true
  echo "  ✓ Reglas udev de mandos configuradas."
else
  echo "  ✓ Reglas udev de mandos ya existentes."
fi

# 3. Realtime Audio limits for low-latency PipeWire / WebAudio
LIMITS_FILE="/etc/security/limits.d/99-emubox-realtime.conf"
if [[ ! -f "${LIMITS_FILE}" ]]; then
  echo "  -> Configurando prioridad de audio en tiempo real..."
  sudo tee "${LIMITS_FILE}" > /dev/null << 'EOF'
@audio - rtprio 95
@audio - memlock unlimited
@audio - nice -19
EOF
  echo "  ✓ Prioridad de audio en tiempo real configurada."
fi
