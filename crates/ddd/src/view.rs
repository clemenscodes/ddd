/// A component's Published Language: the public, read-oriented contract it exposes
/// to consumers, decoupled from its private internal model (its [`crate::Model`]).
///
/// A presentation component is a small bounded context. Its `Model` is the internal
/// model — private, named only within the component. When it is consumed across a
/// boundary it publishes a `View`: the shared language its consumers build and pass,
/// so they integrate against the contract, never the internal model. A `View` that
/// is byte-identical to the `Model` today is not duplication — it is an API surface,
/// free to diverge tomorrow without breaking consumers. Where the component genuinely
/// translates the contract into a richer internal model, the split `Model:
/// From<&View>` is an [`crate::AntiCorruptionLayer`]. An intent marker for the
/// published contract type.
///
/// `Clone + PartialEq` because consumers clone the view in the `From<&View>`
/// translation and the renderer compares it across renders.
pub trait View: Clone + PartialEq {}
