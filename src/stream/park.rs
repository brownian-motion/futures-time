use std::pin::Pin;
use std::task::{Context, Poll};

use crate::channel::{Parker, Receiver};

use futures_core::{ready, Stream};
use pin_project_lite::pin_project;

pin_project! {
    /// Suspend or resume execution of a stream.
    ///
    /// This `struct` is created by the [`park`] method on [`StreamExt`]. See its
    /// documentation for more.
    ///
    /// [`park`]: crate::future::FutureExt::park
    /// [`StreamExt`]: crate::future::StreamExt
    #[must_use = "futures do nothing unless polled or .awaited"]
    pub struct Park<S> {
        #[pin]
        stream: S,
        #[pin]
        receiver: Receiver<Parker>,
        state: State,
    }
}

/// The internal state
#[derive(Debug)]
enum State {
    /// Actively polling the future.
    Active,
    /// The future has been paused, so we wait for a signal from the channel.
    Suspended,
    /// The channel has been dropped, no more need to check it!
    NoChannel,
    /// The future has completed.
    Completed,
}

impl<S> Park<S> {
    pub(super) fn new(stream: S, receiver: Receiver<Parker>) -> Self {
        Self {
            stream,
            receiver,
            state: State::Suspended,
        }
    }
}

impl<S: Stream> Stream for Park<S> {
    type Item = S::Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();
        loop {
            match this.state {
                State::Suspended => match ready!(this.receiver.as_mut().poll_next(cx)) {
                    Some(Parker::Park) => return Poll::Pending,
                    Some(Parker::Unpark) => *this.state = State::Active,
                    None => *this.state = State::NoChannel,
                },
                State::Active => {
                    if let Poll::Ready(Some(Parker::Park)) = this.receiver.as_mut().poll_next(cx) {
                        *this.state = State::Suspended;
                        return Poll::Pending;
                    }
                    match ready!(this.stream.as_mut().poll_next(cx)) {
                        Some(value) => return Poll::Ready(Some(value)),
                        None => {
                            *this.state = State::Completed;
                            return Poll::Ready(None);
                        }
                    }
                }
                State::NoChannel => match ready!(this.stream.as_mut().poll_next(cx)) {
                    Some(value) => return Poll::Ready(Some(value)),
                    None => {
                        *this.state = State::Completed;
                        return Poll::Ready(None);
                    }
                },
                State::Completed => panic!("future polled after completing"),
            }
        }
    }
}

// NOTE(yosh): we should probably test this, but I'm too tired today lol.
