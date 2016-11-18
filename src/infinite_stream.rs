use core::mem;

use {Async, InfiniteIterator, Future};

pub trait InfiniteStream {
    type Item;

    fn and_then<B, F>(self, f: F) -> AndThen<Self, F, B>
        where B: Future,
              F: FnMut(Self::Item) -> B,
              Self: Sized
    {
        AndThen {
            f: f,
            future: None,
            stream: self,
        }
    }

    fn map<B, F>(self, f: F) -> Map<Self, F>
        where F: FnMut(Self::Item) -> B,
              Self: Sized
    {
        Map {
            stream: self,
            f: f,
        }
    }

    fn poll(&mut self) -> Async<Self::Item>;

    fn wait(self) -> InfiniteWait<Self>
        where Self: Sized
    {
        InfiniteWait { stream: self }
    }
}

#[must_use = "futures do nothing unless polled"]
pub struct AndThen<S, F, B>
    where S: InfiniteStream,
          F: FnMut(S::Item) -> B
{
    f: F,
    future: Option<B>,
    stream: S,
}

impl<S, F, B> InfiniteStream for AndThen<S, F, B>
    where S: InfiniteStream,
          F: FnMut(S::Item) -> B,
          B: Future
{
    type Item = B::Item;

    fn poll(&mut self) -> Async<B::Item> {
        if let Some(mut future) = mem::replace(&mut self.future, None) {
            match future.poll() {
                Async::NotReady => {
                    self.future = Some(future);
                    Async::NotReady
                }
                Async::Ready(t) => Async::Ready(t),
            }
        } else {
            match self.stream.poll() {
                Async::NotReady => Async::NotReady,
                Async::Ready(t) => {
                    self.future = Some((self.f)(t));
                    self.poll()
                }
            }
        }
    }
}

pub struct InfiniteWait<S> {
    stream: S,
}

impl<S> InfiniteIterator for InfiniteWait<S>
    where S: InfiniteStream
{
    type Item = S::Item;

    fn next(&mut self) -> Self::Item {
        loop {
            if let Async::Ready(item) = self.stream.poll() {
                return item;
            }
        }
    }
}

#[must_use = "futures do nothing unless polled"]
pub struct Map<S, F> {
    stream: S,
    f: F,
}

impl<S, F, B> InfiniteStream for Map<S, F>
    where S: InfiniteStream,
          F: FnMut(S::Item) -> B
{
    type Item = B;

    fn poll(&mut self) -> Async<Self::Item> {
        self.stream.poll().map(|t| (self.f)(t))
    }
}
