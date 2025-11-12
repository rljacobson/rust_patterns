#![allow(dead_code)]
/*!

# ZST-based interface to static data of a non-ZST type

Let's design a ZST-based interface to the static methods/data of a (possibly) non-ZST type
(in Rust). The idea is that the concrete implementor `M` of some trait exposes its static data
(or methods) through its implementation of a trait, which we'll call [`MyTrait`]. Suppose
we want to interact with that data without having an instance of the concrete type `M`.

Our goal is to create a type the instances of which provide
type-erased proxy access to the static data/methods of interest.

- [`simple_static_vtable`] shows how to do this using a simple static VTable: Type erasure is achieved by storing a function pointer.
- [`static_reference_to_zst`] shows how to do this using a static reference to a ZST: Type erasure is achieved by using a zero-sized trait object behind a reference, which does not allocate.

Both strategies use the same technique of generating static constant data

*/

/// Our example trait implemented by some concrete type `M` provides an interface to static methods or data.
pub trait MyTrait {
    fn get_static_dependency_data() -> &'static MyStaticData;
}

/// The static data we want to expose.
pub struct MyStaticData {
    msg: &'static str,
}

pub mod simple_static_vtable {
    /*!

    ## Using a simple static VTable

    This strategy erases the concrete type `M` by storing a pointer to a function
    in a trait object, and a single static instance for each concrete type `M` is
    stored as an associated constant within a trait having `M` as a generic parameter.

    If you only need to call associated functions (no data), you can skip trait objects entirely
    and pass a pointer to a tiny vtable of function pointers. This gives you a single-word handle.

    The idea is to store the concrete functions as pointers in a struct
    [`StaticMyTraitInterface`]. The [`StaticMyTraitInterface`] struct
    has a static factory method `const fn of<M: MyTrait>() -> Self`.

    An extra set of parentheses is needed to call the function directly:

    ```rust,ignore
    let tag: Tag = tag_for::<ConcreteM>();
    let data = (tag.get_static_dependency_data)();
    ```

    This awkward syntax can be eliminated by hiding it in an appropriately named regular method:

    ```rust,ignore
    let data = tag.get_data();
    ```

    The size of the [`StaticMyTraitInterface`] struct is one machine word per field. If you want
    to be more space efficient, you can have static references to [`StaticMyTraitInterface`]
    values by storing them in a per-`M` holder type [`VTableHolder<M: MyTrait>`]:

    ```rust,ignore
    /// Per-`M` holder that exposes an associated `const` vtable.
    struct VTableHolder<M: MyTrait>(PhantomData<M>);

    impl<M: MyTrait> VTableHolder<M> {
        // Lives in read-only memory; taking `&` yields a `'static` reference.
        const TABLE: StaticMyTraitInterface = StaticMyTraitInterface::of::<M>();
    }

    /// Our erased handle is just a thin pointer.
    type Tag = &'static StaticMyTraitInterface;

    /// Factory: return a `'static` reference to the per-`M` vtable.
    fn tag_for<M: MyTrait>() -> Tag {
        &VTableHolder::<M>::TABLE
    }
    ```

    */

    use std::marker::PhantomData;

    use super::*;

    /// Function VTable describing the static interface we care about.
    #[derive(Copy, Clone)]
    pub struct StaticMyTraitInterface {
        get_static_dependency_data: fn() -> &'static MyStaticData,
    }

    impl StaticMyTraitInterface {
        /// For any `M: MyTrait`, build its interface table as a `'static`.
        pub const fn of<M: MyTrait>() -> Self {
            Self {
                get_static_dependency_data: M::get_static_dependency_data,
            }
        }

        /// Convenience method to eliminate awkward calling syntax
        pub fn get_data(&self) -> &'static MyStaticData {
            (self.get_static_dependency_data)()
        }
    }

    /// Per-`M` holder that exposes an associated `const` vtable.
    pub struct VTableHolder<M: MyTrait>(PhantomData<M>);

    impl<M: MyTrait> VTableHolder<M> {
        // Lives in read-only memory; taking `&` yields a `'static` reference.
        pub const TABLE: StaticMyTraitInterface = StaticMyTraitInterface::of::<M>();
    }

    /// Our erased handle is just a thin pointer.
    pub type Tag = &'static StaticMyTraitInterface;

    /// Factory: return a `'static` reference to the per-`M` vtable.
    /// Compare to the equivalent function in [`static_reference_to_zst`], which
    /// has an additional `'static` trait constraint on `M` due to the returned
    /// `Tag` type being a trait object rather than a concrete static type.
    pub fn tag_for<M: MyTrait>() -> Tag {
        &VTableHolder::<M>::TABLE
    }

    #[cfg(test)]
    mod test {
        use super::*;

        // A mock implementor of MyTrait
        struct MockType;

        static MOCK_DATA: MyStaticData = MyStaticData {
            msg: "hello from MockType",
        };

        impl MyTrait for MockType {
            fn get_static_dependency_data() -> &'static MyStaticData {
                &MOCK_DATA
            }
        }

        #[test]
        fn tag_returns_expected_static_data() {
            println!(
                "size of StaticMyTraitInterface: {}",
                core::mem::size_of::<StaticMyTraitInterface>()
            );
            println!(
                "size of &'static StaticMyTraitInterface: {}",
                core::mem::size_of::<&'static StaticMyTraitInterface>()
            );
            println!("size of Tag: {}", core::mem::size_of::<Tag>());

            // Obtain an erased tag for MockType
            let tag = StaticMyTraitInterface::of::<MockType>();

            // Call via the vtable method
            let data = tag.get_data();

            assert_eq!(data.msg, "hello from MockType");
        }

        #[test]
        fn tag_identity_is_unique_per_type() {
            struct AnotherType;
            static ANOTHER_DATA: MyStaticData = MyStaticData {
                msg: "another type",
            };
            impl MyTrait for AnotherType {
                fn get_static_dependency_data() -> &'static MyStaticData {
                    &ANOTHER_DATA
                }
            }

            let tag1 = StaticMyTraitInterface::of::<MockType>();
            let tag2 = StaticMyTraitInterface::of::<AnotherType>();

            // Pointers differ because they reference different vtables
            assert_ne!(&tag1 as *const _, &tag2 as *const _);
            assert_eq!(tag1.get_data().msg, "hello from MockType");
            assert_eq!(tag2.get_data().msg, "another type");
        }

        #[test]
        fn static_tag_returns_expected_static_data() {
            let tag = tag_for::<MockType>();
            let data = tag.get_data();
            assert_eq!(data.msg, "hello from MockType");
        }

        #[test]
        fn static_vtables_are_distinct_per_concrete_type() {
            struct AnotherType;
            static ANOTHER_DATA: MyStaticData = MyStaticData {
                msg: "another type",
            };

            impl MyTrait for AnotherType {
                fn get_static_dependency_data() -> &'static MyStaticData {
                    &ANOTHER_DATA
                }
            }

            let t1 = tag_for::<MockType>();
            let t2 = tag_for::<AnotherType>();

            // Different vtables => different addresses
            assert_ne!(t1 as *const _, t2 as *const _);

            assert_eq!(t1.get_data().msg, "hello from MockType");
            assert_eq!(t2.get_data().msg, "another type");
        }
    }
}

pub mod static_reference_to_zst {
    /*!
    ## Using a static reference to a ZST

    As in the strategy of [`simple_static_vtable`], a single static instance for each concrete
    type `M` is stored as an associated constant within a trait having `M` as a generic parameter.
    However, type erasure is achieved by storing a reference to a ZST instead of a function pointer.
    The "token" value is two machine words instead of one.

    First, we create a ZST [`TypedTag<M: MyTrait>`] to which we can attach the concrete
    implementing type. Its "job" is really just to be able to "name" the concrete type. We can
    create (zero-sized) instances of this new type without creating instances of the concrete
    (potentially non-zero-sized) type `M`, but this isn't much of an advantage, because we
    need to know `M` in order to interact with it at all, and so we might as well just call
    the static methods on the original type `M` directly. What we are wanting is a way to pass
    around a type-erased value that still gives us (type-erased) access to the underlying static
    interface of `M` but without instantiating a concrete `M` (i.e. no `Box<dyn MyTrait>`).

    As usual, we use a trait [`TypeErasedTag`] to erase the concrete type `M`, and we provide a blanket
    implementation for all `TypedTag<M: MyTrait>`. But these aren't values we can pass around directly.
    They need to be "behind" a reference or box. So to finish, we create a type alias [`Tag`] that
    represents a type-erased reference to a [`TypeErasedTag`], and a factory function [`tag_for`] that
    builds the type-erased tag for a given concrete type `M`.
    */

    use core::marker::PhantomData;

    use super::*;

    /// A ZST that "names" `M`. This gives us access to the concrete type `M` without instantiating it.
    pub struct TypedTag<M: MyTrait>(PhantomData<M>);

    impl<M: MyTrait> TypedTag<M> {
        /// One ZST instance per M that we can take a reference to.
        pub const INSTANCE: Self = Self(PhantomData);
    }

    /// As usual, we use a trait [`TypeErasedTag`] to erase the concrete type `M`, and we
    /// provide a blanket implementation for all [`TypedTag<M: MyTrait>`].
    pub trait TypeErasedTag {
        fn get_static_dependency_data(&self) -> &'static MyStaticData;
    }

    impl<M: MyTrait> TypeErasedTag for TypedTag<M> {
        fn get_static_dependency_data(&self) -> &'static MyStaticData {
            M::get_static_dependency_data()
        }
    }

    /// Type-erased handle: just a fat pointer (2 words), no alloc.
    pub type Tag = &'static dyn TypeErasedTag;

    /// Factory: build the erased tag for a given `M` (without allocating). Compare to the
    /// equivalent function in [`simple_static_vtable`]. We require an additional `'static`
    /// trait constraint, because `Tag` is a trait object rather than a concrete type.
    pub fn tag_for<M: MyTrait + 'static>() -> Tag {
        &TypedTag::<M>::INSTANCE
    }

    #[cfg(test)]
    mod test {
        use super::*;

        /// Mock implementor of `MyTrait`
        struct MockType;

        static MOCK_DATA: MyStaticData = MyStaticData {
            msg: "hello from MockType",
        };

        impl MyTrait for MockType {
            fn get_static_dependency_data() -> &'static MyStaticData {
                &MOCK_DATA
            }
        }

        #[test]
        fn tag_returns_expected_static_data() {
            println!("size of Tag: {}", core::mem::size_of::<Tag>());

            let tag = tag_for::<MockType>();
            let data = tag.get_static_dependency_data();
            assert_eq!(data.msg, "hello from MockType");
        }

        #[test]
        fn distinct_types_have_distinct_vtables() {
            struct AnotherType;

            static ANOTHER_DATA: MyStaticData = MyStaticData {
                msg: "another type",
            };

            impl MyTrait for AnotherType {
                fn get_static_dependency_data() -> &'static MyStaticData {
                    &ANOTHER_DATA
                }
            }

            let t1 = tag_for::<MockType>();
            let t2 = tag_for::<AnotherType>();

            // The underlying vtables differ between concrete types.
            assert_ne!(t1 as *const _, t2 as *const _);

            assert_eq!(t1.get_static_dependency_data().msg, "hello from MockType");
            assert_eq!(t2.get_static_dependency_data().msg, "another type");
        }
    }
}
