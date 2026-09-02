use crate::DomainEvent;
use core::future::Future;

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
///
/// # Every call waits and every call can be refused
///
/// What sits behind an event store is a file, a database or another machine, so
/// appending takes time and loading takes time, and either can come back a
/// refusal. A signature that says otherwise forces every implementation to lie
/// about what it does, either by blocking a thread inside a call that claims not
/// to wait or by swallowing a refusal it has no way to report. The failure is
/// the implementation's own, because what can go wrong belongs to the thing that
/// stores rather than to the domain that asks.
///
/// The futures are `impl Future` rather than boxed futures behind a macro, so a
/// call allocates nothing and this crate keeps the no dependency property it has
/// always had. The cost is that this trait is not object safe, which no consumer
/// of it has ever needed, since every one of them dispatches statically. `Send`
/// is required of every future because an implementation is normally driven by a
/// work stealing runtime that may move a task between threads.
pub trait EventStore<Event: DomainEvent + Clone> {
    /// What a refusal from whatever stores this stream is.
    ///
    /// It belongs to the implementation rather than to this trait, because a
    /// database, a file and a remote store fail in ways that have nothing in
    /// common beyond the fact that they can.
    type Failure;

    /// Appends an event to the end of the stream.
    fn append(&self, event: &Event) -> impl Future<Output = Result<(), Self::Failure>> + Send;

    /// Loads the full event stream in the order it was appended.
    fn load(&self) -> impl Future<Output = Result<Vec<Event>, Self::Failure>> + Send;
}
