#!/usr/bin/env nix-shell
#!nix-shell -i bash ../nix-netboot-serve/shell.nix

set -eux

scratch=$(mktemp -d -t tmp.XXXXXXXXXX)
function finish {
  rm -rf "$scratch"
}
trap finish EXIT

while ! ip link show br0; do
  sleep 0
done

cd .. || exit 1
mkdir -p scratch/gc-roots scratch/cpio-cache
cd nix-netboot-serve || exit 1
RUST_LOG=example::api cargo run -- \
  --gc-root-dir ../scratch/gc-roots \
  --cpio-cache-dir ../scratch/cpio-cache/ \
  --config-dir ../nixos-install-equinix-metal/configurations/types \
  --listen 0.0.0.0:3030
