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
///
/// # Why this one stayed synchronous
///
/// [`crate::Repository`], [`crate::EventStore`], [`crate::EventBus`] and
/// [`crate::UnitOfWork`] became asynchronous and fallible because a store or a
/// transport sits behind each of them, and this trait was not overlooked, it
/// names no boundary of its own. What a handler reaches for, if it reaches for
/// anything, is one of those ports, and that call is where the waiting and the
/// refusing already live. The same holds for [`crate::Projection::apply`], which
/// folds an event into a read model in memory and so waits on nothing at all.
pub trait EventHandler<Event: DomainEvent>: Layered<Layer = ApplicationLayer> {
    /// Handles an event that has occurred.
    fn handle(&self, event: &Event);
}
