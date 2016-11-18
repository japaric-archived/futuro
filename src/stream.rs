use Async;

pub trait Stream {
    type Item;

    fn poll(&mut self) -> Async<Option<Self::Item>>;

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
