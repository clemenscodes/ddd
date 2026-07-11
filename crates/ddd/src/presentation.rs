use crate::Model;

/// A component's signal-free, render-ready presentation: the shaped data its body
/// places into the RSX, one render-tick snapshot with no reactive `Signal` inside.
///
/// Built from the component's [`Model`] — purely (`From<&Model>`) for a shaping leaf,
/// or effectfully (owning the local signals and context reads) for a connected
/// component. The associated `Model` wires the pipeline `View -> Model -> Presentation`
/// at the type level (`Self::Model::View` is navigable); there is no `From` supertrait
/// because the effectful build is not a pure conversion.
pub trait Presentation {
    /// The internal model this presentation is built from.
    type Model: Model;
}
