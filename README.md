# `spleen-font`  — tiny `no_std` PSF-2 bitmap‐font reader

This crate ships the six **Spleen** fixed-width fonts and gives you a
panic-free, heap-free API to render them on any framebuffer.

```text
              glyph lookup                 row iterator                     bit iterator
            ┌──────────────┐   ┌────────────────┐   ┌────────────────┐
UTF-8 ──► `PSF2Font` ──────► `Glyph<'_>` ───────►    `GlyphRow<'_>`    ───────►       bool
            │          (cache)      │   │            one scan-line  │   │       one pixel (fg/bg)   │
            └──────────────┘   └────────────────┘   └────────────────┘
```

## Features

* **`no_std`**, zero allocations – suitable for kernels and bootloaders.
* Constant-time glyph lookup for ASCII and cached Unicode.
* < 2 KiB RAM: 64-entry ring cache + iterator state.
* Exposes only data; pixel drawing/framebuffer manipulation is left to the user.

## Quick start

This example assumes that you have a framebuffer and a function to set pixels.

```rust
use spleen_font::*;
fn set_pixel(_: &mut [u8], _: usize, _: usize, _: bool) {}

// Pick a bundled font (8×16 is a good default) and parse it once.
let (blob, _) = FONTS[2];                 // 8×16
let mut font  = PSF2Font::new(blob).unwrap();

// Look up a glyph (cached) and blit it.
if let Some(glyph) = font.glyph_for_utf8("Å".as_bytes()) {
    for (row_y, row) in glyph.enumerate() {
        for (col_x, on) in row.enumerate() {
            set_pixel(framebuffer, col_x, row_y, on);
        }
    }
}
```

## Re-exports

* [`PSF2Font`] — loader + glyph/Unicode lookup.
* [`Glyph`] / [`GlyphRow`] — iterators over rows and pixels.

## Bundled fonts

| variant | size (px) | index in [`FONTS`] |
|---------|-----------|--------------------|
| `S5x8`  |  5 × 8 | 0 |
| `S6x12` |  6 × 12| 1 |
| `S8x16` |  8 × 16| 2 |
| `S12x24`| 12 × 24| 3 |
| `S16x32`| 16 × 32| 4 |
| `S32x64`| 32 × 64| 5 |

Each entry is a tuple **`(&[u8], Size)`** where the slice is the raw PSF-2
file embedded via `include_bytes!`.

---

*Spleen font Copyright (c) 2018-2024, Frederic Cambus, BSD2 License*

**.psfu files obtained from https://github.com/fcambus/spleen*

## Contributions

Contributions are welcome! Please open an issue or submit a pull request.
