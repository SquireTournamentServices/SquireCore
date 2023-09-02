#![allow(dead_code)]
use std::{
    pin::Pin,
    task::{Context, Poll},
};

use futures::{
    stream::{select_all, FuturesUnordered, SelectAll},
    Future, Stream, StreamExt, FutureExt,
};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio_stream::wrappers::UnboundedReceiverStream;

pub trait ActorState: 'static + Send + Sized {
    type Message: Send;

    fn process(&mut self, scheduler: &mut Scheduler<Self>, msg: Self::Message);
}

pub struct ActorBuilder<A: ActorState> {
    send: UnboundedSender<A::Message>,
    recv: Vec<ActorStream<A>>,
    state: A,
}

pub struct ActorClient<A: ActorState> {
    send: UnboundedSender<A::Message>,
}

enum ActorStream<A: ActorState> {
    Main(UnboundedReceiverStream<A::Message>),
    Secondary(Box<dyn 'static + Send + Unpin + Stream<Item = A::Message>>),
}

impl<A: ActorState> Stream for ActorStream<A> {
    type Item = A::Message;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match *self {
            ActorStream::Main(ref mut stream) => Pin::new(stream).poll_next(cx),
            ActorStream::Secondary(ref mut stream) => Pin::new(stream).poll_next(cx),
        }
    }
}

struct ActorRunner<A: ActorState> {
    state: A,
    scheduler: Scheduler<A>,
}

impl<A: ActorState> ActorBuilder<A> {
    pub fn new(state: A) -> Self {
        let (send, recv) = unbounded_channel();
        let recv = vec![recv.into()];
        Self { state, send, recv }
    }

    pub fn client(&self) -> ActorClient<A> {
        ActorClient {
            send: self.send.clone(),
        }
    }

    pub fn add_input<S, I>(&mut self, stream: S)
    where
        S: 'static + Unpin + Send + Stream<Item = I>,
        I: Into<A::Message>,
    {
        self.recv
            .push(ActorStream::Secondary(Box::new(stream.map(|m| m.into()))));
    }

    pub fn launch(self) -> ActorClient<A> {
        let Self { send, recv, state } = self;
        let runner = ActorRunner::new(state, recv);
        runner.launch();
        ActorClient { send }
    }
}

pub struct Scheduler<A: ActorState> {
    recv: SelectAll<ActorStream<A>>,
    queue: FuturesUnordered<Box<dyn 'static + Send + Unpin + Future<Output = A::Message>>>,
}

impl<A: ActorState> ActorRunner<A> {
    fn new(state: A, recvs: impl IntoIterator<Item = ActorStream<A>>) -> Self {
        let scheduler = Scheduler::new(recvs);
        Self { state, scheduler }
    }

    fn launch(self) {
        // Dropping join handle because `run` will never return
        drop(tokio::spawn(self.run()));
    }

    async fn run(mut self) -> ! {
        loop {
            let msg = self.scheduler.next().await.unwrap();
            self.state.process(&mut self.scheduler, msg);
        }
    }
}

impl<A: ActorState> Scheduler<A> {
    fn new(recv: impl IntoIterator<Item = ActorStream<A>>) -> Self {
        let recv = select_all(recv);
        let queue = FuturesUnordered::new();
        Self { recv, queue }
    }

    pub fn schedule<F, I>(&mut self, fut: F)
    where
        F: 'static + Unpin + Send + Future<Output = I>,
        I: 'static + Into<A::Message>,
    {
        self.queue.push(Box::new(fut.map(Into::into)));
    }

    pub fn add_stream<S, I>(&mut self, stream: S)
    where
        S: 'static + Unpin + Send + Stream<Item = I>,
        I: Into<A::Message>,
    {
        self.recv
            .push(ActorStream::Secondary(Box::new(stream.map(|m| m.into()))));
    }
}

impl<A: ActorState> Stream for Scheduler<A> {
    type Item = A::Message;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let digest = self.recv.poll_next_unpin(cx);
        if digest.is_ready() {
            digest
        } else {
            self.queue.poll_next_unpin(cx)
        }
    }
}

impl<A: ActorState> From<UnboundedReceiver<A::Message>> for ActorStream<A> {
    fn from(value: UnboundedReceiver<A::Message>) -> Self {
        Self::Main(UnboundedReceiverStream::new(value))
    }
}
