use core::{fmt, marker::PhantomData, ops::Deref};

pub mod icc;
pub mod icd;

/// GIC Distributor registers
///
/// **IMPORTANT**: *Shared* between CPUs
pub struct ICD {
    _0: PhantomData<icd::RegistersPtr<'static>>,
}

impl ICD {
    pub unsafe fn steal() -> icd::RegistersPtr<'static> {
        unsafe { icd::RegistersPtr::from_ptr(icd::BASE_ADDRESS as *mut _) }
    }

    pub fn disable() {
        unsafe { Self::steal().ICDDCR().write(0) }
    }

    pub fn enable() {
        unsafe { Self::steal().ICDDCR().write(1) }
    }

    pub fn unmask(n: u16) {
        let this = unsafe { Self::steal() };
        let reg = this.ICDISER().idx(usize::from(n) / 32);
        let mut val = reg.read();
        val |= 1 << (n % 32);
        reg.write(val);
    }

    pub fn mask(n: u16) {
        let this = unsafe { Self::steal() };
        let reg = this.ICDISER().idx(usize::from(n) / 32);
        let mut val = reg.read();
        val &= !(1 << (n % 32));
        reg.write(val);
    }

    pub fn route(n: u16, core: u8) {
        let this = unsafe { Self::steal() };
        this.ICDIPTR_rw().idx(usize::from(n) - 32).write(core);
    }

    pub fn set_sensitivity(n: u16, edge: bool) {
        let this = unsafe { Self::steal() };
        let idx = usize::from(n) * 2 / 32;
        let reg = this.ICDICFR().idx(idx);
        let mut val = reg.read();
        if edge {
            val |= 1 << (n * 2 % 32 + 1);
        } else {
            val &= !(1 << (n * 2 % 32 + 1));
        }
        reg.write(val);
    }

    pub fn pend(n: u16) {
        unsafe {
            Self::steal()
                .ICDISPR()
                .idx(usize::from(n) / 32)
                .write(1 << (n % 32));
        }
    }

    pub fn icdsgir(target: Target, id: u8) {
        unsafe {
            let sgiintid = u32::from(id & 0b1111);

            let filter;
            let mut cpulist = 0;
            match target {
                Target::Loopback => filter = 0b10,
                Target::Broadcast => filter = 0b01,
                Target::Unicast(cpu) => {
                    filter = 0b00;
                    cpulist = 1 << (cpu & 0b111)
                }
            }

            // NOTE SATT = 0 sets the pending bit; SATT = 1 doesn't
            #[allow(clippy::identity_op)]
            Self::steal().ICDSGIR().write(
                (filter << 24) /* TargetListFilter */ |
                (cpulist << 16) |
                (0 << 15) /* SATT */ |
                sgiintid,
            );
        }
    }

    pub unsafe fn set_priority(i: u16, priority: u8) {
        Self::steal().ICDIPR().idx(usize::from(i)).write(priority)
    }
}

pub enum Target {
    // Anycast(u8),
    Broadcast,
    Loopback,
    Unicast(u8),
}

unsafe impl Send for ICD {}

/// GIC CPU Interface registers
///
/// **IMPORTANT** One instance per CPU; all instances have the same base address
pub struct ICC {
    _0: PhantomData<icc::RegistersPtr<'static>>,
}

impl ICC {
    pub unsafe fn steal() -> icc::RegistersPtr<'static> {
        unsafe { icc::RegistersPtr::from_ptr(icc::BASE_ADDRESS as *mut _) }
    }

    pub fn disable() {
        unsafe { Self::steal().ICCICR().write(0) }
    }

    pub fn enable() {
        unsafe {
            Self::steal().ICCICR().write(
                (1 << 1) // EnableNS
                | (1 << 0), // EnableS
            )
        }
    }

    pub fn get_icciar() -> ICCIAR {
        unsafe {
            ICCIAR {
                bits: Self::steal().ICCIAR().read(),
            }
        }
    }

    pub fn get_iccpmr() -> u8 {
        unsafe { Self::steal().ICCPMR().read() as u8 }
    }

    pub fn set_icceoir(icciar: ICCIAR) {
        unsafe { Self::steal().ICCEOIR().write(icciar.bits) }
    }

    // pub unsafe fn set_iccpmr(threshold: u8) {
    //     llvm_asm!("" : : : "memory" : "volatile");
    //     Self::steal().ICCPMR.write(u32::from(threshold));
    //     llvm_asm!("" : : : "memory" : "volatile");
    // }

    pub fn set_priority_mask(mask: u32) {
        unsafe { Self::steal().ICCPMR().write(mask) }
    }

    pub unsafe fn set_iccicr(x: u32) {
        Self::steal().ICCICR().write(x);
    }
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct ICCIAR {
    bits: u32,
}

impl fmt::Debug for ICCIAR {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ICCIAR")
            .field("cpuid", &self.cpuid())
            .field("ackintid", &self.ackintid())
            .finish()
    }
}

impl ICCIAR {
    pub fn bits(&self) -> u32 {
        self.bits
    }

    pub fn ackintid(&self) -> u16 {
        (self.bits & ((1 << 10) - 1)) as u16
    }

    pub fn cpuid(&self) -> u8 {
        ((self.bits >> 10) & 0b111) as u8
    }
}

unsafe impl Send for ICC {}

impl Deref for ICC {
    type Target = icc::Registers;

    fn deref(&self) -> &icc::Registers {
        unsafe { &*(icc::BASE_ADDRESS as *const icc::Registers) }
    }
}
