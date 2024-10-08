#! /usr/bin/bash

IMAGE="rpu-qafm"
TOOL="podman"

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

  "")
    echo "specify one action: build or run"
    ;;

  *)
    echo "unrecognized action: $1"
    ;;

esac
