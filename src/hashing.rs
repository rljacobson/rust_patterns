/*!

# Hiding the Backing Implementation of Algorithms and Data Structures

Split up your hashing algorithm and data structure use cases in your own module so you can switch out the backing
implementations easily.

Whenever you have reasons to use different hash functions and alternative hash set / hash
map implementations, define them in their own module, and always draw the type from your
module in the rest of the codebase. Slicing up your use cases in your own module makes it
trivial to switch out the backing implementations for those specific use cases.

This is one of the most boring, bog-standard software engineering practices that exist,
but for some reason it's amazingly common to ignore it for hashing algorithms and related
data structures. Other categories that somehow escape this standard treatment include:

- thread pools and executors
- time and scheduling
- spans, source code locations (in parsers)
- serialization and deserialization
- interning, e.g. for strings (`string_cache`, `ustr`)
- numeric libraries
- `smallvec`, `tiny_vec`, etc.

We are surprisingly happy to spew the specific backing implementation details across
the codebase for these use cases. On the other hand, we are curiously really good at
hiding backing implementation details from the rest of the codebase for other things like:

- logging and tracing (thanks to the `log` crate)
- random number generation (thanks to the `rand` crate)

Error types could arguably be put on both lists. Error handling is really difficult in any language and at its best
rises to an art form. Others have written about it better than I ever could:

- [Designing error types in Rust — Matthieu
M.](https://mmapped.blog/posts/12-rust-error-handling.html) Discusses
the distinction between library and application error design, and the
tradeoffs between defining specific enums versus using catch-all wrappers.
- [Error type design — Rust Error Handling Project Guide
  (Niko Matsakis et al.).](https://nrc.github.io/error-docs/error-design/error-type-design.html)
  Explores different error-type strategies in Rust, including concrete enums,
  single-struct errors, and trait-object approaches, and when each is appropriate.
- [Structuring and handling errors in 2020 — Nick Groenen.](https://nick.groenen.me/posts/rust-error-handling/)
  Examines modern idioms using crates like `anyhow` and `thiserror`, and how design goals differ for applications
  versus reusable libraries.

*/

use std::hash::{Hash, Hasher};

use twox_hash::XxHash3_128;

pub struct Xxh3Hasher128(XxHash3_128);

impl Default for Xxh3Hasher128 {
    fn default() -> Self {
        Self(XxHash3_128::new())
        // or Xxh3::with_seed(seed) for domain separation
    }
}

impl Hasher for Xxh3Hasher128 {
    fn write(&mut self, bytes: &[u8]) {
        self.0.write(bytes); // stream bytes, no allocation
    }
    // Hasher requires a u64 result; return the 64-bit XXH3 if you want,
    // or the low 64 bits of the 128-bit digest.
    fn finish(&self) -> u64 {
        // digest* usually consumes; clone the small state to compute without mutating
        self.0.finish_128() as u64
    }
}

impl Xxh3Hasher128 {
    pub fn finish_u128(self) -> u128 {
        // consume the state to produce the 128-bit digest
        self.0.finish_128()
    }
}

// Helper for any T: Hash
pub fn one_shot_128<T: Hash>(value: &T) -> u128 {
    let mut h = Xxh3Hasher128::default();
    value.hash(&mut h);
    h.finish_u128()
}

// Helper for any T: Hash
pub fn one_shot_64<T: Hash>(value: &T) -> u64 {
    let mut h = Xxh3Hasher128::default();
    value.hash(&mut h);
    h.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hashes_strings() {
        let a = one_shot_128(&"hello");
        let b = one_shot_128(&"hello");
        let c = one_shot_128(&"world");
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn hashes_structs() {
        #[derive(Hash)]
        struct S {
            x: u32,
            y: String,
        }
        let h1 = one_shot_128(&S {
            x: 1,
            y: "a".into(),
        });
        let h2 = one_shot_128(&S {
            x: 1,
            y: "a".into(),
        });
        assert_eq!(h1, h2);
    }
}
