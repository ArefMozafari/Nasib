#!/usr/bin/env bash
# Builds and tests Nasib for Linux in an Ubuntu 22.04 container, so it works from any machine with Docker.
#   ./Linux/build.sh          build and run the app's and the engine's tests (this machine's architecture)
#   ./Linux/build.sh release  the same for x64, then write the .deb, .rpm and .tar.gz into Linux/build/
set -euo pipefail
cd "$(dirname "$0")/.."

IMAGE_NAME="nasib-linux-build"
RELEASE_PLATFORM="linux/amd64"
# The build folder lives in a volume, one per architecture. On the mounted host folder, an emulated x64
# build sometimes can't see a library it has just written, and fails at random.
TARGET_DIRECTORY="/target"

# Runs a command in the build container for the given platform, with the repository mounted and the
# downloaded crates and build folder kept in volumes between runs.
in_container() {
  local platform="$1"; shift
  docker build --quiet --platform "$platform" --tag "$IMAGE_NAME:${platform//\//-}" Linux >/dev/null
  docker run --rm --platform "$platform" \
    --volume "$PWD:/repository" \
    --volume "$IMAGE_NAME-cargo-registry:/usr/local/cargo/registry" \
    --volume "$IMAGE_NAME-target-${platform//\//-}:$TARGET_DIRECTORY" \
    --env CARGO_TARGET_DIR="$TARGET_DIRECTORY" \
    --workdir /repository \
    "$IMAGE_NAME:${platform//\//-}" "$@"
}

host_platform() {
  case "$(uname -m)" in
    arm64|aarch64) echo "linux/arm64" ;;
    *) echo "linux/amd64" ;;
  esac
}

run_tests() {
  local platform="$1"
  in_container "$platform" sh -c 'cargo test --manifest-path Engine/Cargo.toml && cargo test --manifest-path Linux/Cargo.toml'
}

if [[ "${1:-}" == "release" ]]; then
  run_tests "$RELEASE_PLATFORM"
  in_container "$RELEASE_PLATFORM" sh Linux/packaging/package.sh
else
  run_tests "$(host_platform)"
fi
