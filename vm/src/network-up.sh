#!/bin/sh
set -eu

#ip link set dev bond0 down

ip link set dev "$1" master bond0
ip link set dev "$1" up

#ip link set dev bond0 down
#ip link set dev bond0 up

