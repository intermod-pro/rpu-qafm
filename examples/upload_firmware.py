import sys
from typing import Optional

from presto import hardware

FW_PATH = "../target/armv7r-none-eabihf/release/qafm"


def main(*, address: str, port: Optional[int] = None):
    with hardware.Hardware(address=address, port=port) as hw:
        hw.upload_rpu_firmware(FW_PATH)


if __name__ == "__main__":
    if len(sys.argv) == 2:
        address = sys.argv[1]
        port = None
    elif len(sys.argv) == 3:
        address = sys.argv[1]
        port = int(sys.argv[2])
    else:
        raise RuntimeError("IP address missing! Usage: `python scriptname.py ADDRESS [PORT]`")

    main(address=address, port=port)
