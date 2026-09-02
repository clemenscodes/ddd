use core::future::Future;

/// A transactional boundary, a set of changes that commit together, all or
/// nothing.
///
/// A unit of work batches mutations and flushes them once, so intermediate steps
/// never persist and a failure leaves nothing half-written. This is already
/// supplied by [`crate::Service::commit`], which mutates a snapshot and saves
/// once, the n=1 case over a single aggregate. This trait is the shape it takes
/// when a real multi-aggregate transaction appears, several repositories
/// coordinated to succeed or roll back as one.
///
/// # Committing waits and can be refused
///
/// What a commit ends is a transaction against a database or another store, so
/// it takes time and it can come back a refusal. A signature that says otherwise
/// forces every implementation to lie about what it does, either by blocking a
/// thread inside a call that claims not to wait or by swallowing a refusal it
/// has no way to report. The failure is the implementation's own, because what
/// can go wrong belongs to the thing that stores rather than to the domain that
/// asks.
///
/// The future is `impl Future` rather than a boxed future behind a macro, so a
/// commit allocates nothing and this crate keeps the no dependency property it
/// has always had. `Send` is required because a commit is normally driven by a
/// work stealing runtime that may move a task between threads.
pub trait UnitOfWork {
    /// What a refusal from whatever the changes commit against is.
    ///
    /// It belongs to the implementation rather than to this trait, because a
    /// database, a file and a remote store fail in ways that have nothing in
    /// common beyond the fact that they can. The word is the same one
    /// [`crate::Repository`] uses, so this crate has one name for a refusal
    /// rather than two.
    type Failure;

    /// Commits every tracked change atomically, consuming the unit of work.
    fn commit(self) -> impl Future<Output = Result<(), Self::Failure>> + Send;
}
