#! /usr/bin/bash

IMAGE="rpu-qafm"
TOOL="/usr/bin/podman"

case "$1" in
  build)
    "$TOOL" build -t "$IMAGE" .
    ;;

  run)
    "$TOOL" run \
      -ti \
      --rm \
      -v .:/root/workspace \
      "$IMAGE" "${@:2}"
    ;;

  *)
    echo "unrecognized command: $1"
    ;;

esac
