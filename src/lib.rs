#[macro_use]
extern crate futures;

mod gossip;

use crate::gossip::{Gossip, Gossiper};
use futures::{
    sync::mpsc,
    {
        future::{self, FutureResult},
        Async, Future, Poll, Sink,
    },
};
use std::time::Duration;
use tower_direct_service::DirectService;

pub struct PingGossiper;
impl Gossiper for PingGossiper {
    type Future = FutureResult<(), ()>;

    fn gossip(&mut self) -> Self::Future {
        println!("Ping!");
        future::ok(())
    }
}

pub struct Node {
    sender: mpsc::Sender<Packet>,
    gossip: Gossip<PingGossiper>,
}

impl Node {
    pub fn new() -> Self {
        let (tx, _rx) = mpsc::channel(1000);

        Node {
            sender: tx,
            gossip: Gossip::new(Duration::from_secs(1), PingGossiper),
        }
    }
}

impl DirectService<Packet> for Node {
    type Response = mpsc::Sender<Packet>;
    type Error = mpsc::SendError<Packet>;
    type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;

    fn poll_ready(&mut self) -> Poll<(), Self::Error> {
        Ok(Async::Ready(()))
    }

    fn poll_service(&mut self) -> Poll<(), Self::Error> {
        match self.gossip.poll() {
            Ok(i) => Ok(i),
            Err(e) => panic!("Error occured gossiping: {:?}", e),
        }
    }

    fn poll_close(&mut self) -> Poll<(), Self::Error> {
        self.sender.poll_complete()
    }

    fn call(&mut self, req: Packet) -> Self::Future {
        let sender = self.sender.clone();
        Box::new(sender.send(req))
    }
}

impl Future for Node {
    type Item = ();
    type Error = ();

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        loop {
            if let Ok(Async::NotReady) = self.gossip.poll() {
                return Ok(Async::NotReady);
            }
        }
    }
}

pub enum Packet {
    Ping(u32),
    Ack(u32),
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn basic() {
        let mut node = Node::new();

        node.call(Packet::Ping(0));

        tokio::run(node);
    }
}
