#!/bin/sh
set -x

ip link

ip link set dev "$1" master br0
ip link set dev "$1" up
ip link
