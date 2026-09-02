use std::collections::HashMap;
use std::convert::Infallible;
use std::future::Future;
use std::pin::pin;
use std::ptr;
use std::sync::Arc;
use std::sync::Mutex;
use std::task::Context;
use std::task::Poll;
use std::task::RawWaker;
use std::task::RawWakerVTable;
use std::task::Waker;

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
use ddd::Repository;
use ddd::Service;
use ddd::Specification;
use ddd::ValueObject;

/// Drives a future to completion by polling it in a loop with a waker that does
/// nothing at all. This crate has no dependencies and no runtime, so a test that
/// wants an answer out of a future has to do the driving itself.
///
/// It is enough here for one reason and only that reason. The repository under
/// test keeps its aggregates in a map in memory, so it never actually waits on
/// anything and every future it hands back is already finished the first time it
/// is polled. A future that genuinely waits would return `Poll::Pending` and
/// then never be woken, because this waker drops every wake on the floor, so the
/// loop would spin a core for ever. This is a test driver and nothing else.
fn block_on<FutureType>(future: FutureType) -> FutureType::Output
where
    FutureType: Future,
{
    static VTABLE: RawWakerVTable =
        RawWakerVTable::new(|_| RawWaker::new(ptr::null(), &VTABLE), |_| {}, |_| {}, |_| {});

    let raw_waker = RawWaker::new(ptr::null(), &VTABLE);

    // Safety, the vtable above only ever clones the same dangling raw waker and
    // does nothing on wake or drop, so it never touches the null data pointer.
    let waker = unsafe { Waker::from_raw(raw_waker) };
    let mut context = Context::from_waker(&waker);
    let mut future = pin!(future);

    loop {
        match future.as_mut().poll(&mut context) {
            Poll::Ready(output) => return output,
            Poll::Pending => continue,
        }
    }
}

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

// The stored map sits behind `Arc<Mutex<..>>` rather than `Rc<RefCell<..>>`,
// because a repository owes futures that are `Send` and neither `Rc` nor
// `RefCell` is.
#[derive(Clone, Debug, Default)]
struct InMemoryCounterRepository {
    stored: Arc<Mutex<HashMap<CounterId, Counter>>>,
}

impl Layered for InMemoryCounterRepository {
    type Layer = InfrastructureLayer;
}

impl Adapter for InMemoryCounterRepository {}

impl Repository<Counter> for InMemoryCounterRepository {
    // A map in memory has nothing to refuse with, so the failure of this store is
    // the one that is never constructed.
    type Failure = Infallible;

    async fn find_by_id(&self, id: &CounterId) -> Result<Option<Counter>, Self::Failure> {
        let stored = self.stored.lock().expect("the stored map was not poisoned");
        Ok(stored.get(id).copied())
    }

    async fn find_many(&self, ids: &[CounterId]) -> Result<Vec<Counter>, Self::Failure> {
        let stored = self.stored.lock().expect("the stored map was not poisoned");
        Ok(ids.iter().filter_map(|id| stored.get(id).copied()).collect())
    }

    async fn save(&self, aggregate: &Counter) -> Result<(), Self::Failure> {
        let snapshot = *aggregate;
        let mut stored = self.stored.lock().expect("the stored map was not poisoned");
        stored.insert(snapshot.id, snapshot);
        Ok(())
    }

    async fn delete(&self, id: &CounterId) -> Result<Option<Counter>, Self::Failure> {
        let mut stored = self.stored.lock().expect("the stored map was not poisoned");
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

// The live aggregate is behind `Arc<Mutex<..>>` for the same reason the stored
// map is, the service is borrowed across a wait and so has to be `Sync`.
#[derive(Clone, Debug, Default)]
struct CounterService {
    aggregate: Arc<Mutex<Counter>>,
    repository: InMemoryCounterRepository,
}

impl Layered for CounterService {
    type Layer = ApplicationLayer;
}

impl ApplicationService for CounterService {}

impl Service<Counter> for CounterService {
    type Repository = InMemoryCounterRepository;

    fn repository(&self) -> Self::Repository {
        self.repository.clone()
    }

    fn snapshot(&self) -> Counter {
        let aggregate = self.aggregate.lock().expect("the live aggregate was not poisoned");
        *aggregate
    }

    fn replace(&self, aggregate: Counter) {
        let mut current = self.aggregate.lock().expect("the live aggregate was not poisoned");
        *current = aggregate;
    }
}

fn accept_only_domain_layer<Type>()
where
    Type: Layered<Layer = DomainLayer>,
{
}

#[test]
fn dispatch_applies_command_and_writes_through_to_the_repository() {
    let service = CounterService::default();

    let increment = Increment {
        amount: 3,
    };
    let outcome = block_on(service.dispatch(increment)).expect("the store accepted the write");

    assert_eq!(outcome, 3);

    let live_aggregate = service.snapshot();
    assert_eq!(live_aggregate.value(), 3);

    let persisted_aggregate = block_on(service.repository().find_by_id(&CounterId::default()))
        .expect("the store answered");
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
    block_on(service.commit(change)).expect("the store accepted the write");

    let persisted_aggregate = block_on(service.repository().find_by_id(&CounterId::default()))
        .expect("the store answered");
    let expected_aggregate = Counter {
        id: CounterId::default(),
        value: 5,
    };
    let expected_persisted = Some(expected_aggregate);
    assert_eq!(persisted_aggregate, expected_persisted);
}

#[test]
fn a_domain_layer_bound_accepts_a_domain_type() {
    // Counter is tagged `DomainLayer`. `CounterService` (application layer) would
    // be rejected here, the compile_fail doctest on `Layered` shows why.
    accept_only_domain_layer::<Counter>();
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
struct Coordinate {
    column: u32,
    row: u32,
}

impl Layered for Coordinate {
    type Layer = DomainLayer;
}

impl ValueObject for Coordinate {}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
struct ItemId(u32);

impl Layered for ItemId {
    type Layer = DomainLayer;
}

impl ValueObject for ItemId {}

impl Identifier for ItemId {}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
struct TrackedItem {
    id: ItemId,
    value: u32,
}

impl Layered for TrackedItem {
    type Layer = DomainLayer;
}

impl Entity for TrackedItem {
    type Identity = ItemId;

    fn identity(&self) -> &Self::Identity {
        &self.id
    }
}

#[test]
fn value_objects_compare_by_value_and_entities_by_identity() {
    let first_coordinate = Coordinate {
        column: 1,
        row: 2,
    };
    let second_coordinate = Coordinate {
        column: 1,
        row: 2,
    };
    assert_eq!(first_coordinate, second_coordinate);

    let original_item = TrackedItem {
        id: ItemId(7),
        value: 81,
    };
    let changed_item = TrackedItem {
        id: ItemId(7),
        value: 87,
    };
    let original_identity = original_item.identity();
    let changed_identity = changed_item.identity();
    assert_eq!(original_identity, changed_identity);
    assert_ne!(original_item, changed_item);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
struct AtLeast {
    threshold: u32,
}

impl Layered for AtLeast {
    type Layer = DomainLayer;
}

impl Specification<Counter> for AtLeast {
    fn is_satisfied_by(&self, candidate: &Counter) -> bool {
        candidate.value() >= self.threshold
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
struct AtMost {
    ceiling: u32,
}

impl Layered for AtMost {
    type Layer = DomainLayer;
}

impl Specification<Counter> for AtMost {
    fn is_satisfied_by(&self, candidate: &Counter) -> bool {
        candidate.value() <= self.ceiling
    }
}

#[test]
fn specifications_compose_with_boolean_combinators() {
    let five = Counter {
        id: CounterId::default(),
        value: 5,
    };

    let at_least_three = AtLeast {
        threshold: 3,
    };
    let at_most_seven = AtMost {
        ceiling: 7,
    };
    let within_range = at_least_three.and(at_most_seven);
    assert!(within_range.is_satisfied_by(&five));

    let at_least_ten = AtLeast {
        threshold: 10,
    };
    let below_ten = at_least_ten.not();
    assert!(below_ten.is_satisfied_by(&five));

    let at_least_nine = AtLeast {
        threshold: 9,
    };
    let at_most_one = AtMost {
        ceiling: 1,
    };
    let at_the_extremes = at_least_nine.or(at_most_one);
    assert!(!at_the_extremes.is_satisfied_by(&five));
}

#[test]
fn repository_is_a_collection_keyed_by_identity() {
    block_on(async {
        let repository = InMemoryCounterRepository::default();

        let first = Counter {
            id: CounterId(1),
            value: 10,
        };
        let second = Counter {
            id: CounterId(2),
            value: 20,
        };
        repository.save(&first).await.expect("the store accepted the write");
        repository.save(&second).await.expect("the store accepted the write");

        let found_first = repository.find_by_id(&CounterId(1)).await.expect("the store answered");
        assert_eq!(found_first, Some(first));
        let found_third = repository.find_by_id(&CounterId(3)).await.expect("the store answered");
        assert_eq!(found_third, None);

        // find_many drops the identities that store nothing rather than padding.
        let found = repository
            .find_many(&[CounterId(1), CounterId(3), CounterId(2)])
            .await
            .expect("the store answered");
        assert_eq!(found, vec![first, second]);

        // save keyed by identity replaces the stored aggregate wholesale.
        let first_grown = Counter {
            id: CounterId(1),
            value: 11,
        };
        repository.save(&first_grown).await.expect("the store accepted the write");
        let found_grown = repository.find_by_id(&CounterId(1)).await.expect("the store answered");
        assert_eq!(found_grown, Some(first_grown));

        // delete returns what it removed, and a second delete finds nothing.
        let deleted = repository.delete(&CounterId(1)).await.expect("the store answered");
        assert_eq!(deleted, Some(first_grown));
        let deleted_again = repository.delete(&CounterId(1)).await.expect("the store answered");
        assert_eq!(deleted_again, None);
        let found_after_delete =
            repository.find_by_id(&CounterId(1)).await.expect("the store answered");
        assert_eq!(found_after_delete, None);
    });
}
