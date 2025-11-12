/*!

# Techniques for type-erasure.

The [type-erased API](type_erased_api/index.html) module is an illustration
of a type, a database index, having both a typed and type-erased API.

Sometimes you don't have complete control over the type you want to expose.
Suppose you want a type-erased interface to a _type_ but not _instances_
of that type. The [`static_interface`](static_interface/index.html)
module illustrates how to do this in a couple of different ways.


*/

pub mod static_interface;
pub mod type_erased_api;
