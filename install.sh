#!/usr/bin/env sh
# PAI-Kernel install script — detects OS/arch, downloads the appropriate
# v2.2.2 release binary, verifies the SHA256 checksum, and installs
# pai_governance_daemon + default config + policies to ~/.local/pai-kernel.
#
# Usage:
#   curl -fsSL https://paikernel.org/install.sh | sh
#   curl -fsSL https://raw.githubusercontent.com/PAI-Kernel/pai-kernel/v2.2.2/install.sh | sh
#
# Environment overrides:
#   PAI_KERNEL_VERSION   tag to install (default: v2.2.2)
#   PAI_KERNEL_INSTALL_DIR   install location (default: $HOME/.local/pai-kernel)
#   PAI_KERNEL_ADD_TO_PATH   "yes"/"no" — offer PATH append (default: interactive)
#
# This script:
#   - Works on macOS (Intel + Apple Silicon), Linux (x64 + ARM64), WSL
#   - Does NOT require Rust toolchain
#   - Does NOT run the daemon automatically (prints next-step command)
#   - Aborts cleanly on error, leaves no partial install
#
# Windows users: install manually from the release page or via Docker.
# See https://github.com/PAI-Kernel/pai-kernel/blob/v2.2.2/docs/INSTALL.md

set -eu

VERSION="${PAI_KERNEL_VERSION:-v2.2.2}"
INSTALL_DIR="${PAI_KERNEL_INSTALL_DIR:-$HOME/.local/pai-kernel}"
REPO="PAI-Kernel/pai-kernel"
BASE_URL="https://github.com/${REPO}/releases/download/${VERSION}"

# ---- helpers ----
info()  { printf '\033[0;34m[info]\033[0m  %s\n' "$*"; }
warn()  { printf '\033[0;33m[warn]\033[0m  %s\n' "$*" >&2; }
error() { printf '\033[0;31m[err]\033[0m   %s\n' "$*" >&2; exit 1; }

# ---- detect OS + arch ----
detect_target() {
  uname_s="$(uname -s)"
  uname_m="$(uname -m)"

  case "$uname_s" in
    Darwin)
      os="apple-darwin"
      case "$uname_m" in
        arm64|aarch64) arch="aarch64" ;;
        x86_64)        arch="x86_64" ;;
        *) error "unsupported macOS architecture: $uname_m" ;;
      esac
      ;;
    Linux)
      os="unknown-linux-gnu"
      case "$uname_m" in
        x86_64|amd64)  arch="x86_64" ;;
        aarch64|arm64) arch="aarch64" ;;
        *) error "unsupported Linux architecture: $uname_m" ;;
      esac
      ;;
    *)
      error "unsupported OS: $uname_s (Windows users: see docs/INSTALL.md Option A manual)" ;;
  esac

  echo "${arch}-${os}"
}

# ---- required tools ----
need() {
  command -v "$1" >/dev/null 2>&1 || error "missing required tool: $1"
}

main() {
  need curl
  need tar
  need uname

  target="$(detect_target)"
  archive="pai_governance_daemon-${VERSION}-${target}.tar.gz"
  url="${BASE_URL}/${archive}"
  checksum_url="${url}.sha256"

  info "PAI-Kernel installer"
  info "  version: ${VERSION}"
  info "  target:  ${target}"
  info "  install: ${INSTALL_DIR}"
  info ""

  # Check if already installed
  if [ -d "${INSTALL_DIR}" ] && [ -x "${INSTALL_DIR}/pai_governance_daemon" ]; then
    existing_version="$("${INSTALL_DIR}/pai_governance_daemon" version 2>&1 | head -1 || echo unknown)"
    warn "Existing install detected at ${INSTALL_DIR} (${existing_version})"
    printf '       overwrite? [y/N] '
    read -r reply || reply="n"
    case "$reply" in
      y|Y|yes|YES) info "overwriting..." ;;
      *) info "aborted."; exit 0 ;;
    esac
  fi

  # tmp working directory
  tmp="$(mktemp -d -t pai-kernel-install-XXXXXX)"
  trap 'rm -rf "$tmp"' EXIT

  info "Downloading ${archive}..."
  curl -fSL -o "${tmp}/${archive}" "${url}" || error "download failed: ${url}"

  info "Downloading checksum..."
  if curl -fSL -o "${tmp}/${archive}.sha256" "${checksum_url}" 2>/dev/null; then
    info "Verifying SHA256..."
    cd "${tmp}"
    if command -v sha256sum >/dev/null 2>&1; then
      sha256sum -c "${archive}.sha256" || error "checksum mismatch — aborting"
    elif command -v shasum >/dev/null 2>&1; then
      shasum -a 256 -c "${archive}.sha256" || error "checksum mismatch — aborting"
    else
      warn "no sha256 tool found; skipping verification"
    fi
    cd - >/dev/null
  else
    warn "checksum not available at ${checksum_url}; skipping verification"
  fi

  info "Extracting..."
  tar -xzf "${tmp}/${archive}" -C "${tmp}"
  extracted_dir="${tmp}/pai_governance_daemon-${VERSION}-${target}"
  [ -d "${extracted_dir}" ] || error "expected directory not found: ${extracted_dir}"

  info "Installing to ${INSTALL_DIR}..."
  mkdir -p "${INSTALL_DIR}"
  # Copy contents of extracted dir to install dir (binary + config + policies + docs)
  cp -R "${extracted_dir}/." "${INSTALL_DIR}/"
  chmod +x "${INSTALL_DIR}/pai_governance_daemon" 2>/dev/null || true

  # macOS: strip quarantine attribute if present (Gatekeeper workaround for unsigned binary)
  if [ "$(uname -s)" = "Darwin" ] && command -v xattr >/dev/null 2>&1; then
    xattr -d com.apple.quarantine "${INSTALL_DIR}/pai_governance_daemon" 2>/dev/null || true
  fi

  info ""
  info "Install complete."
  info ""

  # Offer PATH append
  bin_link="${HOME}/.local/bin/pai_governance_daemon"
  add_to_path="${PAI_KERNEL_ADD_TO_PATH:-}"

  case ":${PATH}:" in
    *":${HOME}/.local/bin:"*) path_ok=yes ;;
    *) path_ok=no ;;
  esac

  if [ "${path_ok}" = "yes" ] || [ "${add_to_path}" = "yes" ]; then
    mkdir -p "${HOME}/.local/bin"
    ln -sf "${INSTALL_DIR}/pai_governance_daemon" "${bin_link}"
    info "Symlinked to ~/.local/bin/pai_governance_daemon"
    info ""
    info "Next:"
    info "  pai_governance_daemon --config ${INSTALL_DIR}/pai-kernel.toml"
  else
    info "Next:"
    info "  ${INSTALL_DIR}/pai_governance_daemon --config ${INSTALL_DIR}/pai-kernel.toml"
    info ""
    info "(To use from any directory, add ${INSTALL_DIR} to your PATH or"
    info " create a symlink:  ln -s ${INSTALL_DIR}/pai_governance_daemon ~/.local/bin/)"
  fi

  info ""
  info "Documentation: ${INSTALL_DIR}/INSTALL.md"
  info "Known limits:  ${INSTALL_DIR}/KNOWN_LIMITATIONS.md"
  info "Source:        https://github.com/${REPO}"
}

main "$@"
