import struct
import sys
import time
from typing import Optional, Tuple

import numpy as np

from presto import lockin
from presto.hardware import AdcMode, DacMode


def main(*, address: str, port: Optional[int] = None):
    IN_PORT = 1
    OUT_PORT = 1

    ATTENUATION = 0.0
    CURRENT = 27_500
    AMP = 1.0

    NSW = 16  # number of sliding widnows
    DF = 16e3
    FREQ = 2.4e9

    with lockin.Lockin(
        address=address,
        port=port,
        ext_ref_clk=10e6,
        adc_mode=AdcMode.Mixed,
        dac_mode=DacMode.Mixed,
    ) as lck:
        # set up hardware and lockin stuff
        # IMPORTANT: set up starting DC bias and analog range on X, Y, Z
        lck.hardware.set_dc_bias(0.0, 1, range_i=2)  # Z piezo, ±3.3 V
        lck.hardware.set_dc_bias(0.0, 2, range_i=2)  # X piezo, ±3.3 V
        lck.hardware.set_dc_bias(0.0, 3, range_i=2)  # Y piezo, ±3.3 V
        lck.hardware.set_adc_attenuation(IN_PORT, ATTENUATION)
        lck.hardware.set_dac_current(OUT_PORT, CURRENT)
        lck.hardware.set_inv_sinc(OUT_PORT, 0)
        lck.hardware.configure_mixer(FREQ, in_ports=IN_PORT, out_ports=OUT_PORT)

        # set df to DF*NSW, and use nsum later (RPU uses NSW-long sliding sum)
        _, df = lck.tune_perfect(0.0, DF * NSW)
        lck.set_df(df)
        ig = lck.add_input_group(IN_PORT, 1)
        ig.set_frequencies(0.0)
        og = lck.add_output_group(OUT_PORT, 1)
        og.set_frequencies(0.0).set_amplitudes(AMP)
        lck.apply_settings()
        lck.hardware.sleep(1)

        # program scaling factor for feedback
        program_scale(lck, NSW)
        program_limits(lck, 0.0, 1.0)
        # program (starting) feedback parameters
        program_feedback(lck, 0.001, 660.0, 69.0, 4200.0)

        with lck.stream_pixels(
            summed=True,
            nsum=NSW,
            rpu_params=("qafm", [0, 1], NSW),
        ) as rcv:
            # monitor feedback results
            while True:
                print_pix(rcv, IN_PORT)
                print_all(lck)
                time.sleep(1)


def program_scale(lck: lockin.Lockin, nsw: int):
    """Set the scaling factor for lockin data to the RPU.

    Should be set *before* the feedback is started.
    """
    scale_acc = 1.0 / 0xFFEE_801F
    scale_spp = 1.0 / lck.get_ns("adc")  # scale by nr of samples in a pixel
    scale_slw = 1.0 / nsw  # divide by NSW to get sliding average instead of sliding sum
    scale = scale_acc * scale_spp * scale_slw

    # high bits not used, set to 0.0
    lck.hardware.set_rpu_param(2, f32x2_to_u64(scale, 0.0))


def program_limits(lck: lockin.Lockin, low: float, high: float):
    """Set the DC output limits for the Z piezo.

    Should be set *before* the feedback is started.

    The units are normalized such that 0.0 is full-scale low bias, and 1.0 is full-scale high bias.
    """
    # some very basic input validation
    assert low >= 0.0
    assert high <= 1.0
    assert low < high
    lck.hardware.set_rpu_param(6, f32x2_to_u64(low, high))


def program_feedback(
    lck: lockin.Lockin,
    sp: float,
    kp: float,
    ki: float,
    kd: float,
):
    """Set the PID controller parameters to the RPU.

    Can be changed while the feedback is running.

    Args:
        lck: an active instance of Lockin
        sp: set point
        kp: proportional gain
        ki: integral gain
        kd: derivative gain
    """
    lck.hardware.set_rpu_param(3, f32x2_to_u64(sp, kp))
    lck.hardware.set_rpu_param(4, f32x2_to_u64(ki, kd))


def print_all(lck: lockin.Lockin):
    """Print some values received from the RPU.

    It will print:
    - the current value of amplitude squared (the error signal)
    - the current value of normalized DC bias on the Z piezo (the control signal)
    - the number of processed iterations since the start of the feedback

    Args:
        lck: an active instance of Lockin
    """
    # read number of processed iterations
    nr_irq, _ = u64_to_u32x2(lck.hardware.get_rpu_param(0))
    amp2, z_bias = u64_to_f32x2(lck.hardware.get_rpu_param(1))
    print(f"Amp2 (RPU): {amp2:.2e}")
    print(f"Bias: {z_bias:.4f}")
    print(f"IRQs: {nr_irq:d}")
    print()


def print_pix(rcv: lockin.LockinReceiver, in_port: int):
    """Print the current value of amplitude squared as seen from the "normal" lockin.

    This method receives one "summed" pixel, which is the average of ``nsw`` raw pixels.

    Args:
        lck: an active instance of Lockin
        in_port: the RF input port
    """
    # using summed Lockin
    _, mean_i, _, mean_q, _ = rcv.get_last(1)[in_port]
    # using zero IF
    data = mean_i[0, 0].real + 1j * mean_q[0, 0].real
    amp2 = np.abs(data) ** 2
    print(f"Amp2 (APU): {amp2:.2e}")


def u64_to_f32x2(val: int) -> Tuple[float, float]:
    """Convenience function to extract two f32 values from one u64 value"""
    low = val & 0xFFFF_FFFF
    high = (val >> 32) & 0xFFFF_FFFF
    low = struct.unpack("<f", struct.pack("<I", low))[0]
    high = struct.unpack("<f", struct.pack("<I", high))[0]
    return (low, high)


def u64_to_u32x2(val: int) -> Tuple[int, int]:
    """Convenience function to extract two u32 values from one u64 value"""
    low = val & 0xFFFF_FFFF
    high = (val >> 32) & 0xFFFF_FFFF
    return (low, high)


def f32x2_to_u64(low: float, high: float) -> int:
    """Convenience function to pack two f32 values into one u64 value"""
    low = int.from_bytes(struct.pack("<f", low), byteorder="little")
    high = int.from_bytes(struct.pack("<f", high), byteorder="little")
    val = low & 0xFFFF_FFFF
    val |= (high & 0xFFFF_FFFF) << 32
    return val


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
