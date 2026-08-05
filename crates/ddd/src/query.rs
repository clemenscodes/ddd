use crate::ApplicationLayer;
use crate::Layered;

/// A request to read data without changing anything, the read half of CQRS.
///
/// Where a [`crate::Command`] expresses an intention to change state, a query
/// expresses an intention to observe it, for example "give me every matching
/// record" or "give me the current summary". Separating the two lets reads be
/// served from a shape optimised for reading (a [`crate::ReadModel`]) instead of
/// the write model.
pub trait Query: Layered<Layer = ApplicationLayer> {
    /// The data this query returns.
    type Output;
}
