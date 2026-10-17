use crate::AggregateRoot;
use crate::ApplicationService;
use crate::Command;
use crate::SynchronousRepository;

/// What a refusal from the store behind a [`SynchronousService`] is, the
/// [`SynchronousRepository`]'s own [`SynchronousRepository::Failure`] spelled out
/// once so the signatures below stay readable.
type Refusal<ServiceType, Aggregate> =
    <<ServiceType as SynchronousService<Aggregate>>::Repository as SynchronousRepository<
        Aggregate,
    >>::Failure;

/// The synchronous alternative to [`crate::Service`], the application layer
/// orchestrator for one [`AggregateRoot`] whose [`SynchronousRepository`] answers
/// within the call.
///
/// It keeps every contract of [`crate::Service`]. It holds the live aggregate,
/// owns its repository, and is the sole place mutations happen. The only ways to
/// change the aggregate are [`SynchronousService::commit`] (for named methods)
/// and [`SynchronousService::dispatch`] (for reified [`Command`]s), and both
/// write through to the repository in the same call, so stored state can never
/// drift from the live aggregate. Implementors provide only the three
/// primitives, [`SynchronousService::repository`],
/// [`SynchronousService::snapshot`], and [`SynchronousService::replace`].
///
/// Because nothing here waits, the write is finished before the call returns,
/// and nothing is asked to be `Send` or `Sync`. That is what lets a service hold
/// state a UI framework keeps on one thread and still guarantee that the store
/// has the mutation by the time the caller regains control.
pub trait SynchronousService<Aggregate: AggregateRoot>: ApplicationService {
    /// The concrete repository this service persists through.
    type Repository: SynchronousRepository<Aggregate>;

    /// Returns a handle to the repository backing this service.
    fn repository(&self) -> Self::Repository;

    /// Returns a clone of the current live aggregate.
    fn snapshot(&self) -> Aggregate;

    /// Overwrites the live aggregate with a new value.
    fn replace(&self, aggregate: Aggregate);

    /// Runs a mutation against the aggregate and persists the result in the same
    /// call, the single write through path every mutation funnels through.
    ///
    /// The order is change, then save, then replace, for the same reason as
    /// [`crate::Service::commit`]. **A save that is refused leaves the live
    /// aggregate untouched**, so on a refusal the caller sees the failure and the
    /// service is still the state the store last accepted.
    fn commit<Outcome>(
        &self,
        change: impl FnOnce(&mut Aggregate) -> Outcome,
    ) -> Result<Outcome, Refusal<Self, Aggregate>> {
        let mut aggregate = self.snapshot();
        let outcome = change(&mut aggregate);
        let repository = self.repository();
        repository.save(&aggregate)?;
        self.replace(aggregate);
        Ok(outcome)
    }

    /// Executes a named [`Command`] against the aggregate and persists the
    /// result. Sugar over [`SynchronousService::commit`] for the reified command
    /// path, so it keeps the same rule, a refused save leaves the live aggregate
    /// alone.
    fn dispatch<CommandType>(
        &self,
        command: CommandType,
    ) -> Result<CommandType::Outcome, Refusal<Self, Aggregate>>
    where
        CommandType: Command<Aggregate>,
    {
        self.commit(|aggregate| command.execute(aggregate))
    }
}
