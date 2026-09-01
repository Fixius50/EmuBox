#!/usr/bin/env bash
# ==============================================================================
#  EMUBOX INSTALLER - PERMISSIONS & UDEV RULES MANAGEMENT
# ==============================================================================

setup_user_groups() {
  local user
  user="$(whoami)"
  for grp in input video render audio; do
    if getent group "$grp" >/dev/null 2>&1; then
      if command -v sudo >/dev/null 2>&1; then
        sudo usermod -aG "$grp" "${user}" || true
      fi
    fi
  done
  log_ok "Grupos de usuario configurados (input, video, render, audio)."
}

setup_udev_rules() {
  local rule_file="/etc/udev/rules.d/99-emubox-gamepads.rules"
  if [[ ! -f "${rule_file}" ]]; then
    log_step "Creando reglas udev para mandos..."
    if command -v sudo >/dev/null 2>&1; then
      sudo tee "${rule_file}" > /dev/null << 'EOF'
KERNEL=="uinput", MODE="0660", GROUP="input", OPTIONS+="static_node=uinput"
SUBSYSTEM=="input", ATTRS{name}=="*Xbox*", MODE="0666"
SUBSYSTEM=="input", ATTRS{name}=="*Wireless Controller*", MODE="0666"
SUBSYSTEM=="input", ATTRS{name}=="*DualSense*", MODE="0666"
SUBSYSTEM=="input", ATTRS{name}=="*Pro Controller*", MODE="0666"
SUBSYSTEM=="input", ATTRS{name}=="*8BitDo*", MODE="0666"
KERNEL=="js*", MODE="0666"
KERNEL=="event*", ENV{ID_INPUT_JOYSTICK}=="1", MODE="0666"
EOF
      sudo udevadm control --reload-rules || true
      sudo udevadm trigger || true
      log_ok "Reglas udev de mandos creadas y aplicadas."
    fi
  else
    log_ok "Reglas udev ya presentes."
  fi
}
