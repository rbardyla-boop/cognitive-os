//! Deterministic FNV-1a (64-bit) hashing over length-prefixed canonical byte
//! feeds. No clock, no entropy, no floating point — every hash is a pure
//! function of the ordered bytes fed in, so a curation run replays bit-exact.
//!
//! Each chunk is length-prefixed before its bytes so that `("ab", "c")` and
//! `("a", "bc")` cannot collide.

const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

/// A small accumulator that folds length-prefixed chunks into an FNV-1a digest.
pub struct Fnv1a {
    state: u64,
}

impl Fnv1a {
    pub fn new() -> Self {
        Self { state: FNV_OFFSET }
    }

    fn write_byte(&mut self, b: u8) {
        self.state ^= b as u64;
        self.state = self.state.wrapping_mul(FNV_PRIME);
    }

    fn write_u64(&mut self, v: u64) {
        for b in v.to_le_bytes() {
            self.write_byte(b);
        }
    }

    /// Feed a length-prefixed byte chunk.
    pub fn feed_bytes(&mut self, bytes: &[u8]) {
        self.write_u64(bytes.len() as u64);
        for &b in bytes {
            self.write_byte(b);
        }
    }

    /// Feed a length-prefixed string.
    pub fn feed_str(&mut self, s: &str) {
        self.feed_bytes(s.as_bytes());
    }

    /// Feed a raw integer (fixed 8-byte width; not length-prefixed).
    pub fn feed_u64(&mut self, v: u64) {
        self.write_u64(v);
    }

    pub fn finish(&self) -> u64 {
        self.state
    }
}

impl Default for Fnv1a {
    fn default() -> Self {
        Self::new()
    }
}

/// Stable 16-hex-digit digest of a single content string.
pub fn content_hash(content: &str) -> String {
    let mut h = Fnv1a::new();
    h.feed_str(content);
    format!("{:016x}", h.finish())
}
