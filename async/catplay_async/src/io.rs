use std::{
    fs::{File, OpenOptions},
    io::{self, Read, Seek, SeekFrom},
    ops::{BitOr, BitOrAssign},
    os::{fd::AsRawFd, unix::fs::OpenOptionsExt},
    path::Path,
};

use tokio::io::{Interest as TokioInterest, Ready as TokioReady, unix::AsyncFd};

use crate::{EventReconciler, EventSleeper, EventToken};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Interest(u8);

impl Interest {
    pub const READABLE: Self = Self(0b0001);
    pub const WRITABLE: Self = Self(0b0010);
    pub const PRIORITY: Self = Self(0b0100);
    pub const ERROR: Self = Self(0b1000);

    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    pub const fn is_readable(self) -> bool {
        self.0 & Self::READABLE.0 != 0
    }

    pub const fn is_writable(self) -> bool {
        self.0 & Self::WRITABLE.0 != 0
    }

    pub const fn is_priority(self) -> bool {
        self.0 & Self::PRIORITY.0 != 0
    }

    pub const fn is_error(self) -> bool {
        self.0 & Self::ERROR.0 != 0
    }

    pub const fn add(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    pub fn remove(self, other: Self) -> Option<Self> {
        let interest = Self(self.0 & !other.0);
        (!interest.is_empty()).then_some(interest)
    }

    fn to_tokio(self) -> io::Result<TokioInterest> {
        let mut interest = None;

        if self.is_readable() {
            interest = Some(TokioInterest::READABLE);
        }

        if self.is_writable() {
            interest = Some(match interest {
                Some(interest) => interest | TokioInterest::WRITABLE,
                None => TokioInterest::WRITABLE,
            });
        }

        if self.is_priority() {
            interest = Some(match interest {
                Some(interest) => interest | TokioInterest::PRIORITY,
                None => TokioInterest::PRIORITY,
            });
        }

        if self.is_error() {
            interest = Some(match interest {
                Some(interest) => interest | TokioInterest::ERROR,
                None => TokioInterest::ERROR,
            });
        }

        interest.ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "empty async I/O interest"))
    }

    fn ready_mask(self) -> Ready {
        let mut ready = Ready::EMPTY;
        if self.is_readable() {
            ready |= Ready::READABLE | Ready::READ_CLOSED;
        }
        if self.is_writable() {
            ready |= Ready::WRITABLE | Ready::WRITE_CLOSED;
        }
        if self.is_priority() {
            ready |= Ready::PRIORITY | Ready::READ_CLOSED;
        }
        if self.is_error() {
            ready |= Ready::ERROR;
        }
        ready
    }
}

impl BitOr for Interest {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        self.add(rhs)
    }
}

impl BitOrAssign for Interest {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = self.add(rhs);
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Ready(u8);

impl Ready {
    pub const EMPTY: Self = Self(0);
    pub const READABLE: Self = Self(0b0000_0001);
    pub const WRITABLE: Self = Self(0b0000_0010);
    pub const READ_CLOSED: Self = Self(0b0000_0100);
    pub const WRITE_CLOSED: Self = Self(0b0000_1000);
    pub const PRIORITY: Self = Self(0b0001_0000);
    pub const ERROR: Self = Self(0b0010_0000);

    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    pub const fn is_readable(self) -> bool {
        self.contains(Self::READABLE) || self.is_read_closed()
    }

    pub const fn is_writable(self) -> bool {
        self.contains(Self::WRITABLE) || self.is_write_closed()
    }

    pub const fn is_read_closed(self) -> bool {
        self.contains(Self::READ_CLOSED)
    }

    pub const fn is_write_closed(self) -> bool {
        self.contains(Self::WRITE_CLOSED)
    }

    pub const fn is_priority(self) -> bool {
        self.contains(Self::PRIORITY)
    }

    pub const fn is_error(self) -> bool {
        self.contains(Self::ERROR)
    }

    fn from_tokio(ready: TokioReady) -> Self {
        let mut out = Self::EMPTY;

        if ready.is_readable() {
            out |= Self::READABLE;
        }
        if ready.is_writable() {
            out |= Self::WRITABLE;
        }
        if ready.is_read_closed() {
            out |= Self::READ_CLOSED;
        }
        if ready.is_write_closed() {
            out |= Self::WRITE_CLOSED;
        }
        if ready.is_priority() {
            out |= Self::PRIORITY;
        }
        if ready.is_error() {
            out |= Self::ERROR;
        }

        out
    }

    fn to_tokio(self) -> TokioReady {
        let mut ready = TokioReady::EMPTY;

        if self.contains(Self::READABLE) {
            ready |= TokioReady::READABLE;
        }
        if self.contains(Self::WRITABLE) {
            ready |= TokioReady::WRITABLE;
        }
        if self.contains(Self::READ_CLOSED) {
            ready |= TokioReady::READ_CLOSED;
        }
        if self.contains(Self::WRITE_CLOSED) {
            ready |= TokioReady::WRITE_CLOSED;
        }
        if self.contains(Self::PRIORITY) {
            ready |= TokioReady::PRIORITY;
        }
        if self.contains(Self::ERROR) {
            ready |= TokioReady::ERROR;
        }

        ready
    }
}

impl BitOr for Ready {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for Ready {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl std::ops::Sub for Ready {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 & !rhs.0)
    }
}

pub struct AsyncIo<T: AsRawFd> {
    inner: Option<AsyncFd<T>>,
    interest: Interest,
    pending_ready: Ready,
    error: Option<io::Error>,
    token: EventToken,
}

impl<T: AsRawFd> AsyncIo<T> {
    pub fn new(inner: T, interest: Interest) -> io::Result<Self> {
        Ok(Self {
            inner: Some(AsyncFd::with_interest(inner, interest.to_tokio()?)?),
            interest,
            pending_ready: Ready::EMPTY,
            error: None,
            token: EventToken(1),
        })
    }

    pub fn readable(inner: T) -> io::Result<Self> {
        Self::new(inner, Interest::READABLE)
    }

    pub fn read_write(inner: T) -> io::Result<Self> {
        Self::new(inner, Interest::READABLE | Interest::WRITABLE)
    }

    pub fn priority(inner: T) -> io::Result<Self> {
        Self::new(inner, Interest::PRIORITY)
    }

    pub fn with_token(mut self, token: EventToken) -> Self {
        self.token = token;
        self
    }

    pub fn interest(&self) -> Interest {
        self.interest
    }

    pub fn pending_ready(&self) -> Ready {
        self.pending_ready
    }

    pub fn get_ref(&self) -> &T {
        self.inner
            .as_ref()
            .expect("async I/O backend is missing")
            .get_ref()
    }

    pub fn get_mut(&mut self) -> &mut T {
        self.inner
            .as_mut()
            .expect("async I/O backend is missing")
            .get_mut()
    }

    pub fn into_inner(self) -> T {
        self.inner
            .expect("async I/O backend is missing")
            .into_inner()
    }

    pub fn set_interest(&mut self, interest: Interest) -> io::Result<()> {
        let tokio_interest = interest.to_tokio()?;
        let inner = self.take_inner()?;
        let replacement = match AsyncFd::try_with_interest(inner, tokio_interest) {
            Ok(replacement) => replacement,
            Err(err) => {
                let (inner, cause) = err.into_parts();
                self.inner = Some(AsyncFd::with_interest(inner, self.interest.to_tokio()?)?);
                return Err(cause);
            }
        };
        self.inner = Some(replacement);
        self.interest = interest;
        self.pending_ready = Ready::EMPTY;
        Ok(())
    }

    pub fn enable_interest(&mut self, interest: Interest) -> io::Result<()> {
        self.set_interest(self.interest | interest)
    }

    pub fn disable_interest(&mut self, interest: Interest) -> io::Result<()> {
        let Some(interest) = self.interest.remove(interest) else {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "async I/O interest cannot be empty"));
        };
        self.set_interest(interest)
    }

    pub async fn wait_ready(&mut self) -> io::Result<Ready> {
        let guard = self
            .inner
            .as_mut()
            .expect("async I/O backend is missing")
            .ready_mut(self.interest.to_tokio()?)
            .await?;
        let ready = Ready::from_tokio(guard.ready());
        drop(guard);

        self.pending_ready |= ready;
        Ok(ready)
    }

    pub async fn clear_ready(&mut self, ready: Ready) -> io::Result<()> {
        let mut guard = self
            .inner
            .as_mut()
            .expect("async I/O backend is missing")
            .ready_mut(self.interest.to_tokio()?)
            .await?;
        guard.clear_ready_matching(ready.to_tokio());
        self.pending_ready = self.pending_ready - ready;
        Ok(())
    }

    pub async fn clear_all_ready(&mut self) -> io::Result<()> {
        let mut guard = self
            .inner
            .as_mut()
            .expect("async I/O backend is missing")
            .ready_mut(self.interest.to_tokio()?)
            .await?;
        guard.clear_ready();
        self.pending_ready = Ready::EMPTY;
        Ok(())
    }

    pub fn try_io<R>(&mut self, interest: Interest, f: impl FnOnce(&mut T) -> io::Result<R>) -> io::Result<R> {
        let result = self
            .inner
            .as_mut()
            .expect("async I/O backend is missing")
            .try_io_mut(interest.to_tokio()?, f);
        if result
            .as_ref()
            .is_err_and(|err| err.kind() == io::ErrorKind::WouldBlock)
        {
            self.pending_ready = self.pending_ready - interest.ready_mask();
        }
        result
    }

    fn take_inner(&mut self) -> io::Result<T> {
        self.inner
            .take()
            .map(AsyncFd::into_inner)
            .ok_or_else(|| io::Error::other("async I/O backend is missing"))
    }
}

impl<T: AsRawFd + Send> EventSleeper for AsyncIo<T> {
    async fn sleep(&mut self) -> Option<EventToken> {
        match self.wait_ready().await {
            Ok(_) => Some(EventToken(self.token.0)),
            Err(err) => {
                self.error = Some(err);
                Some(EventToken(self.token.0))
            }
        }
    }
}

impl<T: AsRawFd + Send> EventReconciler for AsyncIo<T> {
    type Error = io::Error;

    async fn reconcile(&mut self) -> Result<(), Self::Error> {
        if let Some(err) = self.error.take() {
            return Err(err);
        }
        Ok(())
    }
}

pub struct SysfsNotify {
    inner: AsyncIo<File>,
    buf: Box<[u8]>,
}

impl SysfsNotify {
    pub fn open(path: impl AsRef<Path>) -> io::Result<Self> {
        Self::open_with_token(path, EventToken(1))
    }

    pub fn open_with_token(path: impl AsRef<Path>, token: EventToken) -> io::Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NONBLOCK | libc::O_CLOEXEC)
            .open(path)?;

        let mut notify = Self {
            inner: AsyncIo::new(file, Interest::PRIORITY | Interest::ERROR)?.with_token(token),
            buf: vec![0u8; 128].into_boxed_slice(),
        };
        notify.consume()?;
        Ok(notify)
    }

    pub fn io(&self) -> &AsyncIo<File> {
        &self.inner
    }

    pub fn io_mut(&mut self) -> &mut AsyncIo<File> {
        &mut self.inner
    }

    fn consume(&mut self) -> io::Result<()> {
        self.inner.get_mut().seek(SeekFrom::Start(0))?;

        loop {
            match self.inner.get_mut().read(&mut self.buf) {
                Ok(_) => return Ok(()),
                Err(err) if err.kind() == io::ErrorKind::Interrupted => continue,
                Err(err) => return Err(err),
            }
        }
    }
}

impl EventSleeper for SysfsNotify {
    async fn sleep(&mut self) -> Option<EventToken> {
        self.inner.sleep().await
    }
}

impl EventReconciler for SysfsNotify {
    type Error = io::Error;

    async fn reconcile(&mut self) -> Result<(), Self::Error> {
        self.inner.reconcile().await?;
        self.consume()
    }
}
