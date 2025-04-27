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
use crate::cache::Cache;

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

#[derive(Clone, Copy)]
pub struct Glyph<'a> {
    /// Raw bytes for the glyph. Taken from the glyph bitmap data.
    pub(crate) data: &'a [u8],
    /// Number of columns (pixels). Taken from the PSF2 header.
    pub(crate) width: usize,
}

pub struct PSF2Font<'a> {
    /// Height of each glyph in pixels.
    pub height: u32,
    /// Width of each glyph in pixels.
    pub width: u32,
    /// Size of PSF2 header in bytes.
    pub header_size: u32,
    /// Size of each glyph in bytes.
    pub bytes_per_glyph: u32,
    /// Number of glyphs in the font.
    pub num_glyphs: u32,
    /// Glyph bitmap data.
    glyphs: &'a [u8],
    /// Indices of Unicode characters mapped to glyph data.
    unicode_mapping: &'a [u8],
    /// Cache for glyph indices.
    cache: Cache,
}

impl<'a> PSF2Font<'a> {
    /// Creates a new PSF2 font from a byte slice.
    /// Parses out header and validates, populates glyph and unicode mapping data.
    pub fn new(data: &'a [u8]) -> Result<Self, &'static str> {
        let header = PSF2Header::from_bytes(data)?;

        // Calculate offsets and ensure data is valid
        let glyphs_offset = header.header_size as usize;
        let glyphs_size = header.num_glyphs as usize * header.bytes_per_glyph as usize;
        let unicode_offset = glyphs_offset + glyphs_size;

        if data.len() < unicode_offset {
            return Err("PSF2 data too short");
        }

        // Extract glyph data and unicode mapping
        let glyphs = &data[glyphs_offset..unicode_offset];
        let unicode_mapping = &data[unicode_offset..];

        Ok(Self {
            height: header.height,
            width: header.width,
            header_size: header.header_size,
            bytes_per_glyph: header.bytes_per_glyph,
            num_glyphs: header.num_glyphs,
            glyphs,
            unicode_mapping,
            cache: Cache::new(),
        })
    }

    /// Returns glyph data for a given UTF-8 byte slice.
    /// Goes through three paths:
    /// 1. If the text is a single ASCII character:
    ///    we simply return the glyph index as mapped to the UTF-8 index.
    /// 2. If the we get a cache hit for our sequence, we return the cached glyph data.
    /// 3. If all else fails, we do a linear search through our unicode mapping table.
    pub fn get_glyph_data(&mut self, text: &[u8]) -> Option<&'a [u8]> {
        if text.len() == 1 && text[0] <= 0x7F {
            return self.glyph_by_idx(text[0] as u32);
        }

        if let Some(idx) = self.cache.get(text) {
            return self.glyph_by_idx(idx);
        }

        if let Some(idx) = self.scan_unicode_table(self.unicode_mapping, text) {
            self.cache.insert(text, idx);
            return self.glyph_by_idx(idx);
        }

        None
    }

    /// Scans the unicode mapping table for a given sequence of bytes.
    /// Returns the glyph index if found, otherwise None.
    /// Does perform a O(n) search through the table. We add a cache to make this less expensive.
    fn scan_unicode_table(&mut self, table: &[u8], sequence: &[u8]) -> Option<u32> {
        let mut glyph_idx: u32 = 0;
        let mut p: usize = 0;

        const START_SEQ: u8 = 0xFE;
        const END_REC: u8 = 0xFF; // Note that the end of **table** marker is 0xFF 0xFF.

        while p < table.len() {
            if table.get(p..p + 2) == Some(&[END_REC, END_REC]) {
                return None;
            }

            loop {
                match table[p] {
                    START_SEQ => p += 1,
                    END_REC => {
                        glyph_idx += 1;
                        p += 1;
                        break;
                    }
                    b => {
                        let start = p;
                        p += Self::next_utf8_len(b)?;
                        while p < table.len() && !matches!(table[p], START_SEQ | END_REC) {
                            p += Self::next_utf8_len(table[p])?;
                        }

                        if &table[start..p] == sequence {
                            return Some(glyph_idx);
                        }
                    }
                }
            }
        }
        None
    }

    /// Decode exactly one valid UTF-8 scalar and return (len, first_byte_masked)
    /// Returns None on malformed UTF-8 or truncated input.
    ///
    /// Every UTF-8 sequence starts with a leading byte that indicates the number of bytes in the sequence.
    /// The leading byte is followed by continuation bytes that each start with the bits 10xxxxxx.
    ///
    /// one byte:       0.......
    /// two bytes:      110..... 10......
    /// three bytes:    1110.... 10...... 10......
    /// four bytes:     11110... 10...... 10...... 10......
    #[inline]
    fn next_utf8_len(b: u8) -> Option<usize> {
        Some(match b {
            0x00..=0x7F => 1, // 0xxxxxxx
            0xC2..=0xDF => 2, // 110xxxxx
            0xE0..=0xEF => 3, // 1110xxxx
            0xF0..=0xF4 => 4, // 11110xxx
            _ => return None, // continuation or invalid
        })
    }

    /// Returns glyph data for a given glyph index.
    /// If the index is out of bounds, returns None.
    #[inline]
    fn glyph_by_idx(&self, idx: u32) -> Option<&'a [u8]> {
        if idx < self.num_glyphs {
            let offset = (self.header_size + idx * self.bytes_per_glyph) as usize;
            Some(&self.glyphs[offset..offset + self.bytes_per_glyph as usize])
        } else {
            None
        }
    }
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

        // Magic number must always be PSF2_MAGIC.
        if magic != PSF2_MAGIC {
            return Err("PSF2 magic number is invalid");
        }

        let version = le_u32(&bytes[4..8])?;

        // Version number must always be 0.
        if version != 0 {
            return Err("PSF2 version is not supported");
        }

        // I would check if this is 32, but maybe it'll change.
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
