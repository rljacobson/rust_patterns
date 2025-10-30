/*!

# Information Hiding, Encapsulation, and Shared Implementation

Rust supports shared interfaces via traits. However, Rust's support for
shared _implementation_ is very limited.

A trait can have a default implementation for a method, but this support for shared
implementation is hamstrung by the fact that traits cannot have data members of any
visibility. Thus, any default implementation can only assume about `self` what the trait
itself asserts and cannot access any data directly. What's more, because they are intended
as a mechanism for defining public interfaces, trait methods are necessarily public.

Generics have similar limitations: they can assume very little about the type parameters,
and they cannot access data members of any visibility. Their advantage is that they are
monomorphized at compile time and thus don't incur the cost of runtime dynamic dispatch.

For someone coming from OOP languages like C++, Rust's lack of language-level
support for _shared implementation_ (not just shared interfaces) can be a pain
point. This module demonstrates patterns for dealing with these limitations:

- The sealed trait concept is an umbrella for a set of techniques for achieving fine-grained access and
  overridability control of trait methods while also maintaining a level of encapsulation.
- `impl`-side trait constraints and blanket trait implementations are patterns
  for dealing with Rust's ophan/coherence rules and that can result in cleaner library interfaces.

## Software Engineering 101.

*Information hiding* means concealing the internal details (design decisions,
data representations, or algorithms) of a software component so that other parts of the system
depend only on its public interface, not on its inner workings.[¹](#info_hiding)

| Term              | Definition                                                   | Relationship to Information Hiding                           |
| ----------------- | ------------------------------------------------------------ | ------------------------------------------------------------ |
| **Encapsulation** | The mechanism enforcing information hiding at the language level. | The implementation of information hiding in the source code. |
| **Abstraction**   | The cognitive process of determining the essential characteristics of a thing without respect to its implementation. | The intellectual outcome of information hiding.              |
| **Modularity**    | Having components ("modules") that are self-contained, have well-defined responsibilities and interfaces, and can be developed, tested, and maintained independently. | The system-level outcome of information hiding.              |

One can (and *should*) quibble with these definitions, but they are good enough for our purpose.

## Notes

<a name="note1">1.</a> The term *information hiding* was introduced by David
Parnas in his 1972 paper, *[“On the Criteria To Be Used in Decomposing Systems
into Modules.”](https://dl.acm.org/doi/pdf/10.1145/361598.361623)* In this
context, the word _module_ is synonymous with _software component_ and need
not correspond directly to the Rust language feature of the same name.

*/

pub mod impl_side_constraints;
pub mod sealed_traits;
