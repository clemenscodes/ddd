use crate::AggregateRoot;

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
pub trait Repository<Aggregate: AggregateRoot> {
    /// Reads the aggregate with this identity, or `None` if none is stored.
    fn find_by_id(&self, id: &Aggregate::Identity) -> Option<Aggregate>;

    /// Reads the aggregates with these identities. Identities that store nothing
    /// are absent from the result, so it may be shorter than `ids`.
    fn find_many(&self, ids: &[Aggregate::Identity]) -> Vec<Aggregate>;

    /// Persists the aggregate, keyed by its own identity, replacing any previously
    /// stored value.
    fn save(&self, aggregate: &Aggregate);

    /// Removes the aggregate with this identity, returning it if one was stored.
    fn delete(&self, id: &Aggregate::Identity) -> Option<Aggregate>;
}
