@echo off

set IMAGE="rpu-qafm"
set TOOL="podman"

if [%1] == [build] (
    %TOOL% build -t %IMAGE% .
) else if [%1] == [run] (
    %TOOL% run -ti --rm -v .:/root/workspace %IMAGE% %2 %3 %4 %5 %6 %7 %8 %9
) else if [%1] == [] (
    echo specify one action: build or run
) else (
    echo unrecognized action: %1%
)
