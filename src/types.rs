use reg_map::{access::ReadWrite, RegMap};
type Reg = reg_map::Reg<'static, u64, ReadWrite>;

#[repr(C)]
#[derive(RegMap)]
pub struct DataMap {
    inner: [u64; 4096], // 32 kiB
}
pub type Data = reg_map::RegArray<'static, Reg, 4096>;

#[repr(C)]
#[derive(RegMap)]
pub struct ParamsMap {
    inner: [u64; 20], // 60 B
}
pub type Params = reg_map::RegArray<'static, Reg, 20>;

#[repr(C)]
#[derive(RegMap)]
pub struct BiasDacMap {
    inner: u64,
}
pub type BiasDac = Reg;
