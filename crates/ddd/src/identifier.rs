use crate::ValueObject;

/// The value object that serves as an [`crate::Entity`]'s identity.
///
/// An identifier is itself a value object, immutable, compared by value, whose
/// job is to say which entity this is, for example an `OrderId` or a `UserId`.
/// Modelling identity as a distinct type, rather than a bare `u32`, is what stops
/// one entity's id from being silently used where another's is expected.
///
/// `Hash` is added on top of [`ValueObject`], so an identity can always key the
/// collection a [`crate::Repository`] presents, without every id type having to
/// remember to allow it.
pub trait Identifier: ValueObject + std::hash::Hash {}
