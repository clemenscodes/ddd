use crate::DomainLayer;
use crate::Layered;

/// Encapsulates the creation of a complex domain object so it can never be
/// constructed in an invalid state.
///
/// When building an [`crate::AggregateRoot`] or [`crate::Entity`] takes more than
/// a struct literal, defaults to materialise, invariants to check, several
/// inputs to reconcile, that assembly belongs in a factory.
pub trait Factory<Product: Layered<Layer = DomainLayer>>: Layered<Layer = DomainLayer> {
    /// The inputs the factory needs to produce a valid product. Left open, the
    /// inputs may be raw values from outside the domain.
    type Blueprint;

    /// Produces a fully valid product from its blueprint.
    fn create(&self, blueprint: Self::Blueprint) -> Product;
}
