//! Parameters shared by reduction and table generation.
pub(super) const OFFSET: u64 = 0x3fe6000000000000;
pub(super) const TABLE_BITS: u32 = 7;
pub(super) const TABLE_SIZE: usize = 1 << TABLE_BITS;
pub(super) const INDEX_SHIFT: u64 = (f64::MANTISSA_DIGITS - 1 - TABLE_BITS) as u64;
