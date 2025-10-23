/*!

# The "Registry Pattern"

The "Registry Pattern" is a plugin-like pattern for managing a collection of
[`RegisteredItem`] instances such that discovery of items is automatic/implicit.

For our use case, the [`RegisteredItem`]s are lazily registered (the allocation or
instantiation of _static_ data) and instantiated (for _instance_ data), but this is not
an essential part of the pattern. What _is_ essential is that the [`RegisteredItem`]s
are automatically discovered, even if they are defined in external crates.

In the simplest formulation, the "registry pattern" is just a vector of [`RegisteredItem`]s
each one of which has a static `index` variable in which is stored its position in
the vector that owns it. We just provide a mechanism for assigning the indices to
the [`RegisteredItem`]s at runtime in a logically consistent and thread-safe way
that doesn't require explicit knowledge of every [`RegisteredItem`] type that exists.

We define the [`RegisteredItems`] struct as a wrapper type around the vector of [`RegisteredItem`]
instances to provide convenient [`RegisteredItems::get_item`] and [`RegisteredItems::get_item_mut`]
methods, which automatically instantiate the [`RegisteredItem`] _instance_ if it has
not yet been initialized. Note that we intentionally do not call the [`RegisteredItems`]
struct a "registry" because it is not necessarily a singleton. In fact, we might have many
[`RegisteredItems`] instances each holding `RegisteredItem` instances it owns. "The registry"
that records which `RegisteredItem` types exist is purely abstract; depending on the underlying
implementation, there might not be any actual list of which `RegisteredItem` types exist at all!

A [`RegisteredItem`] need not actually be a singleton, but it, or rather its _type_, is only ever
_registered_ once at runtime; that is, its `index`, which is shared among all instances of the
type, is initialized only once. This allows a "registered item" type to have "singleton metadata"
[¹](#note1) associated with it in addition to whatever instance data it needs. As already mentioned,
at minimum each [`RegisteredItem`] knows its own `index`, which is used to access the item from
the owning [`RegisteredItems`] container via simple indexing into a vector of [`RegisteredItem`]s.
(See the ['plugins' module](crate::plugins) for more details about implementation.)


## Q: Wait, aren't you just talking about static variables? Or, aren't you just talking about lazy initialization of static data?

A: No, I am describing an interface for *automatic* discovery of special types
and, (though only incidentally) the initialization of their static data. How rich
the static data is depends entirely on your use case, so long as it at minimum has
an `index` field to make accessing instances of the items in the owning container easy.

## Notes
<a name="note1">1.</a>
This is usually just called _static_ data, or data in a _static variable_, but I am trying
to avoid any assumptions about when this data is initialized and whether it is mutable.

*/

use std::{
    any::{Any, TypeId},
    cell::{OnceCell, RefCell},
    collections::HashSet,
    sync::{
        LazyLock, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
};

use polonius_the_crab::{polonius, polonius_return};

/// A trait for items that can be registered (`DataPlugin`, `PersonProperty`)
pub trait RegisteredItem: Any {
    /// Convenient for debugging.
    fn name() -> &'static str
    where
        Self: Sized;

    /// The index of the item in the [`RegisteredItem`] list, typically just a vector that lives in the parent.
    fn index() -> usize
    where
        Self: Sized;

    /// Creates a new instance of the item.
    fn new_boxed() -> Box<Self>
    where
        Self: Sized;
}

/// A boxed [`RegisteredItem`]
pub type BxRegisteredItem = Box<dyn RegisteredItem>;

/// Global item index counter, keeps track of the index that will be assigned to the next item that
/// requests an index. Equivalently, holds a *count* of the number of items currently registered.
///
/// If the "registry pattern" is implemented using "run before `main()`" (which is what we do in
/// this example code; see ['plugins' module](crate::plugins) for discussion), `NEXT_ITEM_INDEX`
/// will hold the total count of all registered items by the time any client code runs,
/// because each type implementing the `RegisteredItem` trait calls `add_to_registry()` before
/// the program's `main()` method is called. If the implementation uses the "distributed
/// slice" mechanism instead, then `NEXT_ITEM_INDEX` increases monotonically as each type
/// implementing the `RegisteredItem` initializes its own `index` static variable, which
/// occurs according to whatever initialization strategy you adopt for `RegisteredItem` types.
pub static NEXT_ITEM_INDEX: Mutex<usize> = Mutex::new(0);

/// For simple applications, just knowing how many items are registered is
/// enough, and that information is already captured in [`NEXT_ITEM_INDEX`]. In
/// more sophisticated implementations, we would want to store metadata about the
/// items that are registered.
///
/// For example, we might want to track dependencies between registered items. We could
/// store that metadata in a static variable, or we could have a global data store like
/// the following. The downside of using a static variable to store the metadata is
/// that client code would be responsible for its implementation. If instead we use a
/// global data store, then the client code can just use the `add_to_registry` method
/// to add items to the store and wouldn't need to worry about the implementation.
#[allow(unused)]
pub static REGISTERED_ITEMS: LazyLock<Mutex<RefCell<HashSet<TypeId>>>> =
    LazyLock::new(|| Mutex::new(RefCell::new(HashSet::default())));

/// Adds a new item to the registry. The job of this method is to create whatever "singleton"
/// data/metadata is associated with the [`RegisteredItem`] if it doesn't ready exist. The "registry"
/// is a global singleton (literally just [`NEXT_ITEM_INDEX`] in this implementation), so it is
/// enough to know the concrete type of the [`RegisteredItem`]. For this illustration, the only singleton
/// metadata that is instantiated is the item's index, which we return in case the caller wants it.
///
/// We could also interpret this method as a lazy getter for the static metadata associated with the
/// item.
pub fn add_to_registry<R: RegisteredItem>() -> usize {
    R::index()
    // If we had a global store for the metadata, we would do something like this:
    // REGISTERED_ITEMS
    //     .lock()
    //     .unwrap()
    //     .borrow_mut()
    //     .insert(<R as RegisteredItem>::type_id());
}

/// An accessor for `NEXT_ITEM_INDEX`
pub fn get_data_plugin_count() -> usize {
    *NEXT_ITEM_INDEX.lock().unwrap()
}

/// Acquires a global lock on the next available plugin index, but only increments
/// it if we successfully initialize the provided index. The `index` of a registered
/// item is assigned at runtime but only once per type. It's possible for a single
/// type to attempt to initialize its index multiple times from different threads,
/// which is why all this synchronization is required. However, the overhead
/// is negligible, as this initialization only happens once upon first access.
pub fn initialize_item_index(plugin_index: &AtomicUsize) -> usize {
    // Acquire a global lock.
    let mut guard = NEXT_ITEM_INDEX.lock().unwrap();
    let candidate = *guard;

    // Try to claim the candidate index. Here we guard against the potential race condition that
    // another instance of this plugin in another thread just initialized the index prior to us
    // obtaining the lock. If the index has been initialized beneath us, we do not update
    // [`NEXT_ITEM_INDEX`], we just return the value `plugin_index` was initialized to.
    // For a justification of the data ordering, see:
    //     https://github.com/CDCgov/ixa/pull/477#discussion_r2244302872
    match plugin_index.compare_exchange(usize::MAX, candidate, Ordering::AcqRel, Ordering::Acquire)
    {
        Ok(_) => {
            // We won the race — increment the global next plugin index and return the new index
            *guard += 1;
            candidate
        }
        Err(existing) => {
            // Another thread beat us — don’t increment the global next plugin index,
            // just return existing
            existing
        }
    }
}

pub struct RegisteredItems {
    /// In this pattern, the [`RegisteredItem`]s are stored in a `Vec<Box<dyn Any>>`.
    items: Vec<OnceCell<Box<dyn Any>>>,

    _phantom: std::marker::PhantomData<dyn RegisteredItem>,
}

impl RegisteredItems {
    /// Fetches an immutable reference to the item `R` from the registry. This
    /// implementation lazily instantiates the item if it has not yet been instantiated.
    #[must_use]
    pub fn get_item<R: RegisteredItem>(&self) -> &R {
        let index = R::index();
        self.items
        .get(index)
        .unwrap_or_else(|| panic!("No registered item found with index = {index:?}. You must use the `define_registered_item!` macro to create a registered item."))
        .get_or_init(|| R::new_boxed())
        .downcast_ref::<R>()
        .expect("TypeID does not match registered item type. You must use the `define_registered_item!` macro to create a registered item.")
    }

    /// Fetches a mutable reference to the item `R` from the registry. This
    /// implementation lazily instantiates the item if it has not yet been instantiated.
    #[must_use]
    pub fn get_item_mut<R: RegisteredItem>(&mut self) -> &mut R {
        let mut self_shadow = self;
        let index = R::index();

        // If the item is already initialized, return a mutable reference.
        // Use polonius to address borrow checker limitations.
        polonius!(|self_shadow| -> &'polonius mut R {
            if let Some(any) = self_shadow.items[index].get_mut() {
                polonius_return!(
                    any.downcast_mut::<R>()
                        .expect("TypeID does not match registered item type")
                );
            }
            // Else, don't return. Fall through and initialize.
        });

        // Initialize the item.
        let cell = self_shadow
        .items
        .get_mut(index)
        .unwrap_or_else(|| panic!("No registered item found with index = {index:?}. You must use the `define_registered_item!` macro to create a registered item."));
        let _ = cell.set(R::new_boxed());
        cell.get_mut()
        .unwrap()
        .downcast_mut::<R>()
        .expect("TypeID does not match registered item type. You must use the `define_registered_item!` macro to create a registered item.")
    }
}

/// This macro ensures correct implementation of the `RegisteredItem` trait. The tricky bit is the implementation of
/// `RegisteredItem::index`, which requires synchronization in multithreaded runtimes. This is an instance of
/// _correctness via macro_.
#[macro_export]
macro_rules! registered_item_impl {
    ($item_name:ident) => {
        impl $crate::item_registry::RegisteredItem for $item_name {
            fn name() -> &'static str
            where
                Self: Sized,
            {
                stringify!($item_name)
            }

            fn index() -> usize {
                // This static must be initialized with a compile-time constant expression.
                // We use `usize::MAX` as a sentinel to mean "uninitialized". This
                // static variable is shared among all instances of this concrete item type.
                static INDEX: std::sync::atomic::AtomicUsize =
                    std::sync::atomic::AtomicUsize::new(usize::MAX);

                // Fast path: already initialized.
                let index = INDEX.load(std::sync::atomic::Ordering::Relaxed);
                if index != usize::MAX {
                    return index;
                }

                // Slow path: initialize it.
                $crate::item_registry::initialize_data_plugin_index(&INDEX)
            }

            fn new() -> Box<Self> {
                Box::new($item_name)
            }
        }

        $crate::paste::paste! {
            $crate::ctor::declarative::ctor!{
                #[ctor]
                fn [<_register_item_$item_name:snake>]() {
                    $crate::item_registry::add_to_registry::<$item_name>()
                }
            }
        }
    };
}
