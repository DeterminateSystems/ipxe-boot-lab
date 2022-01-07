#!/bin/sh
set -eux

while ! mkdir ./net-up-lock; do
  echo "Waiting on the lock for $1..."
  sleep 0
done

function finish {
  rmdir ./net-up-lock
}
trap finish EXIT

ip link set dev bond0 down
ip link set dev "$1" master bond0
ip link set dev "$1" up
ip link set dev bond0 up

