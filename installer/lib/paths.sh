#!/usr/bin/env bash
# ==============================================================================
#  EMUBOX INSTALLER - XDG PATHS RESOLUTION
# ==============================================================================

export XDG_CONFIG_HOME="${XDG_CONFIG_HOME:-$HOME/.config}"
export XDG_DATA_HOME="${XDG_DATA_HOME:-$HOME/.local/share}"
export XDG_CACHE_HOME="${XDG_CACHE_HOME:-$HOME/.cache}"
export XDG_STATE_HOME="${XDG_STATE_HOME:-$HOME/.local/state}"

export EMUBOX_CONFIG_DIR="${XDG_CONFIG_HOME}/emubox"
export EMUBOX_DATA_DIR="${XDG_DATA_HOME}/emubox"
export EMUBOX_CACHE_DIR="${XDG_CACHE_HOME}/emubox"
export EMUBOX_LOGS_DIR="${XDG_STATE_HOME}/emubox/logs"

export EMUBOX_CONFIG_FILE="${EMUBOX_CONFIG_DIR}/config.json"
export EMUBOX_EMULATORS_FILE="${EMUBOX_CONFIG_DIR}/emulators.json"
export EMUBOX_CONTROLLERS_FILE="${EMUBOX_CONFIG_DIR}/controllers.json"
