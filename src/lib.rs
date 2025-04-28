//! # `spleen-font`  — tiny `no_std` PSF-2 bitmap‐font reader
//!
//! This crate ships the six **Spleen** fixed-width fonts and gives you a
//! panic-free, heap-free API to render them on any framebuffer.
//!
//! ```text
//!               glyph lookup       row iterator        bit iterator
//!             ┌──────────────┐   ┌────────────────┐   ┌──────────────────┐
//! UTF-8 ──► `PSF2Font` ──────► `Glyph<'_>` ───────► `GlyphRow<'_>` ─────► bool
//!             │ (cache)      │   │ one scan-line  │   │ one pixel (fg/bg)│
//!             └──────────────┘   └────────────────┘   └──────────────────┘
//! ```
//!
//! ## Features
//!
//! * **`no_std`**, zero allocations – suitable for kernels and bootloaders.
//! * Constant-time glyph lookup for ASCII and cached Unicode.
//! * < 2 KiB RAM: 64-entry ring cache + iterator state.
//! * Exposes only data; pixel drawing/framebuffer manipulation is left to the user.
//!
//! ## Quick start
//!
//! This example assumes that you have a framebuffer and a function to set pixels.
//!
//! ```ignore
//! # use spleen_font::*;
//! # fn set_pixel(_: &mut [u8], _: usize, _: usize, _: bool) {}
//!
//! // Pick a bundled font (8×16 is a good default) and parse it once.
//! let (blob, _) = FONTS[2];                 // 8×16
//! let mut font  = PSF2Font::new(blob).unwrap();
//!
//! // Look up a glyph (cached) and blit it.
//! if let Some(glyph) = font.glyph_for_utf8("Å".as_bytes()) {
//!     for (row_y, row) in glyph.enumerate() {
//!         for (col_x, on) in row.enumerate() {
//!             set_pixel(framebuffer, col_x, row_y, on);
//!         }
//!     }
//! }
//! ```
//!
//! ## Re-exports
//!
//! * [`PSF2Font`] — loader + glyph/Unicode lookup.
//! * [`Glyph`] / [`GlyphRow`] — iterators over rows and pixels.
//!
//! ## Bundled fonts
//!
//! | variant | size (px) | index in [`FONTS`] |
//! |---------|-----------|--------------------|
//! | `S5x8`  |  5 × 8 | 0 |
//! | `S6x12` |  6 × 12| 1 |
//! | `S8x16` |  8 × 16| 2 |
//! | `S12x24`| 12 × 24| 3 |
//! | `S16x32`| 16 × 32| 4 |
//! | `S32x64`| 32 × 64| 5 |
//!
//! Each entry is a tuple **`(&[u8], Size)`** where the slice is the raw PSF-2
//! file embedded via `include_bytes!`.
//!
//! ---
//!
//! *Spleen font Copyright (c) 2018-2024, Frederic Cambus, BSD2 License*
//!
//! **.psfu files obtained from https://github.com/fcambus/spleen*

#![no_std]

mod cache;
pub mod glyph;
pub mod psf;

pub use glyph::{Glyph, GlyphRow};
pub use psf::{PSF2Font, PSF2Header, PSF2_MAGIC};

/// Logical name for each embedded Spleen size.
pub enum Size {
    S5x8,
    S6x12,
    S8x16,
    S12x24,
    S16x32,
    S32x64,
}

#[cfg(feature = "s5x8")]
pub const FONT_5X8: &[u8] = include_bytes!("../fonts/spleen-5x8.psfu");
#[cfg(feature = "s6x12")]
pub const FONT_6X12: &[u8] = include_bytes!("../fonts/spleen-6x12.psfu");
#[cfg(feature = "s8x16")]
pub const FONT_8X16: &[u8] = include_bytes!("../fonts/spleen-8x16.psfu");
#[cfg(feature = "s12x24")]
pub const FONT_12X24: &[u8] = include_bytes!("../fonts/spleen-12x24.psfu");
#[cfg(feature = "s16x32")]
pub const FONT_16X32: &[u8] = include_bytes!("../fonts/spleen-16x32.psfu");
#[cfg(feature = "s32x64")]
pub const FONT_32X64: &[u8] = include_bytes!("../fonts/spleen-32x64.psfu");

#[cfg(feature = "all")]
pub static FONTS: &[(&[u8], Size)] = &[
    (include_bytes!("../fonts/spleen-5x8.psfu"), Size::S5x8),
    (include_bytes!("../fonts/spleen-6x12.psfu"), Size::S6x12),
    (include_bytes!("../fonts/spleen-8x16.psfu"), Size::S8x16),
    (include_bytes!("../fonts/spleen-12x24.psfu"), Size::S12x24),
    (include_bytes!("../fonts/spleen-16x32.psfu"), Size::S16x32),
    (include_bytes!("../fonts/spleen-32x64.psfu"), Size::S32x64),
];

#[cfg(test)]
pub static FONTS: &[(&[u8], Size)] = &[
    (include_bytes!("../fonts/spleen-5x8.psfu"), Size::S5x8),
    (include_bytes!("../fonts/spleen-6x12.psfu"), Size::S6x12),
    (include_bytes!("../fonts/spleen-8x16.psfu"), Size::S8x16),
    (include_bytes!("../fonts/spleen-12x24.psfu"), Size::S12x24),
    (include_bytes!("../fonts/spleen-16x32.psfu"), Size::S16x32),
    (include_bytes!("../fonts/spleen-32x64.psfu"), Size::S32x64),
];

#[cfg(test)]
extern crate std;

#[cfg(test)]
use std::vec::Vec;

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn header_round_trip() {
        let (blob, _) = FONTS[1]; // 8×16 face
        let hdr = PSF2Header::from_bytes(&blob[..32]).unwrap();

        assert_eq!(hdr.magic, PSF2_MAGIC);
        assert_eq!(hdr.version, 0);
        assert_eq!(hdr.header_size, 32);
        assert!(hdr.width > 0 && hdr.height > 0);
        assert_eq!(hdr.bytes_per_glyph, ((hdr.width + 7) >> 3) * hdr.height);
    }

    #[test]
    fn header_rejects_invalid_magic() {
        let mut bad = FONTS[0].0.to_vec();
        bad[0] = 0; // corrupt magic
        assert!(PSF2Header::from_bytes(&bad).is_err());
    }

    #[test]
    fn open_font_and_first_last_glyph() {
        let (blob, _) = FONTS[1]; // 6×12
        let font = PSF2Font::new(blob).unwrap();

        // First glyph should be 0 and not empty.
        assert!(!font.glyph_by_idx(0).unwrap().is_empty());

        // Last glyph index = num_glyphs - 1
        let last = font.glyph_by_idx(font.num_glyphs - 1).unwrap();
        assert_eq!(last.len(), font.bytes_per_glyph as usize);

        // Out-of-range must be None (overflow-safe)
        assert!(font.glyph_by_idx(font.num_glyphs).is_none());
    }

    // ASCII fast path and cache hit
    #[test]
    fn ascii_fast_then_cached() {
        let (blob, _) = FONTS[2]; // 8×16
        let mut font = PSF2Font::new(blob).unwrap();

        // first call -> cold   (fills cache)
        let g1 = font.get_glyph_data(&[b'A']).unwrap();
        // second call -> cache hit
        let g2 = font.get_glyph_data(&[b'A']).unwrap();
        assert!(core::ptr::eq(g1.as_ptr(), g2.as_ptr()));
    }

    // Unicode lookup (é)  & iterator correctness
    #[test]
    fn unicode_lookup_and_iter() {
        let (blob, _) = FONTS[0]; // 5x8
        let mut font = PSF2Font::new(blob).unwrap();
        let mut tmp = [0; 2];
        let utf8 = 'é'.encode_utf8(&mut tmp);

        let glyph = font.glyph_for_utf8(utf8.as_bytes()).expect("é present");

        // Glyph MUST yield exactly height rows
        assert_eq!(glyph.len(), font.height as usize);

        // Every row must yield exactly width bits
        for row in glyph.clone() {
            assert_eq!(row.len(), font.width as usize);
        }

        // Forward → collect() then reverse → collect_rev() must match
        let forward: Vec<bool> = glyph.clone().flat_map(|r| r).collect();
        let reverse: Vec<bool> = glyph.rev().flat_map(|r| r).collect();
        let mut rev2 = reverse.clone();
        rev2.reverse();
        assert_eq!(forward, rev2);
    }
}
