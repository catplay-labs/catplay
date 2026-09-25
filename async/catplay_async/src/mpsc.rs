use futures::{Stream, StreamExt, lock::Mutex, stream::FusedStream};

use std::{
    ops::{Deref, DerefMut},
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use crate::{EventSleeper, EventToken, filling_slot};

#[doc(hidden)]
pub struct SharableReceiverInner<R>
where
    R: Stream,
{
    inner: Arc<Mutex<PeekReceiver<R>>>,
    cached: Option<R::Item>,
}

impl<R> SharableReceiverInner<R>
where
    R: Stream,
{
    pub fn peek(&self) -> Option<&R::Item> {
        self.cached.as_ref()
    }

    pub fn take(&mut self) -> Option<R::Item> {
        self.cached.take()
    }
}

impl<R> EventSleeper for SharableReceiverInner<R>
where
    R: Stream + Unpin + Send,
    R::Item: Send,
{
    async fn sleep(&mut self) -> Option<EventToken> {
        let mut recv = self.inner.lock().await;
        filling_slot(&mut self.cached, recv.inner.next())
            .sleep()
            .await
    }
}

impl<R> Clone for SharableReceiverInner<R>
where
    R: Stream,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            cached: None,
        }
    }
}

#[doc(hidden)]
pub struct PeekReceiver<R>
where
    R: Stream,
{
    inner: R,
    cached: Option<R::Item>,
}

impl<R> PeekReceiver<R>
where
    R: Stream,
{
    pub fn new(inner: R) -> Self {
        Self { inner, cached: None }
    }

    pub fn peek(&self) -> Option<&R::Item> {
        self.cached.as_ref()
    }

    pub fn take(&mut self) -> Option<R::Item> {
        self.cached.take()
    }

    pub fn sharable(mut self) -> SharableReceiverInner<R> {
        SharableReceiverInner {
            cached: self.cached.take(),
            inner: Arc::new(Mutex::new(self)),
        }
    }
}

impl<R> Default for PeekReceiver<R>
where
    R: Stream + Default,
{
    fn default() -> Self {
        Self::new(R::default())
    }
}

impl<R> EventSleeper for PeekReceiver<R>
where
    R: Stream + Unpin + Send,
    R::Item: Send,
{
    async fn sleep(&mut self) -> Option<EventToken> {
        // At this time it doesn't wake up for channel close(`None`), might do that in future if useful
        filling_slot(&mut self.cached, self.inner.next())
            .sleep()
            .await
    }
}

impl<R> Deref for PeekReceiver<R>
where
    R: Stream,
{
    type Target = R;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<R> DerefMut for PeekReceiver<R>
where
    R: Stream,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<R> Stream for PeekReceiver<R>
where
    R: Stream + Unpin,
{
    type Item = R::Item;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<R::Item>> {
        let this = self.as_mut().get_mut();

        if this.cached.is_some() {
            return Poll::Ready(this.cached.take());
        }

        Pin::new(&mut this.inner).poll_next(cx)
    }
}

impl<R> FusedStream for PeekReceiver<R>
where
    R: FusedStream + Unpin,
{
    fn is_terminated(&self) -> bool {
        self.cached.is_none() && self.inner.is_terminated()
    }
}

impl<R> Unpin for PeekReceiver<R> where R: Stream + Unpin {}
unsafe impl<R> Send for PeekReceiver<R> where R: Stream + Send {}

pub type UnboundedReceiver<T> = PeekReceiver<futures::channel::mpsc::UnboundedReceiver<T>>;
pub type Receiver<T> = PeekReceiver<futures::channel::mpsc::Receiver<T>>;
pub type Sender<T> = futures::channel::mpsc::Sender<T>;
pub type UnboundedSender<T> = futures::channel::mpsc::UnboundedSender<T>;
pub type SharableReceiver<T> = SharableReceiverInner<futures::channel::mpsc::Receiver<T>>;
pub type SharableReceiverUnbounded<T> = SharableReceiverInner<futures::channel::mpsc::UnboundedReceiver<T>>;

pub fn unbounded<T>() -> (UnboundedSender<T>, UnboundedReceiver<T>) {
    let chan = futures::channel::mpsc::unbounded();
    (chan.0, PeekReceiver::new(chan.1))
}

pub fn channel<T>(size: usize) -> (Sender<T>, Receiver<T>) {
    let chan = futures::channel::mpsc::channel(size);
    (chan.0, PeekReceiver::new(chan.1))
}
