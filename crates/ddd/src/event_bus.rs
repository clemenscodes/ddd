use crate::DomainEvent;

/// Carries [`DomainEvent`]s from where they are raised to whoever subscribed, a
/// driven [`crate::Port`].
///
/// The application publishes through this interface without knowing the
/// transport. The transport is an [`crate::Adapter`] in the infrastructure layer.
///
/// Typed to one [`DomainEvent`] so the trait stays object-safe, which a driven
/// port needs, `&dyn EventBus<E>`. Parametrising the method instead would forfeit
/// that. A heterogeneous publisher goes through a shared event enum, the same
/// shape [`crate::EventStore`] documents.
///
/// Nothing in this crate wires the write side to the bus, an [`crate::AggregateRoot`]
/// does not record events and a [`crate::Service`] does not publish them after
/// commit. Connecting the two is left to the application, deliberately, to keep
/// the vocabulary minimal.
pub trait EventBus<Event: DomainEvent> {
    /// Publishes an event to every subscribed handler.
    fn publish(&self, event: Event);
}
