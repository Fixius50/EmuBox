#!/usr/bin/env bash

set -euo pipefail

# ============================================================
# EmuBox - Instalador de dependencias y despliegue en Arch Linux
# ============================================================
#
# Prepara un Arch Linux limpio para compilar y ejecutar EmuBox.
#
# NO instala:
#   - Wayland como entorno de escritorio
#   - X11 como entorno de escritorio
#   - GNOME / KDE / XFCE
#   - Display Managers innecesarios
#
# ============================================================

EMUBOX_DIR="/opt/emubox"
REPO_URL="https://github.com/Fixius50/EmuBox.git"

echo "=========================================="
echo "        EmuBox - System Setup"
echo "=========================================="
echo

# ------------------------------------------------------------
# 1. Comprobar root
# ------------------------------------------------------------

if [[ "$EUID" -ne 0 ]]; then
    echo "ERROR: ejecuta este script como root."
    echo
    echo "Ejemplo:"
    echo "  sudo ./scripts/install-emubox.sh"
    exit 1
fi

# ------------------------------------------------------------
# 2. Actualizar sistema
# ------------------------------------------------------------

echo "[1/7] Actualizando Arch Linux..."

pacman -Syu --noconfirm

# ------------------------------------------------------------
# 3. Dependencias generales de compilación
# ------------------------------------------------------------

echo
echo "[2/7] Instalando herramientas de compilación..."

pacman -S --needed --noconfirm \
    base-devel \
    git \
    curl \
    wget \
    unzip \
    zip \
    pkgconf

# ------------------------------------------------------------
# 4. Dependencias de Tauri / WebKitGTK
# ------------------------------------------------------------

echo
echo "[3/7] Instalando dependencias de Tauri..."

pacman -S --needed --noconfirm \
    webkit2gtk-4.1 \
    gtk3 \
    librsvg

# ------------------------------------------------------------
# 5. Node.js + npm
# ------------------------------------------------------------

echo
echo "[4/7] Instalando Node.js y npm..."

pacman -S --needed --noconfirm \
    nodejs \
    npm

# ------------------------------------------------------------
# 6. Rust + Cargo
# ------------------------------------------------------------

echo
echo "[5/7] Instalando Rust y Cargo..."

pacman -S --needed --noconfirm \
    rust

# ------------------------------------------------------------
# 7. Preparar repositorio EmuBox
# ------------------------------------------------------------

echo
echo "[6/7] Preparando EmuBox..."

if [[ -d "$EMUBOX_DIR/.git" ]]; then

    echo "Repositorio existente detectado en $EMUBOX_DIR."
    echo "Actualizando via git pull..."

    git -C "$EMUBOX_DIR" fetch origin
    git -C "$EMUBOX_DIR" pull --ff-only

elif [[ -d "$(pwd)/.git" && -f "$(pwd)/package.json" ]]; then

    echo "Ejecutando desde el repositorio local de desarrollo: $(pwd)"
    mkdir -p "$(dirname "$EMUBOX_DIR")"
    if [[ ! -d "$EMUBOX_DIR" ]]; then
        echo "Copiando arbol de fuentes a $EMUBOX_DIR..."
        cp -r "$(pwd)" "$EMUBOX_DIR"
    fi

else

    echo "Clonando repositorio remoto en $EMUBOX_DIR..."

    mkdir -p "$(dirname "$EMUBOX_DIR")"

    git clone "$REPO_URL" "$EMUBOX_DIR"

fi

# ------------------------------------------------------------
# 8. Instalar dependencias npm
# ------------------------------------------------------------

echo
echo "[7/7] Instalando dependencias de EmuBox..."

cd "$EMUBOX_DIR"

if [[ -f "package-lock.json" ]]; then
    npm ci
else
    npm install
fi

# ------------------------------------------------------------
# 9. Compilar
# ------------------------------------------------------------

echo
echo "=========================================="
echo "       Compilando EmuBox"
echo "=========================================="
echo

npm run build

# ------------------------------------------------------------
# 10. Estructura de directorios de datos
# ------------------------------------------------------------

mkdir -p /var/lib/emubox/{games,roms,saves,states,bios,covers,logs}
mkdir -p /etc/emubox

echo
echo "=========================================="
echo "       Instalacion completada con exito"
echo "=========================================="
echo
echo "EmuBox esta preparado en:"
echo "  $EMUBOX_DIR"
echo
echo "Estructura de datos creada en:"
echo "  /var/lib/emubox/"
echo
echo "Dependencias instaladas:"
echo "  - Node.js y npm"
echo "  - Rust y Cargo"
echo "  - WebKitGTK 4.1 y GTK3"
echo "  - Herramientas de compilacion (base-devel, git, pkgconf)"
echo
echo "NOTA:"
echo "Este script NO instala ningun entorno de escritorio."
echo
