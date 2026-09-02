use crate::AggregateRoot;
use core::future::Future;

/// Persistence for a single kind of [`AggregateRoot`], the boundary between the
/// domain and whatever actually stores it, a file, a database, a remote store.
///
/// A repository is the illusion of an in-memory collection of every aggregate of
/// its type, reached by the root's [`crate::Entity::Identity`]. It deals in whole
/// aggregates. It never exposes partial rows and never lets a caller mutate
/// stored state in place. Reading yields a fully reconstituted aggregate, and
/// saving replaces the stored aggregate wholesale and deleting removes it, both
/// keyed by the root's own identity.
///
/// The two reads here are retrieval by identity, one or many. Querying beyond
/// identity, by a [`crate::Specification`] or across the whole collection, is a
/// capability a store opts into on top, not something every repository owes.
///
/// # Every call waits and every call can be refused
///
/// What sits behind a repository is a file, a database or another machine, so a
/// call takes time and a call can come back a refusal. This trait was written
/// synchronous and infallible, which made it a shape nothing real could
/// implement, and every consumer wrote its own asynchronous fallible trait
/// beside it instead. The failure is the implementation's own, because what can
/// go wrong belongs to the thing that stores rather than to the domain that
/// asks.
///
/// The futures are `impl Future` rather than a boxed future behind a macro, so a
/// call allocates nothing and this crate keeps the no dependency property it has
/// always had. The cost is that this trait is not object safe, which no consumer
/// of it has ever needed, since every one of them dispatches statically.
///
/// `Send` is required of every future because an implementation is normally
/// driven by a work stealing runtime that may move a task between threads.
pub trait Repository<Aggregate: AggregateRoot> {
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
    fn find_by_id(
        &self,
        id: &Aggregate::Identity,
    ) -> impl Future<Output = Result<Option<Aggregate>, Self::Failure>> + Send;

    /// Reads the aggregates with these identities. Identities that store nothing
    /// are absent from the result, so it may be shorter than `ids`.
    fn find_many(
        &self,
        ids: &[Aggregate::Identity],
    ) -> impl Future<Output = Result<Vec<Aggregate>, Self::Failure>> + Send;

    /// Persists the aggregate, keyed by its own identity, replacing any previously
    /// stored value.
    fn save(&self, aggregate: &Aggregate)
    -> impl Future<Output = Result<(), Self::Failure>> + Send;

    /// Removes the aggregate with this identity, returning it if one was stored.
    fn delete(
        &self,
        id: &Aggregate::Identity,
    ) -> impl Future<Output = Result<Option<Aggregate>, Self::Failure>> + Send;
}
