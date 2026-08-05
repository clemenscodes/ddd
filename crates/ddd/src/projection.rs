use crate::DomainEvent;
use crate::ReadModel;

/// Builds a [`ReadModel`] by folding [`DomainEvent`]s into it, the bridge from
/// the write side to a read-optimised shape.
///
/// A projection subscribes to events and updates a denormalised view, folding
/// each event into a running summary, so a read path gets one ready structure
/// instead of recomputing over the full event history on each read. The
/// [`ReadModel`] bound makes the projection itself the read-side structure the
/// events fold into, rather than leaving the target it maintains implicit.
pub trait Projection<Event: DomainEvent>: ReadModel {
    /// Folds one event into the read model.
    fn apply(&mut self, event: &Event);
}
