use std::sync::{
    Arc, Mutex,
    atomic::{AtomicUsize, Ordering},
};

use catplay_async::EventReconciler;

#[derive(Clone, Debug, PartialEq, Eq)]
struct ChildError;

#[derive(Debug, PartialEq, Eq)]
struct RawChildError;

impl From<RawChildError> for ChildError {
    fn from(_: RawChildError) -> Self {
        ChildError
    }
}

#[derive(Debug, PartialEq, Eq)]
enum ParentError {
    Child(ChildError),
}

impl From<ChildError> for ParentError {
    fn from(value: ChildError) -> Self {
        ParentError::Child(value)
    }
}

struct ChildReconciler {
    calls: Arc<AtomicUsize>,
}

impl EventReconciler for ChildReconciler {
    type Error = ChildError;

    async fn reconcile(&mut self) -> Result<(), Self::Error> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}

struct SameErrorReconciler {
    calls: Arc<AtomicUsize>,
}

impl EventReconciler for SameErrorReconciler {
    type Error = ParentError;

    async fn reconcile(&mut self) -> Result<(), Self::Error> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}

#[derive(EventReconciler)]
#[event_reconciler(crate = "catplay_async")]
#[reconcile_error(ParentError)]
#[reconcile_func(reconcile_self)]
struct DerivedReconciler {
    func_calls: Arc<AtomicUsize>,

    #[reconcile(ParentError::Child)]
    child: ChildReconciler,

    #[reconcile]
    same_error: SameErrorReconciler,
}

impl DerivedReconciler {
    async fn reconcile_self(&mut self) -> Result<(), ParentError> {
        self.func_calls.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}

#[tokio::test]
async fn derived_event_reconciler_runs_all_steps() {
    let func_calls = Arc::new(AtomicUsize::new(0));
    let child_calls = Arc::new(AtomicUsize::new(0));
    let same_calls = Arc::new(AtomicUsize::new(0));

    let mut reconciler = DerivedReconciler {
        func_calls: Arc::clone(&func_calls),
        child: ChildReconciler {
            calls: Arc::clone(&child_calls),
        },
        same_error: SameErrorReconciler {
            calls: Arc::clone(&same_calls),
        },
    };

    reconciler
        .reconcile()
        .await
        .expect("reconcile should succeed");

    assert_eq!(func_calls.load(Ordering::SeqCst), 1);
    assert_eq!(child_calls.load(Ordering::SeqCst), 1);
    assert_eq!(same_calls.load(Ordering::SeqCst), 1);
}

struct OrderedReconciler {
    order: Arc<Mutex<Vec<&'static str>>>,
    name: &'static str,
}

impl EventReconciler for OrderedReconciler {
    type Error = ParentError;

    async fn reconcile(&mut self) -> Result<(), Self::Error> {
        self.order
            .lock()
            .expect("order lock should not be poisoned")
            .push(self.name);
        Ok(())
    }
}

#[tokio::test]
async fn reconcile_funcs_run_after_field_reconcilers() {
    #[derive(EventReconciler)]
    #[event_reconciler(crate = "catplay_async")]
    #[reconcile_error(ParentError)]
    #[reconcile_func(reconcile_self)]
    struct OrderedParent {
        order: Arc<Mutex<Vec<&'static str>>>,

        #[reconcile]
        first: OrderedReconciler,

        #[reconcile]
        second: OrderedReconciler,
    }

    impl OrderedParent {
        async fn reconcile_self(&mut self) -> Result<(), ParentError> {
            self.order
                .lock()
                .expect("order lock should not be poisoned")
                .push("func");
            Ok(())
        }
    }

    let order = Arc::new(Mutex::new(Vec::new()));
    let mut parent = OrderedParent {
        order: Arc::clone(&order),
        first: OrderedReconciler {
            order: Arc::clone(&order),
            name: "first",
        },
        second: OrderedReconciler {
            order: Arc::clone(&order),
            name: "second",
        },
    };

    parent.reconcile().await.expect("reconcile should succeed");

    let order = order
        .lock()
        .expect("order lock should not be poisoned")
        .clone();
    assert_eq!(order, ["first", "second", "func"]);
}

#[tokio::test]
async fn mapped_reconcile_error_is_returned() {
    struct FailingChild;

    impl EventReconciler for FailingChild {
        type Error = RawChildError;

        async fn reconcile(&mut self) -> Result<(), Self::Error> {
            Err(RawChildError)
        }
    }

    #[derive(EventReconciler)]
    #[event_reconciler(crate = "catplay_async")]
    #[reconcile_error(ParentError)]
    struct FailingParent {
        #[reconcile(ParentError::Child)]
        child: FailingChild,
    }

    let err = FailingParent { child: FailingChild }
        .reconcile()
        .await
        .expect_err("reconcile should fail");

    assert_eq!(err, ParentError::Child(ChildError));
}

#[tokio::test]
async fn reconcile_pop_returns_and_clears_pending_error() {
    #[derive(EventReconciler)]
    #[event_reconciler(crate = "catplay_async")]
    #[reconcile_error(ParentError)]
    struct PopParent {
        #[reconcile_pop]
        pending: Option<ChildError>,
    }

    let mut parent = PopParent { pending: Some(ChildError) };

    let err = parent
        .reconcile()
        .await
        .expect_err("pending error should be returned");

    assert_eq!(err, ParentError::Child(ChildError));
    assert_eq!(parent.pending, None);
}

#[tokio::test]
async fn reconcile_pop_clone_returns_and_keeps_pending_error() {
    #[derive(EventReconciler)]
    #[event_reconciler(crate = "catplay_async")]
    #[reconcile_error(ParentError)]
    struct PopParent {
        #[reconcile_pop(clone)]
        pending: Option<ChildError>,
    }

    let mut parent = PopParent { pending: Some(ChildError) };

    let err = parent
        .reconcile()
        .await
        .expect_err("pending error should be returned");

    assert_eq!(err, ParentError::Child(ChildError));
    assert_eq!(parent.pending, Some(ChildError));
}
