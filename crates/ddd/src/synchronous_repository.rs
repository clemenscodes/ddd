use crate::AggregateRoot;

/// The synchronous alternative to [`crate::Repository`], persistence for a single
/// kind of [`AggregateRoot`] whose store answers within the call.
///
/// It keeps every contract of [`crate::Repository`]. It is the illusion of an
/// in-memory collection of every aggregate of its type, reached by the root's
/// [`crate::Entity::Identity`], it deals in whole aggregates, and saving replaces
/// the stored aggregate wholesale and deleting removes it, both keyed by the
/// root's own identity.
///
/// # Every call answers at once and every call can still be refused
///
/// Some stores never wait. Browser `localStorage` reads and writes within the
/// call, and so does a map in memory. Handing such a store the futures of
/// [`crate::Repository`] would force every caller to drive a future that is
/// always finished on its first poll, and would demand `Send` of state that has
/// no reason to cross a thread, such as a UI framework's signals. This trait is
/// that store's shape, the same four calls returning their answer directly.
///
/// Answering at once is not the same as never failing. A quota can be exhausted
/// and a store can be unavailable, so every call still returns a `Result` and the
/// failure is still the implementation's own. A store that genuinely cannot fail
/// names [`core::convert::Infallible`].
pub trait SynchronousRepository<Aggregate: AggregateRoot> {
    /// What a refusal from whatever stores these aggregates is.
    ///
    /// It belongs to the implementation rather than to this trait, because a
    /// database, a file and a remote store fail in ways that have nothing in
    /// common beyond the fact that they can.
    type Failure;

    /// Reads the aggregate with this identity, or `None` if none is stored.
    ///
    /// Nothing stored under an identity is an answer and not a failure. A
    /// failure is the store being unable to say.
    fn find_by_id(&self, id: &Aggregate::Identity) -> Result<Option<Aggregate>, Self::Failure>;

    /// Reads the aggregates with these identities. Identities that store nothing
    /// are absent from the result, so it may be shorter than `ids`.
    fn find_many(&self, ids: &[Aggregate::Identity]) -> Result<Vec<Aggregate>, Self::Failure>;

    /// Persists the aggregate, keyed by its own identity, replacing any previously
    /// stored value.
    fn save(&self, aggregate: &Aggregate) -> Result<(), Self::Failure>;

    /// Removes the aggregate with this identity, returning it if one was stored.
    fn delete(&self, id: &Aggregate::Identity) -> Result<Option<Aggregate>, Self::Failure>;
}
