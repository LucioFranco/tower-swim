use futures::{Async, Future, Poll, Stream};
use std::time::Duration;
use tokio::timer::{Error, Interval};

pub trait Gossiper {
    type Future: Future<Item = (), Error = ()>;

    fn gossip(&mut self) -> Self::Future;
}

// TODO: take a service here as the gossiper
pub struct Gossip<G> {
    interval: Interval,
    gossiper: G,
}

impl<G> Gossip<G> {
    pub fn new(interval: Duration, gossiper: G) -> Gossip<G> {
        let fut = Interval::new_interval(interval);

        Gossip {
            interval: fut,
            gossiper,
        }
    }
}

impl<G: Gossiper> Future for Gossip<G> {
    type Item = ();
    type Error = GossipError;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        match try_ready!(self.interval.poll()) {
            Some(_) => self.gossiper.gossip().poll().map_err(GossipError::from),
            None => Ok(Async::NotReady),
        }
    }
}

#[derive(Debug)]
pub enum GossipError {
    BrokenPipe(Error),
    Inner(()),
}

impl From<Error> for GossipError {
    fn from(e: Error) -> GossipError {
        GossipError::BrokenPipe(e)
    }
}

impl From<()> for GossipError {
    fn from(_: ()) -> GossipError {
        GossipError::Inner(())
    }
}
