/// Each glyph is essentially a 2D bitmap.
///
/// Example: for an 8x16 font, each glyph is 16 bytes long;
///
/// Each byte encodes one row of pixels.
///
/// ```text
/// bytes_per_row = ceil(width / 8)
/// bytes_per_glyph = bytes_per_row * height
///
/// ┌ row 0: bytes_per_row bytes
/// │ row 1: bytes_per_row bytes
/// │ …
/// └ row h-1: bytes_per_row bytes
/// ```
#[derive(Clone, Copy)]
pub struct Glyph<'a> {
    /// Raw bytes for the glyph. Taken from the glyph bitmap data.
    data: &'a [u8],
    /// Number of columns (pixels). Taken from the PSF2 header.
    width: usize,
}

#[derive(Clone, Copy)]
pub struct GlyphRow<'a> {
    row: &'a [u8],  // ((width+7)>>3) bytes
    bit_idx: usize, // current bit
    width: usize,
}

impl<'a> Glyph<'a> {
    pub fn new(slice: &'a [u8], width: usize) -> Self {
        Glyph { data: slice, width }
    }
}

impl<'a> GlyphRow<'a> {
    pub fn new(row: &'a [u8], width: usize) -> Self {
        GlyphRow {
            row,
            bit_idx: 0,
            width,
        }
    }
}

impl<'a> Iterator for Glyph<'a> {
    type Item = GlyphRow<'a>;

    /// Returns the next row of the glyph.
    ///
    /// ## Example:
    /// ```
    /// let glyph = Glyph::new(&[0b11001100, 0b00110011], 8);
    /// let mut rows = glyph.into_iter();
    /// assert_eq!(rows.next(), Some(GlyphRow::new(&[0b11001100], 8)));
    /// assert_eq!(rows.next(), Some(GlyphRow::new(&[0b00110011], 8)));
    /// assert_eq!(rows.next(), None);
    /// ```
    ///
    /// Glyph::next()
    ///
    /// data = [row0 | row1 | row2 | …]  ─►  returns GlyphRow(row0)
    ///
    ///                                       keeps rest for next call
    fn next(&mut self) -> Option<Self::Item> {
        let bytes_per_row = (self.width + 7) >> 3;
        if self.data.len() < bytes_per_row {
            None
        } else {
            let (row, rest) = self.data.split_at(bytes_per_row);
            self.data = rest;
            Some(GlyphRow::new(row, self.width))
        }
    }
}

impl ExactSizeIterator for Glyph<'_> {
    fn len(&self) -> usize {
        self.data.len() / ((self.width + 7) >> 3)
    }
}

impl DoubleEndedIterator for Glyph<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let bytes_per_row = (self.width + 7) >> 3;
        if self.data.len() < bytes_per_row {
            return None;
        }
        let split = self.data.len() - bytes_per_row;
        let (rest, row) = self.data.split_at(split);
        self.data = rest;
        Some(GlyphRow::new(row, self.width))
    }
}

impl<'a> Iterator for GlyphRow<'a> {
    type Item = bool;

    /// Returns the next bit of the row as a bool.
    ///
    /// Consumers can use this bool to draw individual pixels to the framebuffer.
    ///
    /// ## Example usage:
    ///
    /// We assume that we can draw pixels at position (x, y) with an arbitrary set_pixel function.
    ///
    /// ```no_run
    /// for (row_y, row) in glyph.enumerate() {
    ///     for (col_x, on) in row.enumerate() { // The glyph row to iterate over
    ///         set_pixel(fb,
    ///                   Position { x: origin.x + col_x,
    ///                              y: origin.y + row_y },
    ///                   if on { fg } else { bg });
    ///     }
    /// }
    /// ```
    fn next(&mut self) -> Option<Self::Item> {
        if self.bit_idx >= self.width {
            None
        } else {
            // We bump each row width that isn't already a multiple of 8 to the next multiple of 8
            // Then we can divide the rounded value by 8 to get the smallest whole byte count
            // that can hold all `width` bits in a row.
            let byte = self.row[self.bit_idx >> 3];

            // Calculate the mask for the current bit index ; we shift by the bit index modulo 8.
            let mask = 0b10000000 >> (self.bit_idx & 7);

            // If the bit is set, return true; otherwise, return false.
            let bit = byte & mask != 0;

            self.bit_idx += 1;
            Some(bit)
        }
    }
}
