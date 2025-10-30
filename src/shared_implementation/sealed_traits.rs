/*!

# Fine-grained Encapsulation and Information Hiding with Traits

This module demonstrates fine-grained access/overridability control of trait methods as
described by Predrag Gruevski in his excellent article[ *A definitive guide to sealed
traits in Rust*](https://predr.ag/blog/definitive-guide-to-sealed-traits-in-rust/).
We demonstrate each of the following:

|                                                              | all methods callable downstream                              | some methods callable downstream                             | no methods callable downstream   |
| ------------------------------------------------------------ | ------------------------------------------------------------ | ------------------------------------------------------------ | -------------------------------- |
| **all methods overridable**                                  | ✅ (`pub trait`)                                              | ❌                                                            | ❌                                |
| **some methods overridable**                                 | ✅ ([signature-sealed default methods + `pub fn` to call them](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=fcdedb4be8f688e36dc07e16df28c080) / ["final methods" pre-RFC](https://internals.rust-lang.org/t/pre-rfc-final-trait-methods/18407)) | ✅ (partially-sealed / ["final methods" pre-RFC](https://internals.rust-lang.org/t/pre-rfc-final-trait-methods/18407)) | ❌                                |
| **trait cannot be impl'd downstream (no methods overridable)** | ✅ (supertrait sealed)                                        | ✅ (at least one signature-sealed method with no default impl) | ✅ (all methods signature-sealed) |


These techniques enable not just shared interfaces but shared
implementation while maintaining a level of encapsulation.


## Information Hiding and Encapsulation in Rust

Rust's language features enabling information hiding and encapsulation are much simpler and
fewer in number than those in C++. In Rust, items and fields are private by default, and
the visibility modifier `pub` is intentionally coarse-grained, limited to `pub` (without
qualification), `pub(crate)`, `pub(super)`, and `pub(in path)`. There is no mechanism to
grant selective access to private items. Rust has no concept of inheritance. Rust uses traits,
which define a set of method signatures, to define shared interfaces. A trait can have one
or more trait constraints*, which look like superclasses if you squint, but which are really
just assertions that an implementor of the trait must also implement some other trait.

Rust's support for shared *implementation* is very limited. A trait can have a default
implementation for a method, but even this support for shared implementation is hamstrung by the
fact that traits cannot have data members of any visibility. Thus any default implementation
can only assume about `self` what the trait itself asserts and cannot access any data directly.
What's more, as a mechanism for defining public interfaces, trait methods are necessarily public.

However, it turns out that it nonetheless is possible to achieve very fine-grained
access and overridability restrictions on traits using a concept called _sealed traits_.

*/

/// A struct we might want to implement a trait for.
pub struct Person;

pub mod no_restrictions {
    // A super person!
    use super::Person;

    /// In the most boring, typical case, a trait is completely visible, and every method
    /// is implementable and overridable (if there is a default implementation to override).
    pub trait Greet {
        /// Implementors will need to supply their own implementation of this method.
        fn greet(&self) -> String;

        /// This default implementation is visible, callable, and overridable by downstream
        /// implementors.
        fn goodbye(&self) -> String {
            "Goodbye!".into()
        }
    }

    impl Greet for Person {
        /// Can implement a trait method
        fn greet(&self) -> String {
            "Hi from a person!".into()
        }
        /// Can override a trait method
        fn goodbye(&self) -> String {
            "Bye from a person!".into()
        }
    }
}

pub mod all_callable_some_overridable {
    // A super person!
    use super::Person;

    /// The `sealed` module is private to this crate, and only this crate can *implement*
    /// the `Sealed` trait it contains.
    mod sealed {
        use crate::shared_implementation::sealed_traits::Person;

        /// The Seal trait is `pub` but lives inside the private `sealed` module.
        pub trait Sealed {}
        impl Sealed for Person {}
    }

    /// The trait is public, but the super trait is NOT public
    pub trait Greet: sealed::Sealed {
        /// Overridable
        fn greet(&self) -> String {
            "Hello from default".into()
        }

        /// Not overridable (final)
        fn final_greet(&self) -> String {
            format!("(final) {}", Self::final_greet_impl())
        }

        /// Sealed signature — cannot override
        #[doc(hidden)]
        fn final_greet_impl() -> &'static str {
            "Hello world"
        }
    }

    /// The `Sealed` trait is already implemented for `Person`, so we are free to implement `Greet` outselves.
    impl Greet for Person {}
}
