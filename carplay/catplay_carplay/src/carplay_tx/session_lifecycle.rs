use tokio::sync::watch;

pub(super) async fn wait_for_close<T: Clone>(mut closed: watch::Receiver<Option<T>>) -> Option<T> {
    loop {
        if let Some(error) = closed.borrow_and_update().clone() {
            return Some(error);
        }
        if closed.changed().await.is_err() {
            return None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn cached_close_does_not_wait_for_another_notification() {
        let (tx, rx) = watch::channel(Some(7));
        assert_eq!(
            tokio::time::timeout(Duration::from_secs(1), wait_for_close(rx))
                .await
                .unwrap(),
            Some(7)
        );
        drop(tx);
    }

    #[tokio::test]
    async fn open_session_waits_until_closed() {
        let (tx, rx) = watch::channel(None);
        let task = tokio::spawn(wait_for_close(rx));
        tokio::task::yield_now().await;
        assert!(!task.is_finished());
        tx.send_replace(Some(7));
        assert_eq!(
            tokio::time::timeout(Duration::from_secs(1), task)
                .await
                .unwrap()
                .unwrap(),
            Some(7)
        );
    }

    #[tokio::test]
    async fn disappeared_publisher_does_not_hang() {
        let (tx, rx) = watch::channel(None::<u8>);
        drop(tx);
        assert_eq!(wait_for_close(rx).await, None);
    }
}
