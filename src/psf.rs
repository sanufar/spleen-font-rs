/// Spleen is a PSF2 font, so we are using the PSF2 magic number.

/// The magic number for PSF2 fonts is stored from LSB to MSB
pub const PSF2_MAGIC: [u8; 4] = [0x72, 0xb5, 0x4a, 0x86];

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
struct PSF2Header {
    /// The magic number for PSF2; see above
    magic: [u8; 4],
    /// Version should be 0.
    version: u32,
    /// Offset of the bitmaps in file. Header size should always be 32.
    header_size: u32,
    /// 0 indicates if there is a unicode table. Else, 1.
    flags: u32,
    /// Number of glyphs in the font.
    num_glyphs: u32,
    /// Size of each glyph in bytes.
    bytes_per_glyph: u32,
    /// Height of each glyph in pixels.
    height: u32,
    /// Width of each glyph in pixels.
    width: u32,
}

impl PSF2Header {
    /// Create a new PSF2 header with the given parameters.
    pub fn new(
        magic: [u8; 4],
        version: u32,
        header_size: u32,
        flags: u32,
        num_glyphs: u32,
        bytes_per_glyph: u32,
        height: u32,
        width: u32,
    ) -> Self {
        PSF2Header {
            magic,
            version,
            header_size,
            flags,
            num_glyphs,
            bytes_per_glyph,
            height,
            width,
        }
    }
}
