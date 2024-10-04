#![no_std]
#![no_main]

use core::panic::PanicInfo;
use core::ptr;
use core::sync::atomic::{AtomicBool, Ordering};

use cortex_r::gic::{ICC, ICD};
use zup_rt::{entry, interrupt};

mod pid;
mod types;
use types::{BiasDac, Data, Params};
mod user;

static GOT_IRQ: AtomicBool = AtomicBool::new(false);

const ADDR_DATA: usize = 0x0000_8000; // ATCM1, 32 kiB
const ADDR_PARAMS: usize = 0xfffc_0060; // OCM _reserved, 160 B
const ADDR_PRESTO: usize = 0x8000_0000; // M_AXI_HPM0_LPD (LPD_PL)
const ADDR_BIAS_DAC: usize = ADDR_PRESTO + 0x60;

/// Block until new lockin data is available.
fn wait_for_new_data() {
    while !GOT_IRQ.swap(false, Ordering::Relaxed) {
        // no new data available, keep waiting
        core::hint::spin_loop();
    }
    // NOTE: more than one new IRQ might have arrived since last time we checked, meaning that
    // we are too slow to process every single IRQ. We choose to ignore that possibility and
    // just process what we can.
}

#[interrupt]
fn PL_PS_04() {
    GOT_IRQ.store(true, Ordering::Relaxed);
}

/// Read current value of CPU cycle counter.
///
/// This is the number of RPU clock cycles elapsed since the start of the program.
/// The RPU is clocked at approximately 500 MHz, so 2 ns per clock cycle.
fn read_cycle_counter() -> u32 {
    use core::arch::asm;

    let ccntr: u32;
    unsafe {
        asm!("MRC p15, 0, {}, c9, c13, 0", out(reg) ccntr); // Read PMCCNTR Register
    }
    ccntr
}

/// Set normalized `bias` to DC bias port `channel+1`.
///
/// Will block until the bias is set.
fn set_dc_bias(bias_dac: &BiasDac, channel: u64, bias: f32) {
    let bias_code = (bias * u16::MAX as f32) as u16;

    let mut word = 0x40_0000; // Write code to and update DAC Channel x
    word |= channel << 16;
    word |= bias_code as u64;

    write_bias_raw(bias_dac, word);
}

fn write_bias_raw(bias_dac: &BiasDac, word: u64) {
    // DAC-ready signal
    const PL_PS_01: u16 = 122;
    // wait for DAC to be ready
    while !irq_status(PL_PS_01) {}
    // send command to DAC
    bias_dac.write(word);
    // wait for DAC to be busy
    while irq_status(PL_PS_01) {}
}

fn irq_status(irq_nr: u16) -> bool {
    let icd = unsafe { ICD::steal() };
    let spi_nr = usize::from(irq_nr) - 32;
    let reg_nr = spi_nr / 32;
    let bit_nr = spi_nr % 32;
    let idx = reg_nr + 1;
    let mask = 1 << bit_nr;
    let reg = icd.ICDIDR().idx(idx).read();
    (reg & mask) > 0
}

#[panic_handler]
fn panic(_panic: &PanicInfo<'_>) -> ! {
    // flag we panicked
    let flag = 1 << 63; // set highest bit
    unsafe {
        let val = ptr::read_volatile(ADDR_PARAMS as *const u64);
        ptr::write_volatile(ADDR_PARAMS as *mut u64, val | flag);
    }
    // halt execution
    loop {}
}

#[entry]
fn main() -> ! {
    unsafe {
        const PL_PS_01: u16 = 122; // bias DAC ready, we read its status manually
        const PL_PS_04: u16 = 125; // DMA 2 `irq_byte_cnt` transferred, we have an IRQ handler

        // disable interrupt routing and signaling during configuration
        ICD::disable();
        ICC::disable();

        // unmask DMA IRQ
        ICD::unmask(PL_PS_04);
        // mask bias DAC IRQ
        ICD::mask(PL_PS_01);

        // route IRQ to R5#1
        ICD::route(PL_PS_04, 2);

        // set sensitivity
        ICD::set_sensitivity(PL_PS_04, true);
        ICD::set_sensitivity(PL_PS_01, false);

        // set priority mask to the lowest priority
        ICC::set_priority_mask(248);

        // set the priority of PL_PS_00 to the second lowest priority
        ICD::set_priority(PL_PS_04, 240);

        // enable interrupt signaling
        ICC::enable();

        // enable interrupt routing
        ICD::enable();

        // unmask IRQ
        cortex_r::enable_irq();
    }

    // create interface to shared data
    let data = unsafe { types::DataMapPtr::from_ptr(ADDR_DATA as *mut _) };

    // create interface to Presto registers
    let bias_dac = unsafe { types::BiasDacMapPtr::from_ptr(ADDR_BIAS_DAC as *mut _) };

    // create interface to parameters
    let params = unsafe { types::ParamsMapPtr::from_ptr(ADDR_PARAMS as *mut _) };

    // clear RPU status
    params.inner().idx(0).write(0);

    // initialize GOT_IRQ to false to avoid processing initial spurious IRQ (if any)
    GOT_IRQ.store(false, Ordering::Relaxed);

    // hand over to user logic
    user::user_logic(data.inner(), bias_dac.inner(), params.inner());
}
