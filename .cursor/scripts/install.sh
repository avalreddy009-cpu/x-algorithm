#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
PHOENIX_DIR="${REPO_ROOT}/phoenix"

if [[ ! -d "${PHOENIX_DIR}" ]]; then
  echo "Phoenix directory not found at ${PHOENIX_DIR}" >&2
  exit 1
fi

cd "${PHOENIX_DIR}"

# Ensure protoc is available (Dockerfile installs it; guard for default-image runs).
if ! command -v protoc >/dev/null 2>&1; then
  echo "protoc not found; installing protoc 28.3..." >&2
  ARCH="$(uname -m)"
  case "${ARCH}" in
    x86_64) PROTOC_ARCH="x86_64" ;;
    aarch64) PROTOC_ARCH="aarch_64" ;;
    *) echo "Unsupported architecture for protoc install: ${ARCH}" >&2; exit 1 ;;
  esac
  curl -fsSL -o /tmp/protoc.zip \
    "https://github.com/protocolbuffers/protobuf/releases/download/v28.3/protoc-28.3-linux-${PROTOC_ARCH}.zip"
  sudo unzip -o /tmp/protoc.zip -d /usr/local 'bin/*' 'include/*'
fi

# Ensure uv is available.
if ! command -v uv >/dev/null 2>&1; then
  echo "uv not found; installing uv..." >&2
  curl -LsSf https://astral.sh/uv/install.sh | sh
  export PATH="${HOME}/.local/bin:${PATH}"
fi

# Ensure Rust >= 1.85 is available for edition-2024 crates in the serving engine.
need_rust_install=false
if ! command -v rustc >/dev/null 2>&1; then
  need_rust_install=true
else
  rust_minor="$(rustc --version | awk '{print $2}' | cut -d. -f2)"
  if [[ "${rust_minor}" -lt 85 ]]; then
    echo "Rust $(rustc --version) is too old; upgrading to stable..." >&2
    need_rust_install=true
  fi
fi
if [[ "${need_rust_install}" == "true" ]]; then
  curl -fsSL https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
fi
if [[ -f "${HOME}/.cargo/env" ]]; then
  # shellcheck disable=SC1091
  source "${HOME}/.cargo/env"
elif [[ -f "/root/.cargo/env" ]]; then
  # shellcheck disable=SC1091
  source "/root/.cargo/env"
fi
# System-wide rustup installs (common on Cloud Agent images) may pin an old default.
if command -v rustup >/dev/null 2>&1; then
  rustup default stable >/dev/null 2>&1 || true
fi

echo "Installing Phoenix Python dependencies and Rust serving engine..."
# Cargo's default cc crate passes clang-only --target flags that break vendored rdma-core.
export CC=gcc
export CXX=g++
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-/tmp/phoenix-cargo}"
uv sync --extra engine

echo "Verifying Phoenix Python imports..."
export PYTHONPATH="${PHOENIX_DIR}"
uv run python -c "import jax; import xrex; print('jax devices:', jax.devices())"

echo "Phoenix install complete."
