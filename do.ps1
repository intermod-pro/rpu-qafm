param($action)

$IMAGE = "rpu-qafm"
$TOOL = "podman"

Switch ($action) {
    "build" {
        & $TOOL build -t $IMAGE .
    }
    "run" {
        & $TOOL run -ti --rm -v .:/root/workspace $IMAGE $args
    }
    "" {
        write-host "specify one action: build or run"
    }
    default {
        write-host "unrecognized action: $action"
    }
}
