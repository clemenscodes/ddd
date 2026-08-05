use crate::DomainLayer;
use crate::Identifier;
use crate::Layered;

/// A domain object defined by a stable identity rather than by its attributes.
///
/// Two entities are the same entity when their [`Entity::Identity`] is equal,
/// even if every other field differs. Changing an attribute while keeping the
/// same identity still refers to the same entity. This is the contrast with a
/// [`crate::ValueObject`], which has no identity and is compared by value. Nothing
/// at the type level stops one type being both, unlike the four mutually exclusive
/// layers, so keeping entity and value object apart stays a modelling discipline.
///
/// `Identity` is an [`Identifier`], so an id is always a value object in its own
/// right and one entity's id cannot be silently used where another's is
/// expected. The price is that every entity wraps its identity in a named type
/// rather than reaching for a bare integer.
pub trait Entity: Layered<Layer = DomainLayer> {
    /// The type that uniquely identifies this entity within its aggregate.
    type Identity: Identifier;

    /// Returns the stable identity of this entity.
    fn identity(&self) -> &Self::Identity;

    /// Whether this and `other` are the same entity, that is, share an identity,
    /// no matter how their other attributes differ. This is what "compared by
    /// identity" means, made callable rather than left for each caller to spell
    /// out.
    fn is_same_entity(&self, other: &Self) -> bool {
        self.identity() == other.identity()
    }
}
