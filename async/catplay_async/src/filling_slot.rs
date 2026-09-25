use core::{future::Future, marker::PhantomData};

use crate::{EventSleeper, EventToken};

pub struct FillingSlot<'a, T, Fut, F, U> {
    slot: &'a mut Option<T>,
    fut: Option<Fut>,
    map: Option<F>,
    _output: PhantomData<fn() -> U>,
}

#[allow(clippy::type_complexity)]
pub fn filling_slot<T, Fut>(slot: &mut Option<T>, fut: Fut) -> FillingSlot<'_, T, Fut, fn(Option<T>) -> Option<T>, Option<T>>
where
    Fut: Future<Output = Option<T>>,
{
    fn identity<T>(value: Option<T>) -> Option<T> {
        value
    }

    FillingSlot {
        slot,
        fut: Some(fut),
        map: Some(identity::<T>),
        _output: PhantomData,
    }
}

#[allow(clippy::type_complexity)]
pub fn filling_slot_value<T, Fut>(slot: &mut Option<T>, fut: Fut) -> FillingSlot<'_, T, Fut, fn(T) -> Option<T>, T>
where
    Fut: Future<Output = T>,
{
    fn some<T>(value: T) -> Option<T> {
        Some(value)
    }

    FillingSlot {
        slot,
        fut: Some(fut),
        map: Some(some::<T>),
        _output: PhantomData,
    }
}

pub fn filling_slot_map<T, Fut, F, U>(slot: &mut Option<T>, fut: Fut, map: F) -> FillingSlot<'_, T, Fut, F, U>
where
    Fut: Future<Output = U>,
    F: FnOnce(U) -> Option<T>,
{
    FillingSlot {
        slot,
        fut: Some(fut),
        map: Some(map),
        _output: PhantomData,
    }
}

impl<T, Fut, F, U> EventSleeper for FillingSlot<'_, T, Fut, F, U>
where
    T: Send,
    Fut: Future<Output = U> + Send,
    F: FnOnce(U) -> Option<T> + Send,
{
    #[inline(always)]
    async fn sleep(&mut self) -> Option<EventToken> {
        if self.slot.is_some() {
            self.fut.take();
            return None;
        }

        let fut = self.fut.take()?;
        let map = self.map.take()?;
        let value = map(fut.await)?;

        self.slot.replace(value);
        Some(EventToken(1))
    }
}

#[cfg(test)]
mod tests {
    use crate::{EventSleeper, EventToken, filling_slot, filling_slot_map, filling_slot_value};

    #[tokio::test]
    async fn filling_slot_sets_empty_slot() {
        let mut slot = None;
        let token = filling_slot(&mut slot, async { Some(7) }).sleep().await;

        assert_eq!(token, Some(EventToken(1)));
        assert_eq!(slot, Some(7));
    }

    #[tokio::test]
    async fn filling_slot_value_sets_empty_slot() {
        let mut slot = None;
        let token = filling_slot_value(&mut slot, async { 7 }).sleep().await;

        assert_eq!(token, Some(EventToken(1)));
        assert_eq!(slot, Some(7));
    }

    #[tokio::test]
    async fn filling_slot_map_filters_output() {
        let mut slot = None;
        let token = filling_slot_map(&mut slot, async { Ok::<_, i32>(()) }, Result::err)
            .sleep()
            .await;

        assert_eq!(token, None);
        assert_eq!(slot, None);

        let token = filling_slot_map(&mut slot, async { Err::<(), _>(9) }, Result::err)
            .sleep()
            .await;

        assert_eq!(token, Some(EventToken(1)));
        assert_eq!(slot, Some(9));
    }

    #[tokio::test]
    async fn filling_slot_with_full_slot_returns_none() {
        let mut slot = Some(3);
        let mut sleeper = filling_slot(&mut slot, async { Some(7) });

        assert_eq!(sleeper.sleep().await, None);

        drop(sleeper);
        assert_eq!(slot, Some(3));
    }

    #[tokio::test]
    async fn filling_slot_no_wakeup_when_callback_returns_none() {
        let mut slot: Option<()> = None;
        let mut sleeper = filling_slot(&mut slot, async { None });

        assert_eq!(sleeper.sleep().await, None);

        drop(sleeper);
        assert_eq!(slot, None);
    }
}
