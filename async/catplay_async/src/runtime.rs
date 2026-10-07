use std::{
    pin::Pin,
    task::{Context, Poll},
};

// API change vs Tokio: we default to AbortOnDropHandle with option to detach on demand.

pub struct AbortOnDropHandle<T>(tokio_util::task::AbortOnDropHandle<T>);
pub struct JoinHandle<T>(tokio::task::JoinHandle<T>);
#[derive(Clone, Debug)]
pub struct Handle(tokio::runtime::Handle);
#[derive(Clone, Debug)]
pub struct AbortHandle(tokio::task::AbortHandle);

impl<T> AbortOnDropHandle<T> {
    /// Abort the task associated with this handle,
    /// equivalent to [`JoinHandle::abort`].
    pub fn abort(&self) {
        self.0.abort()
    }

    /// Checks if the task associated with this handle is finished,
    /// equivalent to [`JoinHandle::is_finished`].
    pub fn is_finished(&self) -> bool {
        self.0.is_finished()
    }

    /// Returns a new [`AbortHandle`] that can be used to remotely abort this task,
    /// equivalent to [`JoinHandle::abort_handle`].
    pub fn abort_handle(&self) -> AbortHandle {
        AbortHandle(self.0.abort_handle())
    }

    /// Cancels aborting on drop and returns the original [`JoinHandle`].
    pub fn detach(self) -> JoinHandle<T> {
        JoinHandle(self.0.detach())
    }
}

impl<T> Future for AbortOnDropHandle<T> {
    type Output = Result<T, tokio::task::JoinError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.0).poll(cx)
    }
}

impl<T> JoinHandle<T> {
    /// Abort the task associated with this handle.
    pub fn abort(&self) {
        self.0.abort()
    }

    /// Checks if the task associated with this handle is finished.
    pub fn is_finished(&self) -> bool {
        self.0.is_finished()
    }

    /// Returns a new [`AbortHandle`] that can be used to remotely abort this task.
    pub fn abort_handle(&self) -> AbortHandle {
        AbortHandle(self.0.abort_handle())
    }

    /// Returns the unique identifier of the task associated with this handle.
    pub fn id(&self) -> tokio::task::Id {
        self.0.id()
    }
}

impl<T> Future for JoinHandle<T> {
    type Output = Result<T, tokio::task::JoinError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.0).poll(cx)
    }
}

impl AbortHandle {
    /// Abort the task associated with this handle.
    pub fn abort(&self) {
        self.0.abort()
    }

    /// Checks if the task associated with this handle is finished.
    pub fn is_finished(&self) -> bool {
        self.0.is_finished()
    }

    /// Returns the unique identifier of the task associated with this handle.
    pub fn id(&self) -> tokio::task::Id {
        self.0.id()
    }
}

impl Handle {
    /// Spawns tasks on this event loop.
    #[must_use = "aborts on drop"]
    pub fn spawn<F>(&self, future: F) -> AbortOnDropHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        AbortOnDropHandle(tokio_util::task::AbortOnDropHandle::new(self.0.spawn(future)))
    }
}

/// Spawns tasks inside current event loop, panics if not inside event loop context.
#[must_use = "aborts on drop"]
pub fn spawn<F>(future: F) -> AbortOnDropHandle<F::Output>
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    AbortOnDropHandle(tokio_util::task::AbortOnDropHandle::new(tokio::spawn(future)))
}

#[must_use = "aborts on drop"]
pub fn spawn_blocking<F, R>(f: F) -> AbortOnDropHandle<R>
where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
{
    AbortOnDropHandle(tokio_util::task::AbortOnDropHandle::new(tokio::task::spawn_blocking(f)))
}

pub mod handle {
    use super::Handle;

    /// Returns a handle to the current event loop.
    pub fn current() -> Handle {
        Handle(tokio::runtime::Handle::current())
    }
}

/// Runs a constructor in a new single-threaded Tokio runtime and returns its result.
///
/// The callback and its result cross a thread boundary and must be `Send`. Create
/// runtime-bound resources inside the callback so their I/O and spawned tasks use
/// the fork's event loop. Later calls made by the caller still use the caller's
/// runtime; only calls executed on the fork inherit its context.
///
/// This call blocks until the callback returns; the callback should only bootstrap
/// the object, not wait for work on the caller's runtime. Every call creates a new
/// named thread and runtime. The thread exits when its spawned tasks finish.
///
/// # Panics
///
/// Panics immediately if there is no current Tokio runtime, or if the thread or
/// runtime cannot be created. A callback panic is resumed on the calling thread.
pub fn spawn_fork<F, R>(thread_name: &str, f: F) -> R
where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
{
    // Check the provider before creating a thread or invoking user code.
    let _ = tokio::runtime::Handle::current();
    let idle = std::sync::Arc::new(tokio::sync::Notify::new());
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .on_thread_park({
            let idle = idle.clone();
            move || {
                if tokio::runtime::Handle::current()
                    .metrics()
                    .num_alive_tasks()
                    == 0
                {
                    idle.notify_one();
                }
            }
        })
        .build()
        .expect("failed to create fork runtime");
    let (sender, receiver) = std::sync::mpsc::sync_channel(1);
    std::thread::Builder::new()
        .name(thread_name.to_owned())
        .spawn(move || {
            runtime.block_on(async move {
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(f));
                let _ = sender.send(result);
                idle.notified().await;
            });
        })
        .expect("failed to create fork thread");

    match receiver
        .recv()
        .expect("fork thread exited before returning its result")
    {
        Ok(result) => result,
        Err(panic) => std::panic::resume_unwind(panic),
    }
}

#[cfg(test)]
mod tests {
    use super::{spawn, spawn_fork};
    use std::{thread, time::Duration};

    #[test]
    #[should_panic(expected = "no reactor running")]
    fn fork_requires_runtime_context() {
        spawn_fork("catplay-fork", || ());
    }

    #[tokio::test]
    async fn fork_returns_object_and_drives_its_tasks_and_io() {
        let caller = thread::current().id();
        let (listener, task, fork_thread) = spawn_fork("catplay-fork", || {
            assert_eq!(thread::current().name(), Some("catplay-fork"));
            assert_eq!(
                tokio::runtime::Handle::current().runtime_flavor(),
                tokio::runtime::RuntimeFlavor::CurrentThread,
            );
            let listener = tokio::net::TcpListener::from_std({
                let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
                listener.set_nonblocking(true).unwrap();
                listener
            })
            .unwrap();
            let address = listener.local_addr().unwrap();
            let task = spawn(async move {
                let (_stream, _) = listener.accept().await.unwrap();
                tokio::time::sleep(Duration::from_millis(1)).await;
                spawn(async { thread::current().id() }).await.unwrap()
            });
            (address, task, thread::current().id())
        });
        assert_ne!(caller, fork_thread);
        let _stream = tokio::net::TcpStream::connect(listener).await.unwrap();
        assert_eq!(
            tokio::time::timeout(Duration::from_secs(5), task)
                .await
                .unwrap()
                .unwrap(),
            fork_thread,
        );
    }

    #[tokio::test]
    async fn forks_with_the_same_name_get_distinct_threads() {
        let (release, wait) = tokio::sync::oneshot::channel();
        let first = spawn_fork("catplay-fork", move || {
            spawn(async move {
                let _ = wait.await;
            })
            .detach();
            thread::current().id()
        });
        let second = spawn_fork("catplay-fork", || thread::current().id());
        assert_ne!(first, second);
        release.send(()).unwrap();
    }

    #[tokio::test]
    async fn fork_exits_after_a_direct_tokio_task_finishes() {
        struct ExitNotify(std::cell::RefCell<Option<std::sync::mpsc::Sender<()>>>);

        impl Drop for ExitNotify {
            fn drop(&mut self) {
                if let Some(sender) = self.0.get_mut().take() {
                    let _ = sender.send(());
                }
            }
        }

        thread_local! {
            static EXIT: ExitNotify = ExitNotify(std::cell::RefCell::new(None));
        }

        let (release, wait) = tokio::sync::oneshot::channel::<()>();
        let (exited, thread_exit) = std::sync::mpsc::channel();
        spawn_fork("catplay-fork-exit", move || {
            EXIT.with(|exit| *exit.0.borrow_mut() = Some(exited));
            tokio::spawn(async move {
                let _ = wait.await;
            });
        });
        assert!(thread_exit.try_recv().is_err());
        release.send(()).unwrap();
        thread_exit.recv_timeout(Duration::from_secs(5)).unwrap();
    }

    #[tokio::test]
    async fn different_names_use_different_threads() {
        let first = spawn_fork("catplay-fork-a", || {
            (thread::current().id(), thread::current().name().unwrap().to_owned())
        });
        let second = spawn_fork("catplay-fork-b", || {
            (thread::current().id(), thread::current().name().unwrap().to_owned())
        });
        assert_ne!(first.0, second.0);
        assert_eq!(first.1, "catplay-fork-a");
        assert_eq!(second.1, "catplay-fork-b");
    }

    #[test]
    fn fork_survives_caller_runtime_and_return_value_drop() {
        let (release, wait) = tokio::sync::oneshot::channel();
        let (done, completed) = std::sync::mpsc::channel();
        let runtime = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap();
        runtime.block_on(async {
            drop(spawn_fork("catplay-fork", move || {
                spawn(async move {
                    wait.await.unwrap();
                    done.send(()).unwrap();
                })
                .detach()
            }));
        });
        drop(runtime);
        release.send(()).unwrap();
        completed.recv_timeout(Duration::from_secs(5)).unwrap();
    }

    #[tokio::test]
    async fn fork_propagates_callback_panic() {
        let panic = std::panic::catch_unwind(|| spawn_fork("catplay-fork", || panic!("fork bootstrap failed"))).unwrap_err();
        assert_eq!(panic.downcast_ref::<&str>(), Some(&"fork bootstrap failed"));
    }
}
