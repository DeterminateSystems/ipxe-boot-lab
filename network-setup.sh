#!/bin/sh

ip link set lo up
ip link add br0 type bridge
ip link set br0 up
ip addr add 10.0.2.1/24 dev br0
sleep infinity
