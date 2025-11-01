# In-progress

"In-progress" means I have bits of it committed to the repo, but it's
either in a rough / first draft state or its missing important bits.

- sealed traits
  - needs examples of each and some exposition. Predrag Gruevski already covered how it works, so exposition here just means "what it does."
- `impl`-side constraints

# Planned

"Planned" means I more-or-less know what I want to say but haven't written it down yet. Some of what
I have to say might have already been said elsewhere a lot better, in which case I won't bother.

- "lazy" initialization
  - levels of laziness: lazy construction of owning container vs lazy construction of owned value, implications for 
    avoiding double borrow. Granularity of mutation mechansim (`RefCell`). 
  - mutation in an immutable context - multiple different models
- Patterns for dealing with immutability vs. mutation
  - separating mutable and immutable phases
  - `Thing::with_widget(&mut self, f: impl FnOnce(&mut Widget)` pattern
  - change lists (collect tokens representing changes that need to happen during an immutable phase)
- working around Rust orphan / coherence rules
  - if there _can_ be a conflict––there doesn't have to _actually_ be one.
  - blanket impls
- "extensible": There has been a lot of discussion about trait objects vs. enums vs.
  opaque types that subvert the type system vs.... But not a lot of it is high quality software engineering exposition.
- Type-state programming: Do I have anything to say about this that hasn't been said yet?
- Something about the type spaghetti that I needed to do for `QueryResultIterator`
  - Need to erase the _unrepresentable_ type of the iterator constructed using iterator combinators.
  - How `fn make_thing() -> impl Trait` works but is very fragile.
- tuples: anything worth saying about lessons learned from multi-properties?


# Other

Another collection of notes: https://qouteall.fun/qouteall-blog/2025/How%20to%20Avoid%20Fighting%20Rust%20Borrow%20Checker
Discussion: https://lobste.rs/s/cnhjj2/how_avoid_fighting_rust_borrow_checker
