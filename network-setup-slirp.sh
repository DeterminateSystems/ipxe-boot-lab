#!/bin/sh

set -eux

target_tap=tap42

# Wait for tap interface
while ! ip link show "$target_tap"; do
  sleep 1
done

# Wait for bridge interface
while ! ip link show br0; do
  sleep 1
done

ip link set dev "$target_tap" master br0
ip link set dev "$target_tap" up

# slirp4netns can't provide a next boot
nft "add table netdev foo; add chain netdev foo nodhcp { type filter hook ingress device $target_tap priority 0; policy accept; }"
nft add rule netdev foo nodhcp udp sport 67 drop

sleep infinity