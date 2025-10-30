/*!
# The Registry Pattern

The *registry pattern* is a plugin-like pattern for managing a collection of
[`RegisteredItem`] instances such that discovery of items is automatic/implicit.

For our use case, the [`RegisteredItem`]s are lazily registered (the allocation or
instantiation of _static_ data) and instantiated (for _instance_ data), but this is not
an essential part of the pattern. What _is_ essential is that the [`RegisteredItem`]s
are automatically discovered, *even if they are defined in external crates*.

> **Note:** If you are happy with the _lazy_ discovery of [`RegisteredItem`]s, then you
> can get away with a simpler implementation that uses neither "run before `main()`" nor
> "distributed slice" mechanisms (see ['plugins' module](crate::plugins) for discussion).
> There is a trade-off: You need a mechanism for interior mutability of `RegisteredItems`
> (the owner of the item instances) if you want to initialize the [`RegisteredItem`] on
> first access, because you don't know how many [`RegisteredItem`] types there are globally
> when [`RegisteredItems`] is constructed and thus don't know how big the vector should be.

In the simplest formulation, the "registry pattern" is just a vector of [`RegisteredItem`]s, one instance for each concrete [`RegisteredItem`] type that exists,
each one of which having a static `index` variable in which is stored its position in
the vector that owns it. We just provide a mechanism for assigning the indices to
the concrete [`RegisteredItem`] types at runtime in a logically consistent and thread-safe way
that doesn't require explicit knowledge of every [`RegisteredItem`] type that exists. To recap: We have one item instance per concrete [`RegisteredItem`]  type per store.

In the next section we go into much more detail about the design decisions one has to make for any given implementation of the registry pattern. In that section we will point out which decisions we've made for the example we are illustrating in the code provided here, but here's a brief rundown of the main points: We define the [`RegisteredItems`] struct as a wrapper type around the vector of [`RegisteredItem`]
instances to provide convenient [`RegisteredItems::get_item`] and [`RegisteredItems::get_item_mut`]
methods, which automatically instantiate the [`RegisteredItem`] _instance_ if it has
not yet been initialized. Note that we intentionally do not call the [`RegisteredItems`]
struct a "registry" because it is not necessarily a singleton. In fact, we might have many
[`RegisteredItems`] instances each holding `RegisteredItem` instances it owns. "The registry"
that records which `RegisteredItem` types exist is purely abstract; depending on the underlying
implementation, there might not be any actual list of which `RegisteredItem` types exist at all!

A [`RegisteredItem`] need not actually be a singleton, but it, or rather its _type_, is only ever
_registered_ once at runtime; that is, its `index`, which is shared among all instances of the
type, is initialized only once (in a `ctor`). This allows a "registered item" type to have "singleton metadata"
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

# Design Options for the Registry Pattern

In this section we explore in detail the different design choices one must make when implementing the registry pattern. We will call the `RegisteredItem` types of a *particular* implementation of the registry pattern the `Widget` types, or similar made up name. The owner will be some type wrapping a vector, which we'll call the `WidgetStore`, or some variant of `*Store`.

## Design Dimensions for `Widget`s and `WidgetStore`s

Below is my attempt at carving up the design space for our registry pattern implementation with a few alternatives listed for each. I'm using the phrase *dimension* to refer to the major orthogonal design choices that need to be made about our container. By *orthogonal* I mean a choice in one dimension does not depend on a choice in another dimension. In actual practice, implementation choices in one dimension may affect implementation in another dimension, so this independence is approximate and "leaky".

All registered items that participate in the same registry pattern implementation must share the semantics implied by these design choices to avoid significant complexity explosion.

**Initialization time**

- In a singleton at program start (a la "distributed slice", prototype style)
- Upon `WidgetStore` creation (an instance for each `Widget` type that exists is eagerly created)
- Lazily, e.g. upon first access in `WidgetStore`. The "slots" in the owning `WidgetStore` are occupied by `OnceCell`s
- Something else? E.g. super lazily: upon write access in `WidgetStore`.

*In this example:* We lazily instantiate `Widget`s upon first access. We use `ctor` magic to know how many "slots" the `WidgetStore` needs to allocate.

**Ownership semantics:** Specific to `WidgetStore`'s relationship to `Widget` values, _not_ to views client code have into the data.

- `Widget`s are owned by the `WidgetStore` which gives out regular mutable or immutable references
- A `WidgetStore` is either literally or essentially a singleton (possibly a naked "distributed slice"), only one instance of each `Widget` type is created globally.
- `Widget`s are shared, stored in `Arc`/`Rc` or `Cow` types.
- `Widget`s are `Copy` types
- Move semantics. I'm not sure what the use case would be, but it's possible to move `Widget` instances out of the `WidgetStore`.

*In this example:* `Widget`s are owned by the `WidgetStore` which gives out regular mutable or immutable references

**Interior mutability**

- "default" semantics: `Widget`s are owned by the `WidgetStore`, which is in turn owned by the `Context` (say), and they are all mutable or immutable at once accordingly.
- `WidgetStore` lives in a `RefCell`, so entire store is mutable at once or immutable at once
- `WidgetStore` holds individual `Widget`s in `RefCell`s, so multiple distinct `Widget`s can be simultaneously mutated
- `Widget`s are `Copy` types, so they can be copied around and mutated in place (possibly as atomic types)
- `Widget`s implement their own interior mutability

*In this example:* "default" semantics (modulo `OnceCell`'s internal mechanisms); Mutation requires mutable access to the `WidgetStore`

**Access control:** what "views" into the data are enabled

- Client code get's "direct" access to a `&Widget` or `&mut Widget`
- Client code passes a closure to a `with_widget` method, which closure is passed a `&Widget`; and similarly for mutable variant
- `WidgetStore` provides rich API for interacting with `Widget`s, e.g. `WidgetStore::froculate_widget::<Sprocket>()` froculate's a widget on the caller's behalf. (We generally do this with `DataPlugin`s, for example.)
- Pass out `Ref<'a, Widget>`s / `RefMut<'a, Widget>`s  to client code, which implement `Deref`/`DerefMut` to `&Widget`/`&mut Widget`
- `Widget`s have `Copy` semantics, `WidgetStore::get()` returns a copy of the stored `Widget`. Have a corresponding `WidgetStore::set()` instead of `WidgetStore::get_mut()` with reference semantics.
- YOLO raw pointers 🤣

*In this example:* Client code gets "direct" access to a `&Widget` or `&mut Widget`.

**Other Dimensions:** Some other dimensions I don't have much to say about.

- **Type Discovery:** We want "automatic" discovery, but some designs can get away with not using `ctor`/`linkme` tricks, in particular when discovery can happen on an as-needed (we "discover" `ConcreteWidget` only when `WidgetStore::get::<ConcreteWidget>()` is called); see notes below. This example uses `ctor` to know how many concrete `Widget` types there are globally and to assign each an index.
- **Concurrency Model:** In this example, we take care to initialize each `Widget` type's index in a thread-safe way but otherwise don't share access across thread boundaries.
- **Lifetime Model:** How long are `Widget` and `WidgetStore` instances expected to live? This implementation makes no assumptions.

**"Leaky" Implementation Bits:**

- `OnceCell` has its own interior mutability mechanism; same with `inventory`, others; though they can be optimized for specific use cases (e.g. `OnceCell` is fast to access after initialization)
- If `WidgetStore` does not need to be immutable, or if it or its wrapped vector can live in a `RefCell`, then magic `ctor`/`linkme` mechanisms are not necessary, because we don't have to know the total count of all `Widget` types upon `WidgetStore` creation--we can grow the vector as needed.
- If `WidgetStore` needs to be immutable, then we need to know the total count of all `Widget` types upon `WidgetStore` creation in order to allocate enough "slots" for all the `Widget` type _even if we defer `Widget` creation by using `OnceCell`s_.
  - `OnceCell`s push (a fast form of) interior mutability out from the `Context` or `WidgetStore` level to the individual `Widget` level
  - Eager initialization is an alternative to `OnceCell` and isn't necessarily harder to implement, since you already pay the price of fancy linker tricks for the `OnceCell` solution.

## Multiple registries for different _thing_ categories vs. a single registry for all (Generic) Registered Items

We can imagine two different approaches to how many implementations of the registry pattern are in a given project:

1. A single implementation of the registry pattern could conceivably be generic over every category of *thing* that can be registered, or at least every category that matches all of your choices for you have made in all of the dimensions listed in the previous section.
   - Unifies the storage and access for `Widget`s, `Gadget`s, `Blogets`, `Sprockets`, ..., potentially lots of disparate categories of objects.

   - Minimizes lines of code, API
2. You have separate implementations in your project of the registry pattern for each category even if the design choices in all dimensions of the previous section are the same. For example, you might implement the registry pattern for your `Widget` system and then have a separate independent implementation of the registry pattern for your `Gadget` system.
   - Different public API for different categories of objects
   - If we want to change the storage / ownership / access semantics for a category of objects, it's already distinguished from other categories and has its own API.

My advice is to choose (2) almost always. Superficial implementation features are generally poor decomposition criteria. A `Widget` system and a `Gadget` system should generally not be artificially coupled.
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
pub trait RegisteredItem: Any + Default {
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
        Self: Sized {
        Box::new(Default::default())
    }
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
/// data/metadata is associated with the [`RegisteredItem`] if it doesn't already exist. The "registry"
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
pub fn get_registered_item_count() -> usize {
    *NEXT_ITEM_INDEX.lock().unwrap()
}

/// Encapsulates the synchronization logic for initializing an item's index.
///
/// Acquires a global lock on the next available item index, but only increments
/// it if we successfully initialize the provided index. The `index` of a registered
/// item is assigned at runtime but only once per type. It's possible for a single
/// type to attempt to initialize its index multiple times from different threads,
/// which is why all this synchronization is required. However, the overhead
/// is negligible, as this initialization only happens once upon first access.
///
/// In fact, for the example implement provided here, we know we are calling
/// this function once for each type in each `RegisteredItem`'s `ctor` function,
/// which should be the only time this method is ever called for the type.
pub fn initialize_item_index(plugin_index: &AtomicUsize) -> usize {
    // Acquire a global lock.
    let mut guard = NEXT_ITEM_INDEX.lock().unwrap();
    let candidate = *guard;

    // Try to claim the candidate index. Here we guard against the potential race condition that
    // another instance of this plugin in another thread just initialized the index prior to us
    // obtaining the lock. If the index has been initialized beneath us, we do not update
    // [`NEXT_ITEM_INDEX`], we just return the value `plugin_index` was initialized to.
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
    // An argument that the ordering given in the call to
    // `compare_exhange` above is minimally constrained follows.
    //
    // On success (`plugin_index == usize::MAX`), we write the new value (candidate) to the atomic.
    //
    // **The Release ordering (write side):** Prevents previous writes from
    // being reordered after the atomic store. Guarantees that any thread which
    // later acquires this atomic will see all the writes that happened-before.
    //
    // **The Acquire ordering (read side):** Prevents subsequent reads from being moved
    // before the successful `compare_exchange`. Ensures we see all memory effects that
    // happened-before any prior release (of which there are none in our case). On failure,
    // we only read the existing value from the atomic, so we want to guarantee we do not
    // read a stale value. So I think we only need the `Ordering::Acquire` ordering, which
    // prevents the read from being moved before the previous successful `compare_exchange`.
    //
    // We only need `Ordering::AcqRel` instead of `Ordering::SeqCst` for the success case,
    // because we only care about the relative ordering of the reads/writes across threads
    // rather than a single total global ordering all threads agree on. ([The difference is
    // subtle](https://en.cppreference.com/w/cpp/atomic/memory_order.html#Sequentially-consistent_ordering),
    // to put it mildly.)
}


/// A wrapper around a vector of [`RegisteredItem`]s.
pub struct RegisteredItems {
    items: Vec<OnceCell<Box<dyn Any>>>,
}

impl RegisteredItems {

    /// Creates a new [`RegisteredItems`] instance, allocating the exact number
    /// of slots as there are types that implement [`RegisteredItem`]s.
    ///
    /// This method assumes all types implementing [`RegisteredItem`] have been implemented
    /// _correctly_. This is one of the pitfalls of this pattern: there is no guarantee
    /// that types implementing [`RegisteredItem`] implemented a `ctor` that correctly
    /// initializes its `index` and so forth. We can have at least some confidence,
    /// though, in their correctness by supplying a correct implementation via a macro.
    ///
    /// Observe that we create an empty `OnceCell` in each slot in this implementation, but
    /// we could just as easily eagerly initialize the [`RegisteredItem`] instances here
    /// instead (possibly by iterating over constructor functions from [`REGISTERED_ITEMS`]).
    pub fn new() -> Self {
        let num_items = get_registered_item_count();
        Self {
            items: (0..num_items).map(|_| OnceCell::new()).collect(),
        }
    }

    /// Fetches an immutable reference to the item `R` from the registry. This
    /// implementation lazily instantiates the item if it has not yet been instantiated.
    #[must_use]
    pub fn get<R: RegisteredItem>(&self) -> &R {
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
    pub fn get_mut<R: RegisteredItem>(&mut self) -> &mut R {
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
        impl $crate::plugins::item_registry::RegisteredItem for $item_name {
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
                $crate::plugins::item_registry::initialize_item_index(&INDEX)
            }
        }

        $crate::paste::paste! {
            $crate::ctor::declarative::ctor!{
                #[ctor]
                fn [<_register_item_$item_name:snake>]() {
                    $crate::plugins::item_registry::add_to_registry::<$item_name>();
                }
            }
        }
    };
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Barrier};
    use std::thread;

    // Test item types
    #[derive(Debug, Clone, PartialEq)]
    struct TestItem1 {
        value: usize,
    }
    impl Default for TestItem1 {
        fn default() -> Self {
            Self { value: 42 }
        }
    }

    #[derive(Debug, Clone, PartialEq)]
    struct TestItem2 {
        name: String,
    }
    impl Default for TestItem2 {
        fn default() -> Self {
            TestItem2 {
                name: "test".to_string(),
            }
        }
    }

    #[derive(Debug, Clone, PartialEq)]
    struct TestItem3 {
        data: Vec<u8>,
    }
    impl Default for TestItem3 {
        fn default() -> Self {
            TestItem3 {
                data: vec![1, 2, 3],
            }
        }
    }

    // Implement RegisteredItem manually for testing without macro
    registered_item_impl!(TestItem1);
    registered_item_impl!(TestItem2);
    registered_item_impl!(TestItem3);


    // Test the internal synchronization mechanisms of `initialize_item_index()`.
    //
    // It is convenient to only have a single test that mutates `NEXT_ENTITY_INDEX`,
    // because we can assume no other thread is incrementing it and can therefore
    // test the value of `NEXT_ENTITY_INDEX` at the beginning and then at the end of
    // the test.
    //
    // Note that this doesn't really interfere with other tests involving `RegisteredItems`,
    // because at worst `RegisteredItems` will just allocate addition slots for
    // nonexistent items, which will never be requested with a `get()` call.
    #[test]
    fn test_initialize_item_index_concurrent() {
        // Test 1: Try to initialize a single index from multiple threads simultaneously.
        let initial_registered_items_count = get_registered_item_count();

        const NUM_THREADS: usize = 100;
        let index = Arc::new(AtomicUsize::new(usize::MAX));
        let barrier = Arc::new(Barrier::new(NUM_THREADS));

        let handles: Vec<_> = (0..NUM_THREADS)
            .map(|_| {
                let index_clone = Arc::clone(&index);
                let barrier_clone = Arc::clone(&barrier);

                thread::spawn(move || {
                    // Wait for all threads to be ready
                    barrier_clone.wait();
                    // All threads try to initialize at once
                    initialize_item_index(&index_clone)
                })
            })
            .collect();

        let results: Vec<usize> = handles.into_iter().map(|h| h.join().unwrap()).collect();

        let first = results[0];

        // The index should be initialized
        assert_ne!(first, usize::MAX);
        // All threads should get the same index
        assert!(results.iter().all(|&r| r == first));
        // And that index should be what was originally the next available index
        assert_eq!(first, initial_registered_items_count);


        // Test 2: Try to initialize multiple indices from multiple threads simultaneously.
        //
        // Creates 5 different entities (each with their own atomic). Initializes
        // each from a separate thread. Verifies they receive sequential,
        // unique indices. Confirms the global counter matches the entity count.

        // W
        let initial_registered_items_count = get_registered_item_count();

        // Create multiple different entities (each with their own atomic)
        const NUM_ENTITIES: usize = 5;
        let entities: Vec<_> = (0..NUM_ENTITIES)
            .map(|_| Arc::new(AtomicUsize::new(usize::MAX)))
            .collect();

        let mut handles = vec![];

        // Initialize each entity from a different thread
        for entity in entities.iter() {
            let entity_clone = Arc::clone(entity);
            let handle = thread::spawn(move || {
                initialize_item_index(&entity_clone)
            });
            handles.push(handle);
        }

        // Collect results
        let mut results = vec![];
        for handle in handles {
            results.push(handle.join().unwrap());
        }

        // Each entity should get a unique, sequential index starting with `initial_registered_items_count`.
        results.sort();
        for (i, &result) in results.iter().enumerate() {
            assert_eq!(result, i + initial_registered_items_count, "Entity should have index {}, got {}", i, result);
        }

        // Test 3: Try to initialize multiple entities from multiple threads multiple times.

        // We account for the fact that some entities have been initialized
        // in their `ctors`, so the indices we create don't start with 0.
        let initial_registered_items_count = get_registered_item_count();

        // Create 3 entities
        let entity1 = Arc::new(AtomicUsize::new(usize::MAX));
        let entity2 = Arc::new(AtomicUsize::new(usize::MAX));
        let entity3 = Arc::new(AtomicUsize::new(usize::MAX));

        let mut handles = vec![];

        // Multiple threads racing on each of entity1, entity2, entity3
        for _ in 0..5 {
            let e1 = Arc::clone(&entity1);
            handles.push(thread::spawn(move || initialize_item_index(&e1)));

            let e2 = Arc::clone(&entity2);
            handles.push(thread::spawn(move || initialize_item_index(&e2)));

            let e3 = Arc::clone(&entity3);
            handles.push(thread::spawn(move || initialize_item_index(&e3)));
        }

        // Collect all results
        let results: Vec<_> = handles.into_iter()
                                     .map(|h| h.join().unwrap())
                                     .collect();

        // Count occurrences of each index
        let mut counts = std::collections::HashMap::new();
        for &result in &results {
            *counts.entry(result).or_insert(0) += 1;
        }

        // Should have exactly 3 unique indices
        assert_eq!(counts.len(), 3, "Should have 3 unique indices");

        // Each index should appear exactly 5 times (one entity, 5 threads)
        for (&idx, &count) in &counts {
            assert_eq!(
                count, 5,
                "Index {} should appear 5 times, appeared {} times",
                idx, count
            );
        }

        // Global counter should be 3
        assert_eq!(get_registered_item_count() - initial_registered_items_count, 3);

        // Each entity should have one of the indices
        let indices: Vec<_> = vec![
            entity1.load(Ordering::Acquire),
            entity2.load(Ordering::Acquire),
            entity3.load(Ordering::Acquire),
        ];

        let mut sorted_indices = indices.clone();
        sorted_indices.sort();
        // As before, we account for the fact that some entities have been
        // initialized in their `ctors`, so the indices we created don't start at 0.
        let expected_indices = vec![
            0 + initial_registered_items_count,
            1 + initial_registered_items_count,
            2 + initial_registered_items_count
        ];
        assert_eq!(sorted_indices,
                   expected_indices
        );

    }

    // Registering items is idempotent
    #[test]
    fn test_add_to_registry_idempotent() {
        let index1 = TestItem1::index();
        let index2 = TestItem2::index();
        let index3 = TestItem3::index();

        // All should be initialized (uninitialized indices are `usize::MAX`)
        assert_ne!(index1, usize::MAX);
        assert_ne!(index2, usize::MAX);
        assert_ne!(index3, usize::MAX);

        // Each should have a unique index
        assert_ne!(index1, index2);
        assert_ne!(index2, index3);
        assert_ne!(index1, index3);

        // Adding the same type multiple times should return the same index.
        add_to_registry::<TestItem1>();
        add_to_registry::<TestItem1>();
        add_to_registry::<TestItem1>();

        let index_from_registry_1 = TestItem1::index();
        let index_from_registry_2 = TestItem2::index();
        let index_from_registry_3 = TestItem3::index();

        assert_eq!(index1, index_from_registry_1);
        assert_eq!(index2, index_from_registry_2);
        assert_eq!(index3, index_from_registry_3);
    }

    // Getting items lazily initializes `Entity` instances
    #[test]
    fn test_registered_items_get() {
        // Test mutable `RegisteredItems::get_mut`
        {
            let mut items = RegisteredItems::new();

            let item1 = items.get_mut::<TestItem1>();
            assert_eq!(item1.value, 42);
            assert_eq!(TestItem1::name(), "TestItem1");

            let item2 = items.get_mut::<TestItem2>();
            assert_eq!(item2.name, "test");

            let item3 = items.get_mut::<TestItem3>();
            assert_eq!(item3.data, vec![1, 2, 3]);
        }

        // Test immutable `RegisteredItems::get`
        {
            let items = RegisteredItems::new();

            let item1 = items.get::<TestItem1>();
            assert_eq!(item1.value, 42);
            assert_eq!(TestItem1::name(), "TestItem1");

            let item2 = items.get::<TestItem2>();
            assert_eq!(item2.name, "test");

            let item3 = items.get::<TestItem3>();
            assert_eq!(item3.data, vec![1, 2, 3]);
        }
    }

    // Initialization happens once
    #[test]
    fn test_registered_items_get_cached() {
        // Test immutable `RegisteredItems::get`
        {
            let items = RegisteredItems::new();

            // Get the item twice
            let item1_ref1 = items.get::<TestItem1>();
            let item1_ref2 = items.get::<TestItem1>();

            // Both should point to the same instance
            assert!(std::ptr::eq(item1_ref1, item1_ref2));
        }

        // Test mutable `RegisteredItems::get_mut`
        {
            let mut items = RegisteredItems::new();

            // Get the item twice. We can safely get multiple mutable pointers so long as we don't dereference them.
            let item1_ptr1: *mut TestItem1 = items.get_mut::<TestItem1>();
            let item1_ptr2: *mut TestItem1 = items.get_mut::<TestItem1>();

            // Both should point to the same instance
            assert!(std::ptr::eq(item1_ptr1, item1_ptr2));
        }
    }

    #[test]
    fn test_registered_items_get_mut() {
        let mut items = RegisteredItems::new();

        // Get mutable reference and modify
        let item = items.get_mut::<TestItem1>();
        assert_eq!(item.value, 42);
        item.value = 100;

        // Verify the change persisted
        let item = items.get::<TestItem1>();
        assert_eq!(item.value, 100);
    }

    #[test]
    fn test_registered_items_multiple_items_mutated() {
        let mut items = RegisteredItems::new();

        // Read and mutate multiple items
        let item1 = items.get_mut::<TestItem1>();
        assert_eq!(item1.value, 42);
        item1.value = 10;

        let item2 = items.get_mut::<TestItem2>();
        assert_eq!(item2.name, "test");
        item2.name = "modified".to_string();

        let item3 = items.get_mut::<TestItem3>();
        assert_eq!(item3.data, vec![1, 2, 3]);
        item3.data = vec![9, 8, 7];

        // Verify all changes
        assert_eq!(items.get::<TestItem1>().value, 10);
        assert_eq!(items.get::<TestItem2>().name, "modified");
        assert_eq!(items.get::<TestItem3>().data, vec![9, 8, 7]);
    }

    #[test]
    #[should_panic(expected = "No registered item found with index")]
    fn test_registered_items_invalid_index() {
        #[derive(Debug, Default)]
        struct UnregisteredEntity;

        // Intentionally implement `RegisteredItem` incorrectly.
        impl RegisteredItem for UnregisteredEntity {
            fn name() -> &'static str where Self: Sized {
                "UnregisteredItem"
            }

            fn index() -> usize where Self: Sized {
                87000 // An invalid index
            }

            // fn as_any(&self) -> &dyn Any { self }
            // fn as_any_mut(&mut self) -> &mut dyn Any { self }
        }

        // Create items container with insufficient capacity
        let items = RegisteredItems::new();

        // This should panic because TestItem1's index doesn't exist
        let _ = items.get::<UnregisteredEntity>();
    }

    #[test]
    fn test_registered_item_trait_name() {
        assert_eq!(TestItem1::name(), "TestItem1");
        assert_eq!(TestItem2::name(), "TestItem2");
        assert_eq!(TestItem3::name(), "TestItem3");
    }

    #[test]
    fn test_registered_item_new_boxed() {
        let boxed1 = TestItem1::new_boxed();
        assert_eq!(boxed1.value, 42);

        let boxed2 = TestItem2::new_boxed();
        assert_eq!(boxed2.name, "test");

        let boxed3 = TestItem3::new_boxed();
        assert_eq!(boxed3.data, vec![1, 2, 3]);
    }

    #[test]
    fn test_box_dyn_registered_item_type_alias() {
        let item = TestItem1::new_boxed();
        assert_eq!(
            (item as Box<dyn Any>)
                .downcast_ref::<TestItem1>()
                .unwrap()
                .value,
            42
        );
    }
}
