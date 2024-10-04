// #![feature(asm)]
// #![feature(llvm_asm)]
#![no_std]
#![allow(clippy::missing_safety_doc)]

use core::sync::atomic::{compiler_fence, Ordering};

pub mod asm;
pub mod gic;
pub mod register;

// NOTE(unsafe) can break a critical section
// pub unsafe fn enable_fiq() {
//     match () {
//         #[cfg(target_arch = "arm")]
//         () => llvm_asm!("cpsie f" : : : "memory" : "volatile"),

//         #[cfg(not(target_arch = "arm"))]
//         () => unimplemented!(),
//     }
// }

// pub fn disable_fiq() {
//     match () {
//         #[cfg(target_arch = "arm")]
//         () => unsafe { llvm_asm!("cpsid f" : : : "memory" : "volatile") },

//         #[cfg(not(target_arch = "arm"))]
//         () => unimplemented!(),
//     }
// }

// NOTE(unsafe) can break a critical section
pub unsafe fn enable_irq() {
    match () {
        #[cfg(target_arch = "arm")]
        () => {
            extern "C" {
                fn __cpsie();
            }

            // Ensure no preceeding memory accesses are reordered to after interrupts are enabled.
            compiler_fence(Ordering::SeqCst);

            __cpsie();
        }

        #[cfg(not(target_arch = "arm"))]
        () => unimplemented!(),
    }
}

pub fn disable_irq() {
    match () {
        #[cfg(target_arch = "arm")]
        () => unsafe {
            extern "C" {
                fn __cpsid();
            }

            __cpsid();

            // Ensure no subsequent memory accesses are reordered to before interrupts are disabled.
            compiler_fence(Ordering::SeqCst);
        },

        #[cfg(not(target_arch = "arm"))]
        () => unimplemented!(),
    }
}
