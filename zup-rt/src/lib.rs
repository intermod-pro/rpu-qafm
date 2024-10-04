#![deny(warnings)]
#![no_std]

#[cfg(not(debug_assertions))]
use core::sync::atomic::{self, Ordering};

// use cortex_r::gic::{ICC, ICCIAR};
use cortex_r::gic::ICCIAR;
pub use zup_rt_macros::{entry, exception, interrupt};

#[allow(unused_attributes)]
#[no_mangle]
unsafe extern "C" fn DefaultHandler() {
    // Unhandled exception

    #[allow(clippy::empty_loop)]
    loop {
        // NOTE(compiler_fence) prevents LLVM from turning this infinite loop into an abort
        // instruction
        #[cfg(not(debug_assertions))]
        atomic::compiler_fence(Ordering::SeqCst);
    }
}

#[allow(non_camel_case_types)]
pub enum Exception {
    DefaultHandler,

    Undefined,
    SVC,
    PrefetchAbort,
    DataAbort,
    // IRQ,
    FIQ,
}

#[allow(non_camel_case_types)]
pub enum Interrupt {
    SG0,
    SG1,
    SG2,
    SG3,
    SG4,
    SG5,
    SG6,
    SG7,
    SG8,
    SG9,
    SG10,
    SG11,
    SG12,
    SG13,
    SG14,
    SG15,
    IPI_CH1,
    IPI_CH2,
    PL_PS_00,
    PL_PS_01,
    PL_PS_02,
    PL_PS_03,
    PL_PS_04,
    PL_PS_05,
    PL_PS_06,
    PL_PS_07,
}

// TODO consider rewriting this in assembly to make it constant time and as fast as possible
#[allow(unused_attributes)]
#[no_mangle]
unsafe extern "C" fn IRQ(icciar: ICCIAR) {
    // cortex_r::enable_irq(); /* done in asm.s */
    let ackintid = icciar.ackintid();
    if ackintid == 1023 {
        // spurious interrupt; ignore
        // return;
    } else if ackintid < 16 {
        extern "C" {
            fn SG0();
            fn SG1();
            fn SG2();
            fn SG3();
            fn SG4();
            fn SG5();
            fn SG6();
            fn SG7();
            fn SG8();
            fn SG9();
            fn SG10();
            fn SG11();
            fn SG12();
            fn SG13();
            fn SG14();
            fn SG15();
        }

        static VECTORS: [unsafe extern "C" fn(); 16] = [
            SG0, SG1, SG2, SG3, SG4, SG5, SG6, SG7, SG8, SG9, SG10, SG11, SG12, SG13, SG14, SG15,
        ];

        VECTORS.get_unchecked(usize::from(ackintid))();
    } else if ackintid == 65 {
        extern "C" {
            fn IPI_CH1();
        }

        IPI_CH1();
    } else if ackintid == 66 {
        extern "C" {
            fn IPI_CH2();
        }

        IPI_CH2();
    } else if ackintid == 121 {
        extern "C" {
            fn PL_PS_00();
        }

        PL_PS_00();
    } else if ackintid == 122 {
        extern "C" {
            fn PL_PS_01();
        }

        PL_PS_01();
    } else if ackintid == 123 {
        extern "C" {
            fn PL_PS_02();
        }

        PL_PS_02();
    } else if ackintid == 124 {
        extern "C" {
            fn PL_PS_03();
        }

        PL_PS_03();
    } else if ackintid == 125 {
        extern "C" {
            fn PL_PS_04();
        }

        PL_PS_04();
    } else if ackintid == 126 {
        extern "C" {
            fn PL_PS_05();
        }

        PL_PS_05();
    } else if ackintid == 127 {
        extern "C" {
            fn PL_PS_06();
        }

        PL_PS_06();
    } else if ackintid == 128 {
        extern "C" {
            fn PL_PS_07();
        }

        PL_PS_07();
    } else {
        // TODO extend the vector table
        cortex_r::disable_irq();

        DefaultHandler()
    }

    // cortex_r::disable_irq(); /* done in asm.s */
    // ICC::set_icceoir(icciar); /* done in asm.s */
}
