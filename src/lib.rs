#![no_std]

pub mod prelude;

pub enum Async<T> {
    NotReady,
    Ready(T),
}

pub trait Future {
    type Item;

    fn poll(&mut self) -> Async<Self::Item>;

    // TODO adaptors

    fn wait(mut self) -> Self::Item
        where Self: Sized
    {
        loop {
            if let Async::Ready(item) = self.poll() {
                return item;
            }
        }
    }
}

pub trait InfiniteIterator {
    type Item;

    fn next(&mut self) -> Self::Item;

    // TODO adaptors
}

pub trait InfiniteStream {
    type Item;

    fn poll(&mut self) -> Async<Self::Item>;

    // TODO adaptors

    fn wait(self) -> InfiniteWait<Self>
        where Self: Sized
    {
        InfiniteWait { stream: self }
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

pub trait Stream {
    type Item;

    fn poll(&mut self) -> Async<Option<Self::Item>>;

    // TODO adaptors

    fn wait(self) -> Wait<Self>
        where Self: Sized
    {
        Wait { stream: self }
    }
}

pub struct Wait<S> {
    stream: S,
}

impl<S> Iterator for Wait<S>
    where S: Stream
{
    type Item = S::Item;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Async::Ready(item) = self.stream.poll() {
                return item;
            }
        }
    }
}
