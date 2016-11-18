#![no_std]

pub mod prelude;

mod infinite_stream;
mod stream;

use core::mem;

pub enum Async<T> {
    NotReady,
    Ready(T),
}

impl<T> Async<T> {
    pub fn map<F, U>(self, f: F) -> Async<U>
        where F: FnOnce(T) -> U
    {
        match self {
            Async::NotReady => Async::NotReady,
            Async::Ready(t) => Async::Ready(f(t)),
        }
    }
}

pub trait Future {
    type Item;

    fn poll(&mut self) -> Async<Self::Item>;

    fn and_then<F, B>(self, f: F) -> AndThen<Self, B, F>
        where F: FnOnce(Self::Item) -> B,
              B: Future<Item = Self::Item>,
              Self: Sized
    {
        AndThen::First(self, f)
    }

    fn fuse(self) -> Fuse<Self>
        where Self: Sized
    {
        Fuse { future: Some(self) }
    }

    fn join<B>(self, other: B) -> Join<Self, B>
        where B: Future,
              Self: Sized
    {
        Join::BothRunning(self, other)
    }

    fn map<F, T>(self, f: F) -> Map<Self, F>
        where F: FnOnce(Self::Item) -> T,
              Self: Sized
    {
        Map {
            f: Some(f),
            future: self,
        }
    }

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

#[must_use = "futures do nothing unless polled"]
pub enum AndThen<A, B, F> {
    First(A, F),
    Second(B),
    Done,
}

impl<A, B, F> Future for AndThen<A, B, F>
    where A: Future,
          F: FnOnce(A::Item) -> B,
          B: Future<Item = A::Item>
{
    type Item = A::Item;

    fn poll(&mut self) -> Async<A::Item> {
        let state = mem::replace(self, AndThen::Done);

        let mut b = match state {
            AndThen::First(mut a, f) => {
                match a.poll() {
                    Async::NotReady => {
                        *self = AndThen::First(a, f);
                        return Async::NotReady;
                    }
                    Async::Ready(a) => f(a),
                }
            }
            AndThen::Second(b) => b,
            AndThen::Done => panic!("cannot poll `and_then` twice"),
        };

        match b.poll() {
            Async::NotReady => {
                *self = AndThen::Second(b);
                Async::NotReady
            }
            Async::Ready(c) => {
                *self = AndThen::Done;
                Async::Ready(c)
            }
        }
    }
}

#[must_use = "futures do nothing unless polled"]
pub struct Fuse<A> {
    future: Option<A>,
}

impl<A> Future for Fuse<A>
    where A: Future
{
    type Item = A::Item;

    fn poll(&mut self) -> Async<A::Item> {
        match self.future.as_mut().map(|f| f.poll()) {
            Some(Async::Ready(a)) => {
                self.future = None;
                Async::Ready(a)
            }
            _ => Async::NotReady,
        }
    }
}

#[must_use = "futures do nothing unless polled"]
pub enum Join<A, B>
    where A: Future,
          B: Future
{
    BothRunning(A, B),
    Done,
    FirstDone(A::Item, B),
    SecondDone(A, B::Item),
}

impl<A, B> Future for Join<A, B>
    where A: Future,
          B: Future
{
    type Item = (A::Item, B::Item);

    fn poll(&mut self) -> Async<Self::Item> {
        let state = mem::replace(self, Join::Done);

        match state {
            Join::BothRunning(mut a, mut b) => {
                match (a.poll(), b.poll()) {
                    (Async::NotReady, Async::NotReady) => Async::NotReady,
                    (Async::NotReady, Async::Ready(b)) => {
                        *self = Join::SecondDone(a, b);
                        Async::NotReady
                    }
                    (Async::Ready(a), Async::NotReady) => {
                        *self = Join::FirstDone(a, b);
                        Async::NotReady
                    }
                    (Async::Ready(a), Async::Ready(b)) => Async::Ready((a, b)),

                }
            }
            Join::Done => panic!("cannot poll `join` twice"),
            Join::FirstDone(a, mut b) => {
                match b.poll() {
                    Async::NotReady => {
                        *self = Join::FirstDone(a, b);
                        Async::NotReady
                    }
                    Async::Ready(b) => Async::Ready((a, b)),
                }
            }
            Join::SecondDone(mut a, b) => {
                match a.poll() {
                    Async::NotReady => {
                        *self = Join::SecondDone(a, b);
                        Async::NotReady
                    }
                    Async::Ready(a) => Async::Ready((a, b)),

                }
            }
        }
    }
}

#[must_use = "futures do nothing unless polled"]
pub struct Map<A, F> {
    future: A,
    f: Option<F>,
}

impl<A, F, B> Future for Map<A, F>
    where A: Future,
          F: FnOnce(A::Item) -> B
{
    type Item = B;

    fn poll(&mut self) -> Async<B> {
        let f = self.f.take().expect("cannot poll `map` twice");

        match self.future.poll() {
            Async::Ready(t) => Async::Ready(f(t)),
            Async::NotReady => {
                self.f = Some(f);
                Async::NotReady
            }
        }
    }
}

pub trait InfiniteIterator {
    type Item;

    fn next(&mut self) -> Self::Item;
}
