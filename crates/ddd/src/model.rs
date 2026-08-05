use crate::Layered;
use crate::PresentationLayer;
use crate::View;

/// A component's private internal model, built from its published [`View`].
///
/// A type cannot be `Model` without being `From<&View>`, so every internal model
/// publishes a contract, and the `From` conversion is the translation at the
/// component's boundary (an [`crate::AntiCorruptionLayer`] when the model is richer
/// than the contract). Consumers integrate against the `View`. The `Model` stays
/// private. `Clone + PartialEq` because the model is the component's renderer props.
/// This is what forces "no model without a view". The [`PresentationLayer`] bound
/// places it, with its [`crate::Presentation`], in the presentation tier.
pub trait Model:
    for<'view> From<&'view Self::View> + Clone + PartialEq + Layered<Layer = PresentationLayer>
{
    /// The published contract this internal model is built from.
    type View: View;
}
