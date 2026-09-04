#!/usr/bin/env bash
# ==============================================================================
#  EMUBOX INSTALLER - XDG PATHS RESOLUTION
# ==============================================================================

export EMUBOX_CONFIG_DIR="/etc/emubox"
export EMUBOX_DATA_DIR="/var/lib/emubox"
export EMUBOX_CACHE_DIR="/var/cache/emubox"
export EMUBOX_LOGS_DIR="/var/log/emubox"
export EMUBOX_RUNTIME_DIR="/run/emubox"

export EMUBOX_CONFIG_FILE="${EMUBOX_CONFIG_DIR}/config.json"
export EMUBOX_EMULATORS_FILE="${EMUBOX_CONFIG_DIR}/emulators.json"
export EMUBOX_CONTROLLERS_FILE="${EMUBOX_CONFIG_DIR}/controllers.json"
