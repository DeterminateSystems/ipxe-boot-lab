#!/bin/sh

set -eux

ENVID=oogabooga
ident='sh -c sleep infinity; echo "'"$ENVID"'"'

while ! pgrep -xf "$ident"; do
  sleep 0
done

pid=$(pgrep -xf 'sh -c sleep infinity; echo "'"$ENVID"'"')

sleep 1; # don't ask me

slirp4netns "$pid" tap42

#slirp4netns --configure --mtu=65520 --cidr=192.168.0.0/24 --disable-host-loopback "$pid" tap0
