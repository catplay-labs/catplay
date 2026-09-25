use futures::{
    Future, Stream, StreamExt,
    stream::{FusedStream, FuturesOrdered as InnerFuturesOrdered, FuturesUnordered as InnerFuturesUnordered},
};

use std::{
    ops::{Deref, DerefMut},
    pin::Pin,
    task::{Context, Poll},
};

use crate::{EventSleeper, EventToken, filling_slot};

pub struct PeekFutures<R>
where
    R: Stream,
{
    inner: R,
    cached: Option<R::Item>,
}

impl<R> PeekFutures<R>
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
}

impl<R> Default for PeekFutures<R>
where
    R: Stream + Default,
{
    fn default() -> Self {
        Self::new(R::default())
    }
}

impl<R> EventSleeper for PeekFutures<R>
where
    R: Stream + Unpin + Send,
    R::Item: Send,
{
    async fn sleep(&mut self) -> Option<EventToken> {
        filling_slot(&mut self.cached, self.inner.next())
            .sleep()
            .await
    }
}

impl<R> Deref for PeekFutures<R>
where
    R: Stream,
{
    type Target = R;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<R> DerefMut for PeekFutures<R>
where
    R: Stream,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<R> Stream for PeekFutures<R>
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

impl<R> FusedStream for PeekFutures<R>
where
    R: FusedStream + Unpin,
{
    fn is_terminated(&self) -> bool {
        self.cached.is_none() && self.inner.is_terminated()
    }
}

impl<R> Unpin for PeekFutures<R> where R: Stream + Unpin {}
unsafe impl<R> Send for PeekFutures<R> where R: Stream + Send {}

pub type FuturesOrdered<Fut> = PeekFutures<InnerFuturesOrdered<Fut>>;
pub type FuturesUnordered<Fut> = PeekFutures<InnerFuturesUnordered<Fut>>;

pub fn ordered<Fut>() -> FuturesOrdered<Fut>
where
    Fut: Future,
{
    PeekFutures::new(InnerFuturesOrdered::new())
}

pub fn unordered<Fut>() -> FuturesUnordered<Fut>
where
    Fut: Future,
{
    PeekFutures::new(InnerFuturesUnordered::new())
}

#[cfg(test)]
mod tests {
    use futures::future;

    use crate::{EventSleeper, EventToken};

    #[tokio::test]
    async fn ordered_sleep_peek_take_result() {
        let mut futures = super::ordered();

        futures.push_back(future::ready(1));
        futures.push_back(future::ready(2));

        assert_eq!(futures.sleep().await, Some(EventToken(1)));
        assert_eq!(futures.peek(), Some(&1));
        assert_eq!(futures.take(), Some(1));
        assert_eq!(futures::StreamExt::next(&mut futures).await, Some(2));
    }

    #[tokio::test]
    async fn unordered_sleep_peek_take_result() {
        let mut futures = super::unordered();

        futures.push(future::ready(3));

        assert_eq!(futures.sleep().await, Some(EventToken(1)));
        assert_eq!(futures.peek(), Some(&3));
        assert_eq!(futures.take(), Some(3));
        assert_eq!(futures.sleep().await, None);
    }
}
