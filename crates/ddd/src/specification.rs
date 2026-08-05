use crate::DomainLayer;
use crate::Layered;

/// A named, composable predicate that answers "does this candidate satisfy this
/// piece of domain truth?" as a first class value.
///
/// Instead of scattering a raw boolean expression across the code, name it, for
/// example `Active.and(Overdue)`, and reuse it for filtering, validation, and
/// expressing rules declaratively. A list view and a report are natural homes, each
/// is a specification over the same candidate. The default combinators build the
/// boolean algebra.
pub trait Specification<Candidate>: Layered<Layer = DomainLayer> {
    /// Returns whether the candidate satisfies this specification.
    fn is_satisfied_by(&self, candidate: &Candidate) -> bool;

    /// A specification satisfied only when both this and `other` are.
    fn and<Other>(self, other: Other) -> And<Self, Other>
    where
        Self: Sized,
        Other: Specification<Candidate>,
    {
        And {
            left: self,
            right: other,
        }
    }

    /// A specification satisfied when either this or `other` is.
    fn or<Other>(self, other: Other) -> Or<Self, Other>
    where
        Self: Sized,
        Other: Specification<Candidate>,
    {
        Or {
            left: self,
            right: other,
        }
    }

    /// A specification satisfied exactly when this one is not.
    fn not(self) -> Not<Self>
    where
        Self: Sized,
    {
        Not {
            inner: self,
        }
    }
}

/// The conjunction of two specifications. Built by [`Specification::and`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct And<Left, Right> {
    left: Left,
    right: Right,
}

/// The disjunction of two specifications. Built by [`Specification::or`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct Or<Left, Right> {
    left: Left,
    right: Right,
}

/// The negation of a specification. Built by [`Specification::not`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct Not<Inner> {
    inner: Inner,
}

// The combinators carry the domain-layer marker only when the specifications they
// wrap do, so a nonsense composite over non-domain types is not certified domain.
impl<Left, Right> Layered for And<Left, Right>
where
    Left: Layered<Layer = DomainLayer>,
    Right: Layered<Layer = DomainLayer>,
{
    type Layer = DomainLayer;
}

impl<Left, Right> Layered for Or<Left, Right>
where
    Left: Layered<Layer = DomainLayer>,
    Right: Layered<Layer = DomainLayer>,
{
    type Layer = DomainLayer;
}

impl<Inner> Layered for Not<Inner>
where
    Inner: Layered<Layer = DomainLayer>,
{
    type Layer = DomainLayer;
}

impl<Candidate, Left, Right> Specification<Candidate> for And<Left, Right>
where
    Left: Specification<Candidate>,
    Right: Specification<Candidate>,
{
    fn is_satisfied_by(&self, candidate: &Candidate) -> bool {
        let left_holds = self.left.is_satisfied_by(candidate);
        let right_holds = self.right.is_satisfied_by(candidate);
        left_holds && right_holds
    }
}

impl<Candidate, Left, Right> Specification<Candidate> for Or<Left, Right>
where
    Left: Specification<Candidate>,
    Right: Specification<Candidate>,
{
    fn is_satisfied_by(&self, candidate: &Candidate) -> bool {
        let left_holds = self.left.is_satisfied_by(candidate);
        let right_holds = self.right.is_satisfied_by(candidate);
        left_holds || right_holds
    }
}

impl<Candidate, Inner> Specification<Candidate> for Not<Inner>
where
    Inner: Specification<Candidate>,
{
    fn is_satisfied_by(&self, candidate: &Candidate) -> bool {
        let inner_holds = self.inner.is_satisfied_by(candidate);
        !inner_holds
    }
}
