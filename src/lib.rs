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
//! ```no_run
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
pub use psf::PSF2Font;

/// Logical name for each embedded Spleen size.
pub enum Size {
    S5x8,
    S6x12,
    S8x16,
    S12x24,
    S16x32,
    S32x64,
}

/// Array of `(font_blob, Size)` for the six bundled Spleen variants.
pub static FONTS: &[(&[u8], Size)] = &[
    (include_bytes!("../fonts/spleen-5x8.psfu"), Size::S5x8),
    (include_bytes!("../fonts/spleen-6x12.psfu"), Size::S6x12),
    (include_bytes!("../fonts/spleen-8x16.psfu"), Size::S8x16),
    (include_bytes!("../fonts/spleen-12x24.psfu"), Size::S12x24),
    (include_bytes!("../fonts/spleen-16x32.psfu"), Size::S16x32),
    (include_bytes!("../fonts/spleen-32x64.psfu"), Size::S32x64),
];
