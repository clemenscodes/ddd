/// A boundary interface the application defines and the outside world plugs into,
/// the ports of Ports and Adapters, the hexagonal architecture.
///
/// In Rust a port is a trait, [`crate::Repository`] and [`crate::EventBus`] are
/// ports already. This marker is a pragmatic label for the concept, with an honest
/// limitation. That a trait is a port is a property of a trait, and Rust markers
/// attach to types, so it cannot be enforced the way a type's layer can. It
/// documents the hexagonal role, the crate graph does the enforcing. Because of
/// that the port traits stay unmarked, the marker rides on a concrete type where
/// naming its hexagonal role earns its keep, not on `Repository`/`EventBus`.
pub trait Port {}

/// A driving, or primary, port, the interface the application offers to the world
/// that drives it, the use cases a [`crate::Service`] exposes, invoked by the
/// presentation layer.
pub trait DrivingPort: Port {}

/// A driven, or secondary, port, what the application needs from the world,
/// persistence through a [`crate::Repository`] and messaging through an
/// [`crate::EventBus`], which infrastructure supplies as an [`crate::Adapter`].
pub trait DrivenPort: Port {}
