/// In order to have faster lookup times, we cache glyphs in a simple ring buffer.
/// Note that this cache is the second fallback - we first check if the glyph index falls within ASCII range.

/// Longest possible key we can copy into the cache.
/// This covers every UTF-8 scalar (spleen doesn't come with emojis).
const MAX_KEY_SIZE: usize = 16;

/// Maximum size of the cache.
const CACHE_SIZE: usize = 64;

#[derive(Clone, Copy)]
struct CacheEntry {
    len: u8,
    key: [u8; MAX_KEY_SIZE],
    glyph: u32,
}

/// A simple ring buffer cache for glyphs.
/// Uses a round-robin insertion cursor.
pub struct Cache {
    entries: [CacheEntry; CACHE_SIZE],
    next: usize,
}

impl Cache {
    /// Creates a new cache.
    pub const fn new() -> Self {
        Cache {
            entries: [CacheEntry {
                len: 0,
                key: [0; MAX_KEY_SIZE],
                glyph: 0,
            }; CACHE_SIZE],
            next: 0,
        }
    }

    /// Tries to get a glyph from the cache.
    pub fn get(&self, key: &[u8]) -> Option<u32> {
        self.entries.iter().find_map(|e| {
            if e.len as usize == key.len() && &e.key[..key.len()] == key {
                Some(e.glyph)
            } else {
                None
            }
        })
    }

    /// Inserts a glyph into the the cache.
    /// Overwrites the next slot if full.
    pub fn insert(&mut self, key: &[u8], glyph: u32) {
        if key.len() > MAX_KEY_SIZE {
            return;
        }

        let entry = &mut self.entries[self.next];
        entry.len = key.len() as u8;
        entry.key[..key.len()].copy_from_slice(key);
        entry.glyph = glyph;
        self.next = (self.next + 1) % CACHE_SIZE;
    }
}
