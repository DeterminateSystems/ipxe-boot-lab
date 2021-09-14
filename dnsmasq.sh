#!/usr/bin/env nix-shell
#!nix-shell -i bash -p qemu "dnsmasq.overrideAttrs(old: {preBuild = ''makeFlagsArray=(\"COPTS=-DNO_INOTIFY\")'';})"

scratch=$(mktemp -d -t tmp.XXXXXXXXXX)
function finish {
  rm -rf "$scratch"
}
trap finish EXIT

while ! ip link show br0; do
  sleep 0
done

URL="http://192.168.0.1:3030/foo"
URL=http://10.0.2.1:3030/dispatch/configuration/small

dnsmasq --leasefile-ro \
  --dhcp-broadcast \
  --dhcp-authoritative \
  -p 0 \
  -F 10.0.2.15,10.0.2.30,255.255.255.0,1h \
  -O 3,10.0.2.2 \
  -O 6,8.8.8.8 \
  -M "$URL" \
  --pid-file="$scratch/pid" \
  --listen-address=10.0.2.1 -d
