use crate::DomainLayer;
use crate::Layered;

/// Stateless domain logic that belongs to no single [`crate::Entity`] or
/// [`crate::ValueObject`] because it operates across several of them.
///
/// Logic that answers a domain question spanning many objects and owns no state
/// of its own, for example checking a rule across many records or resolving a
/// cascading effect. Contrast with a [`crate::Service`], which lives in the
/// application layer and handles orchestration and persistence but no rules, a
/// domain service is the rule.
///
/// An intent marker that forces only its layer. Because it requires the domain
/// layer and [`crate::ApplicationService`] requires the application layer, one
/// type can never be both, the two layers are mutually exclusive.
pub trait DomainService: Layered<Layer = DomainLayer> {}
