/// A denormalised structure shaped for reading, not for enforcing invariants.
///
/// The write model (the [`crate::AggregateRoot`]) is optimised for consistency, a
/// read model is optimised for a specific view, a summary list, a dashboard
/// view. It is derived, disposable, and rebuildable from the write model or
/// from events via a [`crate::Projection`], so it holds no authority of its own.
/// An intent marker, it labels a type as read-side, derived state.
///
/// Unlike most markers here it pins no layer. A read model is derived on the write
/// side yet consumed on the read edge and belongs cleanly to neither of the four
/// layers, so the omission is deliberate, not an oversight. A [`crate::Projection`]
/// inherits this, being a `ReadModel`.
pub trait ReadModel {}
