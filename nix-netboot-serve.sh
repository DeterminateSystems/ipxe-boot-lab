#!/usr/bin/env nix-shell
#!nix-shell -i bash ../shell.nix

scratch=$(mktemp -d -t tmp.XXXXXXXXXX)
function finish {
  rm -rf "$scratch"
}
trap finish EXIT

while ! ip link show br0; do
  sleep 0
done

cd nix-netboot-serve
RUST_LOG=example::api cargo run -- --gc-root-dir ./gc-roots --config-dir /home/grahamc/projects/github.com/x/v2/configurations/types --profile-dir ./profiles/ --cpio-cache-dir ./cpio-cache/ --listen 0.0.0.0:3030
