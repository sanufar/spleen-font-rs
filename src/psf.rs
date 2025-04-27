/// Spleen is a PSF2 font, so we are using the PSF2 magic number.
/// We need to parse out the header information first (32 bytes)
/// Since Spleen files have a Unicode table, we will need to allocate ~128KB for lookups.
/// Approach:
/// 1. Parse out header information.
/// 2. Store glyph information/table
///     - Data starts at header_size offset.
///     - Total size should be num_glyphs * bytes_per_glyph.
/// 3. Store Unicode information/offsets
///     - Data starts at header_size + num_glyphs * bytes_per_glyph offset.
///     - Unicode characters are mapped to glyph indices; each "line" ends in 0xFF.

/// The magic number for PSF2 fonts is stored from LSB to MSB
pub const PSF2_MAGIC: [u8; 4] = [0x72, 0xb5, 0x4a, 0x86];

#[derive(Debug, Clone, Copy)]
pub struct PSF2Header {
    /// The magic number for PSF2; see above
    pub magic: [u8; 4],
    /// Version should be 0.
    pub version: u32,
    /// Offset of the bitmaps in file. Header size should always be 32.
    pub header_size: u32,
    /// 0 indicates if there is a unicode table. Else, 1.
    pub flags: u32,
    /// Number of glyphs in the font.
    pub num_glyphs: u32,
    /// Size of each glyph in bytes.
    pub bytes_per_glyph: u32,
    /// Height of each glyph in pixels.
    pub height: u32,
    /// Width of each glyph in pixels.
    pub width: u32,
}

pub struct PSF2Font<'a> {
    /// Height of each glyph in pixels.
    pub height: u32,
    /// Width of each glyph in pixels.
    pub width: u32,
    /// Size of each glyph in bytes.
    pub bytes_per_glyph: u32,
    /// Number of glyphs in the font.
    pub num_glyphs: u32,
    /// Glyph bitmap data.
    glyphs: &'a [u8],
    /// Indices of Unicode characters mapped to glyph data.
    unicode_mapping: &'a [u8],
}

impl PSF2Font<'_> {}

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

    /// Tries to parse a PSF2 header from a byte slice; returns an error if the header is invalid.
    /// Each field is parsed as a little-endian u32, propagates errors if the parsing fails.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, &'static str> {
        if bytes.len() < 32 {
            return Err("PSF2 header is too short");
        }

        fn le_u32(chunk: &[u8]) -> Result<u32, &'static str> {
            let arr: [u8; 4] = chunk
                .try_into()
                .map_err(|_| "failed to read little-endian u32")?;
            Ok(u32::from_le_bytes(arr))
        }

        let magic: [u8; 4] = bytes[0..4]
            .try_into()
            .map_err(|_| "failed to read magic number")?;

        let version = le_u32(&bytes[4..8])?;
        let header_size = le_u32(&bytes[8..12])?;
        let flags = le_u32(&bytes[12..16])?;
        let num_glyphs = le_u32(&bytes[16..20])?;
        let bytes_per_glyph = le_u32(&bytes[20..24])?;
        let height = le_u32(&bytes[24..28])?;
        let width = le_u32(&bytes[28..32])?;

        Ok(PSF2Header::new(
            magic,
            version,
            header_size,
            flags,
            num_glyphs,
            bytes_per_glyph,
            height,
            width,
        ))
    }
}
