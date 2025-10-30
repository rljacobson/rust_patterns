# Rust Programming Patterns

This repository is a curated and growing collection of Rust programming patterns, idioms, and
implementation techniques drawn from real-world systems programming, language design, and
library development. I might also include opinionated best practices. See [TODO.md](TODO.md)
for topics about which I have something to say that hasn't made it into this repo yet.

Many of these examples were developed or collected during work on
[ixa](https://github.com/CDCgov/ixa) — an agent-based modeling framework that
makes heavy use of reflection and plugin-like systems, which explains the bias
towards those themes here. Some ideas come from my work on
[mod2](https://github.com/rljacobson/mod2), a Rust implementation of some of the
advanced pattern matching algorithms in Maude.

Some of the ideas represented here are idiomatic Rust rediscoveries of much
older techniques from systems programming, functional programming, and type
theory (for example, patterns inspired by Haskell’s type-level design or
ML-style module systems). While these patterns are well-known across programming
language communities, their practical and idiomatic realization in Rust is of
particular interest here.

**Goals:**

- illustration of ideas, not (necessarily) production quality implementation —
  adapt to your use case or use an existing crate

- zero-cost abstractions
- general applicability
- Rust-specific context

**Major Themes**:

- Type-indexed data and reflection
- Registries and plugin systems
- Hashing and map design
- Type-erased vs. strongly typed APIs
- Strategies for initialization / separation of mutable and immutable accesses.
- Memory ownership and life cycle management

**Similar Efforts:**

Anyone interested in this repo will almost certainly want to study
[Rust Design Patterns (rust-unofficial)](https://rust-unofficial.github.io/patterns/),
a documented catalogue of idiomatic Rust patterns, anti-patterns, and community
conventions. Includes behavioral, structural, and creational patterns.

## How to Read

The idea is to read the docs like a book. The docs are rendered on GitHub Pages:
<https://www.robertjacobson.dev/rust_patterns/rust_patterns/>

Commentary is provided in doc comments and in the source code itself. Sometimes
there isn't a doc comment that is a good fit for something I have to say, so
I'll but information in a module-level doc comment. There isn't a perfect
solution. Hopefully it's still useful.

## Authorship, Attribution, and License

Copyright © 2025 Robert Jacobson. This software is distributed under the terms
of the [MIT license](LICENSE-MIT) or the [Apache 2.0 license](LICENSE-APACHE) at
your preference.

This collection represents a blend of:

- techniques independently developed or refined during work on Rust projects
  such as [ixa](https://github.com/CDCgov/ixa).
- patterns that are well-established in the wider systems, functional, and
  language-theory communities.
- idiomatic Rust implementations of known ideas — often inspired by patterns
  that have evolved organically within the Rust community itself.

Although the implementations and commentary here are original (to the extent
that even means anything, as some are also _trivial_), the underlying ideas are
shared heritage within decades of programming language development. This
repository is intended in that spirit — as a documentation of understanding,
adaptation, and synthesis, rather than as a claim of invention.
