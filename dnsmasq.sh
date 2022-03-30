#!/usr/bin/env nix-shell
#!nix-shell -i bash -p ipxe qemu "dnsmasq.overrideAttrs(old: {preBuild = ''makeFlagsArray=(\"COPTS=-DNO_INOTIFY\")'';})"

set -eux

scratch=$(mktemp -d -t tmp.XXXXXXXXXX)
function finish {
  rm -rf "$scratch"
}
trap finish EXIT

while ! ip link show br0; do
  sleep 0
done

IPXE_ROOT=$(echo $buildInputs | tr ' ' '\n' | grep ipxe)
URL="http://192.168.0.1:3030/foo"
URL=http://10.0.2.1:3030/dispatch/configuration/small

# For the vendorclass matches down below, see:
# https://serverfault.com/questions/758384/dnsmasq-proxy-mode-and-efi-boot

dnsmasq --leasefile-ro \
  --no-daemon \
  --pid-file="$scratch/pid" \
  --port 0 \
  --listen-address=10.0.2.1 \
  --dhcp-broadcast \
  --dhcp-authoritative \
  --dhcp-range=10.0.2.15,10.0.2.30,255.255.255.0,1h \
  --dhcp-option=3,10.0.2.2 \
  --dhcp-option=6,8.8.8.8 \
  --enable-tftp \
  --tftp-root=$IPXE_ROOT \
  --dhcp-match=IPXE,175 \
  --dhcp-vendorclass=BIOS,PXEClient:Arch:00000 \
  --dhcp-vendorclass=UEFI,PXEClient:Arch:00007 \
  --dhcp-vendorclass=UEFI,PXEClient:Arch:00009 \
  --dhcp-boot="net:#IPXE,net:UEFI,ipxe.efi" \
  --dhcp-boot="net:IPXE,$URL"
