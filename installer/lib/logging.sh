#!/usr/bin/env bash
# ==============================================================================
#  EMUBOX INSTALLER - LOGGING LIBRARY
# ==============================================================================

CLR_RESET="\033[0m"
CLR_BOLD="\033[1m"
CLR_CYAN="\033[36m"
CLR_GREEN="\033[32m"
CLR_YELLOW="\033[33m"
CLR_RED="\033[31m"

log_banner() {
  echo -e "${CLR_CYAN}${CLR_BOLD}"
  echo "  ███████╗███╗   ███╗██╗   ██╗██████╗  ██████╗ ██╗  ██╗"
  echo "  ██╔════╝████╗ ████║██║   ██║██╔══██╗██╔═══██╗╚██╗██╔╝"
  echo "  █████╗  ██╔████╔██║██║   ██║██████╔╝██║   ██║ ╚███╔╝ "
  echo "  ██╔══╝  ██║╚██╔╝██║██║   ██║██╔══██╗██║   ██║ ██╔██╗ "
  echo "  ███████╗██║ ╚═╝ ██║╚██████╔╝██████╔╝╚██████╔╝██╔╝ ██╗"
  echo "  ╚══════╝╚═╝     ╚═╝ ╚═════╝ ╚═════╝  ╚═════╝ ╚═╝  ╚═╝"
  echo "         Dedicated Console Environment for Arch Linux   "
  echo -e "${CLR_RESET}"
}

log_step() {
  echo -e "${CLR_CYAN}==>${CLR_BOLD} $1${CLR_RESET}"
}

log_ok() {
  echo -e "  ${CLR_GREEN}✓${CLR_RESET} $1"
}

log_warn() {
  echo -e "  ${CLR_YELLOW}⚠${CLR_RESET} $1"
}

log_error() {
  echo -e "  ${CLR_RED}✗ ERROR:${CLR_RESET} $1" >&2
}
