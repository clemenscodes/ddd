use std::cell::RefCell;
use std::collections::HashMap;
use std::convert::Infallible;
use std::rc::Rc;

use ddd::Adapter;
use ddd::AggregateRoot;
use ddd::ApplicationLayer;
use ddd::ApplicationService;
use ddd::Command;
use ddd::DomainLayer;
use ddd::Entity;
use ddd::Identifier;
use ddd::InfrastructureLayer;
use ddd::Layered;
use ddd::SynchronousRepository;
use ddd::SynchronousService;
use ddd::ValueObject;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
struct CounterId(u32);

impl Layered for CounterId {
    type Layer = DomainLayer;
}

impl ValueObject for CounterId {}

impl Identifier for CounterId {}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
struct Counter {
    id: CounterId,
    value: u32,
}

impl Counter {
    fn value(&self) -> u32 {
        self.value
    }

    fn set_value(&mut self, value: u32) {
        self.value = value;
    }
}

impl Layered for Counter {
    type Layer = DomainLayer;
}

impl Entity for Counter {
    type Identity = CounterId;

    fn identity(&self) -> &Self::Identity {
        &self.id
    }
}

impl AggregateRoot for Counter {}

// The stored map sits behind `Rc<RefCell<..>>`, which is neither `Send` nor
// `Sync`. A synchronous repository is never driven across a thread, so it owes
// neither, and this store proves the trait asks for neither.
#[derive(Clone, Debug, Default)]
struct InMemoryCounterRepository {
    stored: Rc<RefCell<HashMap<CounterId, Counter>>>,
}

impl Layered for InMemoryCounterRepository {
    type Layer = InfrastructureLayer;
}

impl Adapter for InMemoryCounterRepository {}

impl SynchronousRepository<Counter> for InMemoryCounterRepository {
    // A map in memory has nothing to refuse with, so the failure of this store is
    // the one that is never constructed.
    type Failure = Infallible;

    fn find_by_id(&self, id: &CounterId) -> Result<Option<Counter>, Self::Failure> {
        let stored = self.stored.borrow();
        Ok(stored.get(id).copied())
    }

    fn find_many(&self, ids: &[CounterId]) -> Result<Vec<Counter>, Self::Failure> {
        let stored = self.stored.borrow();
        Ok(ids.iter().filter_map(|id| stored.get(id).copied()).collect())
    }

    fn save(&self, aggregate: &Counter) -> Result<(), Self::Failure> {
        let snapshot = *aggregate;
        let mut stored = self.stored.borrow_mut();
        stored.insert(snapshot.id, snapshot);
        Ok(())
    }

    fn delete(&self, id: &CounterId) -> Result<Option<Counter>, Self::Failure> {
        let mut stored = self.stored.borrow_mut();
        Ok(stored.remove(id))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
struct Increment {
    amount: u32,
}

impl Layered for Increment {
    type Layer = ApplicationLayer;
}

impl Command<Counter> for Increment {
    type Outcome = u32;

    fn execute(self, aggregate: &mut Counter) -> u32 {
        let current_value = aggregate.value();
        let incremented_value = current_value + self.amount;
        aggregate.set_value(incremented_value);
        aggregate.value()
    }
}

// The live aggregate is behind `Rc<RefCell<..>>` too, a synchronous service is
// never borrowed across a wait and so has no reason to be `Sync`.
#[derive(Clone, Debug, Default)]
struct CounterService {
    aggregate: Rc<RefCell<Counter>>,
    repository: InMemoryCounterRepository,
}

impl Layered for CounterService {
    type Layer = ApplicationLayer;
}

impl ApplicationService for CounterService {}

impl SynchronousService<Counter> for CounterService {
    type Repository = InMemoryCounterRepository;

    fn repository(&self) -> Self::Repository {
        self.repository.clone()
    }

    fn snapshot(&self) -> Counter {
        *self.aggregate.borrow()
    }

    fn replace(&self, aggregate: Counter) {
        *self.aggregate.borrow_mut() = aggregate;
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
struct StoreUnavailable;

// A store that refuses every write, to prove a refused save leaves the live
// aggregate as the store last accepted it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
struct RefusingCounterRepository;

impl Layered for RefusingCounterRepository {
    type Layer = InfrastructureLayer;
}

impl Adapter for RefusingCounterRepository {}

impl SynchronousRepository<Counter> for RefusingCounterRepository {
    type Failure = StoreUnavailable;

    fn find_by_id(&self, _id: &CounterId) -> Result<Option<Counter>, Self::Failure> {
        Err(StoreUnavailable)
    }

    fn find_many(&self, _ids: &[CounterId]) -> Result<Vec<Counter>, Self::Failure> {
        Err(StoreUnavailable)
    }

    fn save(&self, _aggregate: &Counter) -> Result<(), Self::Failure> {
        Err(StoreUnavailable)
    }

    fn delete(&self, _id: &CounterId) -> Result<Option<Counter>, Self::Failure> {
        Err(StoreUnavailable)
    }
}

#[derive(Clone, Debug, Default)]
struct RefusedCounterService {
    aggregate: Rc<RefCell<Counter>>,
}

impl Layered for RefusedCounterService {
    type Layer = ApplicationLayer;
}

impl ApplicationService for RefusedCounterService {}

impl SynchronousService<Counter> for RefusedCounterService {
    type Repository = RefusingCounterRepository;

    fn repository(&self) -> Self::Repository {
        RefusingCounterRepository
    }

    fn snapshot(&self) -> Counter {
        *self.aggregate.borrow()
    }

    fn replace(&self, aggregate: Counter) {
        *self.aggregate.borrow_mut() = aggregate;
    }
}

#[test]
fn dispatch_applies_command_and_writes_through_to_the_repository() {
    let service = CounterService::default();

    let increment = Increment {
        amount: 3,
    };
    let Ok(outcome) = service.dispatch(increment);

    assert_eq!(outcome, 3);

    let live_aggregate = service.snapshot();
    assert_eq!(live_aggregate.value(), 3);

    let Ok(persisted_aggregate) = service.repository().find_by_id(&CounterId::default());
    let expected_aggregate = Counter {
        id: CounterId::default(),
        value: 3,
    };
    let expected_persisted = Some(expected_aggregate);
    assert_eq!(persisted_aggregate, expected_persisted);
}

#[test]
fn commit_is_the_write_through_path_for_named_mutations() {
    let service = CounterService::default();

    let change = |counter: &mut Counter| {
        let recomputed_value = counter.value() + 5;
        counter.set_value(recomputed_value);
    };
    let Ok(()) = service.commit(change);

    let Ok(persisted_aggregate) = service.repository().find_by_id(&CounterId::default());
    let expected_aggregate = Counter {
        id: CounterId::default(),
        value: 5,
    };
    let expected_persisted = Some(expected_aggregate);
    assert_eq!(persisted_aggregate, expected_persisted);
}

#[test]
fn a_refused_save_leaves_the_live_aggregate_untouched() {
    let service = RefusedCounterService::default();

    let increment = Increment {
        amount: 3,
    };
    let refusal = service.dispatch(increment);

    assert_eq!(refusal, Err(StoreUnavailable));
    let live_aggregate = service.snapshot();
    assert_eq!(live_aggregate.value(), 0);
}

#[test]
fn repository_is_a_collection_keyed_by_identity() {
    let repository = InMemoryCounterRepository::default();

    let first = Counter {
        id: CounterId(1),
        value: 10,
    };
    let second = Counter {
        id: CounterId(2),
        value: 20,
    };
    let Ok(()) = repository.save(&first);
    let Ok(()) = repository.save(&second);

    let Ok(found_first) = repository.find_by_id(&CounterId(1));
    assert_eq!(found_first, Some(first));
    let Ok(found_third) = repository.find_by_id(&CounterId(3));
    assert_eq!(found_third, None);

    // find_many drops the identities that store nothing rather than padding.
    let Ok(found) = repository.find_many(&[CounterId(1), CounterId(3), CounterId(2)]);
    assert_eq!(found, vec![first, second]);

    // save keyed by identity replaces the stored aggregate wholesale.
    let first_grown = Counter {
        id: CounterId(1),
        value: 11,
    };
    let Ok(()) = repository.save(&first_grown);
    let Ok(found_grown) = repository.find_by_id(&CounterId(1));
    assert_eq!(found_grown, Some(first_grown));

    // delete returns what it removed, and a second delete finds nothing.
    let Ok(deleted) = repository.delete(&CounterId(1));
    assert_eq!(deleted, Some(first_grown));
    let Ok(deleted_again) = repository.delete(&CounterId(1));
    assert_eq!(deleted_again, None);
    let Ok(found_after_delete) = repository.find_by_id(&CounterId(1));
    assert_eq!(found_after_delete, None);
}
