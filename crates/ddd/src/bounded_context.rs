/// A boundary within which one model and one ubiquitous language hold
/// consistently, strategic DDD's central unit.
///
/// The same word means different things in different contexts, for example an
/// order in a sales context is not an order in a shipping context, so each
/// context keeps its own model and translates at the edges. A bounded context
/// usually maps to a crate here. An intent marker, it labels the type that
/// anchors a context.
pub trait BoundedContext {}
