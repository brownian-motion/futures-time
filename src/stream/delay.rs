use core::future::Future;
use core::pin::Pin;
use core::time::Duration;

use pin_project_lite::pin_project;

use async_io::Timer;
use core::task::{Context, Poll};
use futures_core::stream::Stream;

pin_project! {
    /// Delay execution of a stream once for the specified duration.
    #[derive(Debug)]
    #[must_use = "streams do nothing unless polled or .awaited"]
    pub struct Delay<S> {
        #[pin]
        stream: S,
        #[pin]
        timer: Timer,
        state: State,
    }
}

#[derive(Debug)]
enum State {
    Timer,
    Streaming,
}

impl<S> Delay<S> {
    pub(super) fn new(stream: S, dur: Duration) -> Self {
        Delay {
            stream,
            timer: Timer::after(dur),
            state: State::Timer,
        }
    }
}

impl<S> Stream for Delay<S>
where
    S: Stream,
{
    type Item = S::Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();

        match this.state {
            State::Timer => match this.timer.poll(cx) {
                Poll::Pending => return Poll::Pending,
                Poll::Ready(_) => {
                    *this.state = State::Streaming;
                    this.stream.poll_next(cx)
                }
            },
            State::Streaming => this.stream.poll_next(cx),
        }
    }
}
