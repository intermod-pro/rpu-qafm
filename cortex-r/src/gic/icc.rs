use reg_map::RegMap;

pub const BASE_ADDRESS: usize = super::icd::BASE_ADDRESS + 0x1000;

#[allow(non_snake_case)]
#[repr(C)]
#[derive(RegMap)]
pub struct Registers {
    /// 0x00 - CPU Interface Control Register
    #[reg(RW)]
    pub ICCICR: u32,
    /// 0x04 - Interrupt Priority Mask Register
    #[reg(RW)]
    pub ICCPMR: u32,
    /// 0x08 - Binary Point Register
    #[reg(RW)]
    pub ICCBPR: u32,
    /// 0x0C - Interrupt Acknowledge Register
    #[reg(RO)]
    pub ICCIAR: u32,
    /// 0x10 - End of Interrupt Register
    #[reg(WO)]
    pub ICCEOIR: u32,
    /// 0x14 - Running Priority Register
    #[reg(RO)]
    pub ICCRPR: u32,
    /// 0x18 - Highest Pending Interrupt Register
    #[reg(RO)]
    pub ICCHPIR: u32,
    /// 0x1C - Aliased Binary Point Register
    #[reg(RW)]
    pub ICCABPR: u32,
    /// 0x20..=0x3C - Reserved
    _reserved0: [u32; 8],
    /// 0x40..=0xCF - Implementation Defined Registers
    #[reg(RW)]
    pub ICCIDR: [u32; 36],
}
