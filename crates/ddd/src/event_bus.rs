use crate::DomainEvent;
use core::future::Future;

/// Carries [`DomainEvent`]s from where they are raised to whoever subscribed, a
/// driven [`crate::Port`].
///
/// The application publishes through this interface without knowing the
/// transport. The transport is an [`crate::Adapter`] in the infrastructure layer.
///
/// Typed to one [`DomainEvent`] rather than parametrising the method, so a
/// heterogeneous publisher goes through a shared event enum, the same shape
/// [`crate::EventStore`] documents.
///
/// Nothing in this crate wires the write side to the bus, an [`crate::AggregateRoot`]
/// does not record events and a [`crate::Service`] does not publish them after
/// commit. Connecting the two is left to the application, deliberately, to keep
/// the vocabulary minimal.
///
/// # Publishing waits and can be refused
///
/// What sits behind a bus is a transport, a channel, a queue or another machine,
/// so handing an event across it takes time and it can come back a refusal. A
/// signature that says otherwise forces every implementation to lie about what
/// it does, either by blocking a thread inside a call that claims not to wait or
/// by swallowing a refusal it has no way to report. The failure is the
/// implementation's own, because what can go wrong belongs to the transport
/// rather than to the domain that raised the event.
///
/// The future is `impl Future` rather than a boxed future behind a macro, so a
/// publish allocates nothing and this crate keeps the no dependency property it
/// has always had. The cost is object safety, so this port is no longer reachable
/// as `&dyn EventBus<E>` and is dispatched statically like every other port here.
/// `Send` is required because a publish is normally driven by a work stealing
/// runtime that may move a task between threads.
pub trait EventBus<Event: DomainEvent> {
    /// What a refusal from the transport carrying these events is.
    ///
    /// It belongs to the implementation rather than to this trait, because an
    /// in-process channel, a broker and a remote endpoint fail in ways that have
    /// nothing in common beyond the fact that they can.
    type Failure;

    /// Publishes an event to every subscribed handler.
    fn publish(&self, event: Event) -> impl Future<Output = Result<(), Self::Failure>> + Send;
}
