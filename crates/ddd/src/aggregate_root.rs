use crate::Entity;

/// The single entry point to a cluster of domain objects that must stay mutually
/// consistent, the transactional boundary of the domain.
///
/// Every change to the objects inside the cluster goes through the root, so the
/// root is the one place invariants are enforced. The root is itself an
/// [`Entity`], so it carries the identity a [`crate::Repository`] finds and saves
/// it by, and a [`crate::Service`] mutates exactly one aggregate root through
/// named [`crate::Command`]s. `Clone` lets a service snapshot the aggregate
/// before persisting it. Being an entity, the aggregate lives, by definition, in
/// the domain layer.
pub trait AggregateRoot: Entity + Clone {}
