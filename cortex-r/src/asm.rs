use core::arch::global_asm;

global_asm!(".global __cpsid", "__cpsid:", "cpsid i", "bx lr",);
global_asm!(".global __cpsie", "__cpsie:", "cpsie i", "bx lr",);

// pub fn nop() {
//     unsafe { llvm_asm!("NOP" : : : : "volatile") }
// }

// pub fn wfi() {
//     unsafe { llvm_asm!("WFI" : : : : "volatile") }
// }
