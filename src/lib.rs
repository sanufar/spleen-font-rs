#![no_std]

pub mod psf;

pub enum Size {
    S5x8,
    S6x12,
    S8x16,
    S12x24,
    S16x32,
    S32x64,
}

pub static FONTS: &[(&[u8], Size)] = &[
    (include_bytes!("../fonts/spleen-5x8.psfu"), Size::S5x8),
    (include_bytes!("../fonts/spleen-6x12.psfu"), Size::S6x12),
    (include_bytes!("../fonts/spleen-8x16.psfu"), Size::S8x16),
    (include_bytes!("../fonts/spleen-12x24.psfu"), Size::S12x24),
    (include_bytes!("../fonts/spleen-16x32.psfu"), Size::S16x32),
    (include_bytes!("../fonts/spleen-32x64.psfu"), Size::S32x64),
];
