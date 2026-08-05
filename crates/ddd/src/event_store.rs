use crate::DomainEvent;

/// Append-only persistence for [`DomainEvent`]s, the backbone of event sourcing,
/// and a driven [`crate::Port`].
///
/// Instead of storing current state, an event-sourced system stores the full
/// ordered history of what happened and rebuilds state by replaying it. Not
/// every system needs this, a store that keeps only the latest materialised
/// state can suffice, but it is the natural home for a complete, replayable undo
/// history should snapshots ever become too heavy.
///
/// The store is typed to one [`DomainEvent`] here. A real stream of
/// heterogeneous event types needs a shared event enum to append and load
/// through, which this minimal shape deliberately leaves to the implementor.
pub trait EventStore<Event: DomainEvent + Clone> {
    /// Appends an event to the end of the stream.
    fn append(&self, event: &Event);

    /// Loads the full event stream in the order it was appended.
    fn load(&self) -> Vec<Event>;
}
