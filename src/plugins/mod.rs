/*!

# Plugin Systems in Rust

Writing a plugin system in Rust usually involves two main components:

1. A mechanism for providing a generic interface between the main program and the plugin.
2. A mechanism for plugin discovery, which you might call "initialization"
  or "registration", but which is more about knowing which plugins exist,
  possibly across crate boundaries, without explicit central declaration.

The first is well-trodden ground in Rust. You typically use a trait to define
the generic interface. (See [crate::type_erased_api].)

For the second, the tricky part is avoiding having to know beforehand what plugins exist
and what their dependencies are. The phrase "plugin" as we are using it here implies some
degree of automatic discovery of these things. (Otherwise, they're just libraries or modules
or, if you want to be maximally generic, *dependencies*.) We are especially interested
the situation where you have a library that can use plugins defined in external crates.

While some kind of automatic initialization is implied, note that the plugin problem itself is a
separate problem from that of initialization (lazy or otherwise), which `OnceCell`, `LazyStatic`,
and other mechanisms address. The issue is *not* that initialization needs to occur at all, but
rather that we need to know which items exist in the first place. Once we know which items exist,
our strategy for initialization, while interesting in its own right, is a separate question.

There are two main ways to do plugins in Rust, plus hybrid approaches.

## 1. “Run Before Main” Mechanisms

These systems rely on functions that are automatically executed before
`main()` is executed. They typically use compiler or linker attributes (`#[ctor]`,
`#[dtor]`, or `#[used]`) to mark static constructors that run at startup.

Example crates: [ctor](https://crates.io/crates/ctor), [init](https://crates.io/crates/init)

This mechanism is easy to understand and enjoys wide platform
support, because it's the same mechanism used by C++ for construction of
statics, but it has severe limitations in Rust applications, including but not limited to:

- unspecified execution order
- [fragility in dynamic linking contexts](https://github.com/rust-lang/rust/issues/28794#issuecomment-368693049)
  (cdylib, plugins, embedded targets).
- [fragility in static linking contexts](https://github.com/mmastrac/rust-ctor#warnings) ("some linker configurations
  may cause `#[ctor]` and `#[dtor]` functions to be stripped from the final binary")
- execution of code prior to program initialization (e.g. using `println!` causes a panic)

## 2. Linker-based "Distributed Slice" Mechanisms

This category uses linker section merging to collect specially annotated static
data items (not code) across crates into a single contiguous slice. (The static
data could be function pointers or static structs with their own methods.)

Example crates: [linkme](https://crates.io/crates/linkme),
[distributed_slice](https://crates.io/crates/distributed_slice).

This mechanism is true automatic cross-crate registration with *zero runtime overhead*, because
it is literally just static data globally accessible by name. As with the `ctor` solution,
order is not specified, which in practice just means that the question of initialization
order is moved to user land at runtime, arguably where it belongs. Unfortunately, this
mechanism (currently) relies on specific linker behavior and is not compatible with certain
build environments and runtime targets. Crucially, *there is currently no support for Wasm.*

## 3. Hybrid Approaches

These approaches add layers of convenience, for example, ways to do lazy initialization and thread safety.

- [inventory](https://crates.io/crates/inventory) — layers initialization on
  top of the "distributed slice" concept for situations where initialization
  requires runtime code execution, not just compile-time statics.
- [static_init](https://crates.io/crates/static_init) — adds sophisticated initialization
  mechanisms that claim to be superior to `lazy_static` and other alternatives for certain use cases.

---

The general consensus is that the "distributed slice" mechanism is the best
way forward for the Rust ecosystem, but compiler support is ultimately
required. The lack of Wasm support is a nonstarter for many applications.

The reason the "distributed slice" mechanism is the "right" thing to do boils down to Rust's
fundamental assumption that nothing executes before or after `main()`. Violating that runtime
assumption will be fragile at best *in any conceivable future*. On the flip side, "run
before main" has widespread platform support inherited from C++, so the "worse" solution is
currently the best solution right now—indeed, the only solution if Wasm support is required.

It's not ideal. Everybody knows this. It's just a hard nut to crack.

*/

pub mod item_registry;
