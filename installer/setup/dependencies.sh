#!/usr/bin/env bash
# ==============================================================================
#  EMUBOX SETUP - SYSTEM DEPENDENCIES & PACKAGE INSTALLER (ARCH LINUX)
# ==============================================================================

set -euo pipefail

# Core System & Compositor Dependencies
CORE_PKGS=(
  "gamescope"
  "pipewire"
  "pipewire-pulse"
  "pipewire-alsa"
  "wireplumber"
  "vulkan-icd-loader"
  "vulkan-tools"
  "libretro-core-info"
  "retroarch"
  "xdg-user-dirs"
  "libevdev"
)

# Optional Native Standalone Emulators (if available in official repos / AUR)
EMU_PKGS=(
  "pcsx2"
  "duckstation-qt"
  "mgba-qt"
  "flycast"
)

echo "  -> Comprobando gestor de paquetes de Arch Linux (pacman)..."

if command -v pacman >/dev/null 2>&1; then
  MISSING_CORE=()
  for pkg in "${CORE_PKGS[@]}"; do
    if ! pacman -Qi "$pkg" >/dev/null 2>&1; then
      MISSING_CORE+=("$pkg")
    fi
  done

  if [[ ${#MISSING_CORE[@]} -gt 0 ]]; then
    echo "  -> Instalando paquetes de sistema necesarios: ${MISSING_CORE[*]}"
    sudo pacman -S --needed --noconfirm "${MISSING_CORE[@]}" || {
      echo "  [ADVERTENCIA] Algunos paquetes no pudieron instalarse vía pacman estándar."
    }
  else
    echo "  ✓ Todos los paquetes de sistema esenciales ya están instalados."
  fi
else
  echo "  [AVISO] pacman no disponible. Saltando comprobación automática de paquetes."
fi
