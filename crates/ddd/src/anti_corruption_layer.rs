/// A translation boundary that keeps another context's model from leaking into
/// this one.
///
/// When a context must integrate with a foreign or legacy model, an
/// anti-corruption layer translates between the two languages at the seam, so the
/// foreign concepts never contaminate the local model. Translating a foreign
/// system's raw data into this system's clean domain model is an ACL's job. An
/// intent marker for the translating type.
pub trait AntiCorruptionLayer {}
