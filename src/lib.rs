#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"))]

/*!

# Ideas That Do Not Yet Have a Home

## Correctness via Macro

There are many ways to enforce "correctness" in a computer program:

- the type system: data types, method signatures
- ownership model: borrowing, lifetimes, RAII techniques, copy/move semantics
- visibility/access control: packages, modules, namespaces
- exhaustiveness checking
- bounds checking
- etc.

The best situation is when the compiler can enforce correctness with as little engagement
as possible from the programmer; correctness just "happens" without manually having
to be checked and without the programmer having to write a lot of code to ensure it.

But this is not always possible. Consider the case of [`plugin systems`](crate::plugins), which
have some degree of automatic discovery of plugins across crate boundaries. In such a case, the
library author is not in control of the code that defines the plugin. Rust's orphan/coherence rules
prevent the library author from writing implementation on behalf of client code. So how can the
library author ensure that the implementation of a plugin defined in client code is correct? By
providing the implementation in client code through a macro that can realize the implementation in
the context of the client crate. A concrete example is given in the [`registered_item_impl`] macro.

Unfortunately, there is no mechanism to _require_ that client code use the macro to
implement a plugin. Also, Rust macros have exactly the same access and limitations
as any other client code, so the implementation they provide is not equivalent to
other information hiding mechanisms. Finally, the correctness of the implementation
the macro generates is obviously only correct if the macro itself is correct.

*/

pub mod hashing;
pub mod plugins;
pub mod shared_implementation;
pub mod type_erased_api;

// Re-exported for use in exported macros
pub use ctor;
pub use paste;
