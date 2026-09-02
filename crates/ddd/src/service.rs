use crate::AggregateRoot;
use crate::ApplicationService;
use crate::Command;
use crate::Repository;
use core::future::Future;

/// What a refusal from the store behind a [`Service`] is, the [`Repository`]'s
/// own [`Repository::Failure`] spelled out once so the signatures below stay
/// readable.
type Refusal<ServiceType, Aggregate> =
    <<ServiceType as Service<Aggregate>>::Repository as Repository<Aggregate>>::Failure;

/// The application layer orchestrator for one [`AggregateRoot`]. It holds the
/// live aggregate, owns its [`Repository`], and is the sole place mutations
/// happen.
///
/// A service exposes no way to change the aggregate except [`Service::commit`]
/// (for named methods) or [`Service::dispatch`] (for reified [`Command`]s). Both
/// paths write through to the repository in the same call, so stored state can
/// never drift from the live aggregate. Implementors provide only the three
/// primitives, [`Service::repository`], [`Service::snapshot`], and
/// [`Service::replace`]. The write through discipline is supplied here and
/// cannot be bypassed.
///
/// The [`ApplicationService`] bound pins the layer the same way every other
/// functional role in this crate pins its own, so a domain or infrastructure
/// type can never be a service. Reads by identity are the query side's job, the
/// service owns the repository only as the write-through sink for the one
/// aggregate it holds.
///
/// # Writing through waits and can be refused
///
/// A [`Repository`] takes time and can come back a refusal, so the two write
/// paths here take time and can come back a refusal too. The failure is the
/// repository's own, because a service adds no way to fail beyond the store it
/// writes to.
///
/// The futures are `impl Future` rather than boxed ones, so a call allocates
/// nothing and this crate keeps the no dependency property it has always had.
/// The bounds on the two write paths are only what a future that survives being
/// moved between threads needs, the service borrowed across the wait, so
/// `Self: Sync`, and every value the wait steps over, so the repository handle,
/// the aggregate, the mutation and its outcome are all `Send`. Nothing here asks
/// for `Aggregate: Sync`, because the aggregate is only ever lent to the
/// repository's own future, which the [`Repository`] trait already declares
/// `Send`.
pub trait Service<Aggregate: AggregateRoot>: ApplicationService {
    /// The concrete repository this service persists through.
    type Repository: Repository<Aggregate>;

    /// Returns a handle to the repository backing this service.
    fn repository(&self) -> Self::Repository;

    /// Returns a clone of the current live aggregate.
    fn snapshot(&self) -> Aggregate;

    /// Overwrites the live aggregate with a new value.
    fn replace(&self, aggregate: Aggregate);

    /// Runs a mutation against the aggregate and persists the result in the same
    /// call, the single write through path every mutation funnels through.
    ///
    /// The order is change, then save, then replace, and it is that order for one
    /// reason. **A save that is refused leaves the live aggregate untouched.**
    /// Replacing first and saving after would mean a refused write leaves a live
    /// aggregate that nothing stored ever agreed to, which is exactly the drift
    /// this trait exists to prevent. On a refusal the caller sees the failure and
    /// the service is still the state the store last accepted.
    fn commit<Outcome>(
        &self,
        change: impl FnOnce(&mut Aggregate) -> Outcome + Send,
    ) -> impl Future<Output = Result<Outcome, Refusal<Self, Aggregate>>> + Send
    where
        Self: Sync,
        Self::Repository: Send,
        Aggregate: Send,
        Outcome: Send,
    {
        async move {
            let mut aggregate = self.snapshot();
            let outcome = change(&mut aggregate);
            let repository = self.repository();
            repository.save(&aggregate).await?;
            self.replace(aggregate);
            Ok(outcome)
        }
    }

    /// Executes a named [`Command`] against the aggregate and persists the
    /// result. Sugar over [`Service::commit`] for the reified command path, so it
    /// keeps the same rule, a refused save leaves the live aggregate alone.
    fn dispatch<CommandType>(
        &self,
        command: CommandType,
    ) -> impl Future<Output = Result<CommandType::Outcome, Refusal<Self, Aggregate>>> + Send
    where
        Self: Sync,
        Self::Repository: Send,
        Aggregate: Send,
        CommandType: Command<Aggregate> + Send,
        CommandType::Outcome: Send,
    {
        self.commit(|aggregate| command.execute(aggregate))
    }
}
