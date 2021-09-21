#!/bin/sh
set -eu

ip link set dev "$1" master br0
ip link set dev "$1" up
