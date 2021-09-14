#!/bin/sh

set -eux

ENVID=oogabooga
unshare \
    --fork \
    --pid \
    --mount \
    --net \
    --user \
    --map-root-user \
    --map-group=0 \
    --setuid 0 \
    --mount-proc \
    --kill-child \
    -- \
    sh -c 'sleep infinity; echo "'"$ENVID"'"' &
unsharepid=$!

function finish {
  echo "Killing unshare, pid $unsharepid"
  kill -9 "$unsharepid"
}
trap finish EXIT

echo "Waiting for unshare to finish..."
wait
