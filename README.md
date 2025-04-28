# `spleen-font`  — tiny `no_std` PSF-2 bitmap‐font reader

[![tests](https://github.com/sanufar/spleen-font-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/sanufar/spleen-font-rs/actions/workflows/ci.yml)

This crate ships the six **Spleen** fixed-width fonts and gives you a
panic-free, heap-free API to render them on any framebuffer.

```text
              glyph lookup        row iterator         bit iterator
            ┌──────────────┐   ┌────────────────┐   ┌────────────────┐
UTF-8 ──► `PSF2Font` ──────► `Glyph<'_>` ───────►  `GlyphRow<'_>` ────► bool
            │    (cache)   │   |  one scan-line │   |    one pixel   │
            └──────────────┘   └────────────────┘   └────────────────┘
```

## Features

* **`no_std`**, zero allocations – suitable for kernels and bootloaders.
* Constant-time glyph lookup for ASCII and cached Unicode.
* < 2 KiB RAM: 64-entry ring cache + iterator state.
* Exposes only data; pixel drawing/framebuffer manipulation is left to the user.

## Quick start

This example assumes that you have a framebuffer and a function to set pixels, and that you have enabled the `s8x16` feature in your `Cargo.toml`.

```rust
// Pick a bundled font (8×16 is a good default) from the feature list (below)
use spleen_font::{PSF2Font, FONT_8X16};
fn set_pixel(_: &mut [u8], _: usize, _: usize, _: bool) {}

let mut font  = PSF2Font::new(FONT_8X16).unwrap();

// Look up a glyph (cached) and blit it.
if let Some(glyph) = font.glyph_for_utf8("é".as_bytes()) {
    for (row_y, row) in glyph.enumerate() {
        for (col_x, on) in row.enumerate() {
            set_pixel(framebuffer, col_x, row_y, on);
        }
    }
}
```

## Features

To reduce the overall footprint, each of the six fonts is gated behind a feature. Note that while this does not reduce the size of the crate on crates.io, it does reduce the size of the compiled binary - only the blob for the selected font you enable will be included in the final binary.

Feature | Enables constant | Size (KiB)
|---------|-----------|--------------------|
s5x8 | FONT_5X8 | 2 KiB
s6x12 | FONT_6X12 | 6 KiB
s8x16 | FONT_8X16 | 4 KiB
s12x24 | FONT_12X24 | 12 KiB
s16x32 | FONT_16X32 | 32 KiB
s32x64 | FONT_32X64 | 128 KiB
all | all of the above | 184 KiB

Each entry is a raw byte slice **`&[u8]`** where the slice is the raw PSF-2 file embedded via `include_bytes!`.

By default, no font is enabled. Enabling a font in your Cargo.toml should look like this:

```toml
[dependencies]
spleen-font = { version = "0.1", features = ["s8x16"] }
```

## Re-exports

* [`PSF2Font`] — loader + glyph/Unicode lookup.
* [`Glyph`] / [`GlyphRow`] — iterators over rows and pixels.
---

*Spleen font Copyright (c) 2018-2024, Frederic Cambus, BSD2 License*

**.psfu files obtained from https://github.com/fcambus/spleen*

## Contributions

Contributions are welcome! Please open an issue or submit a pull request.
