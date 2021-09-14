#!/bin/sh

set -eux

ENVID=oogabooga
ident='sh -c sleep infinity; echo "'"$ENVID"'"'

while ! pgrep -xf "$ident"; do
  sleep 0
done

pid=$(pgrep -xf 'sh -c sleep infinity; echo "'"$ENVID"'"')

nsenter -t "$pid" -U -n --preserve-credentials "$@"
