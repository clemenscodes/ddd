use crate::ApplicationLayer;
use crate::DomainEvent;
use crate::Layered;

/// Reacts to a [`DomainEvent`] that has been published, the subscriber side of
/// the event flow.
///
/// One event can have many independent handlers, each blind to the others. When
/// a given event fires, one handler might refresh a read model while another
/// logs it. Handlers coordinate the application. They never live inside the
/// aggregate that raised the event.
pub trait EventHandler<Event: DomainEvent>: Layered<Layer = ApplicationLayer> {
    /// Handles an event that has occurred.
    fn handle(&self, event: &Event);
}
