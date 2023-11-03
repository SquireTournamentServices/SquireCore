use std::{
    fmt::Debug,
    pin::Pin,
    task::{Context, Poll},
};

pub use async_trait::async_trait;
use futures::{
    stream::{select_all, FuturesUnordered, SelectAll},
    Future, FutureExt, Stream, StreamExt,
};
use instant::Instant;
use pin_project::pin_project;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
pub use tokio::sync::oneshot::{
    channel as oneshot_channel, Receiver as OneshotReceiver, Sender as OneshotSender,
};
use tokio_stream::wrappers::UnboundedReceiverStream;

use crate::compat::{
    sleep_until, spawn_task, Sendable, SendableFuture, SendableStream, SendableWrapper, Sleep,
};

// This state needs to be send because of constraints of `async_trait`. Ideally, it would be
// `Sendable`.
#[async_trait]
pub trait ActorState: 'static + Send + Sized {
    type Message: Sendable;

    #[allow(unused_variables)]
    async fn start_up(&mut self, scheduler: &mut Scheduler<Self>) {}

    async fn process(&mut self, scheduler: &mut Scheduler<Self>, msg: Self::Message);
}

pub struct ActorBuilder<A: ActorState> {
    send: UnboundedSender<A::Message>,
    recv: Vec<ActorStream<A>>,
    state: A,
}

pub struct ActorClient<A: ActorState> {
    send: SendableWrapper<UnboundedSender<A::Message>>,
}

impl<A: ActorState> Clone for ActorClient<A> {
    fn clone(&self) -> Self {
        Self::new(self.send.clone().take())
    }
}

enum ActorStream<A: ActorState> {
    Main(UnboundedReceiverStream<A::Message>),
    Secondary(Box<dyn SendableStream<Item = A::Message>>),
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
        ActorClient::new(self.send.clone())
    }

    pub fn add_input<S, I>(&mut self, stream: S)
    where
        S: SendableStream<Item = I>,
        I: Into<A::Message>,
    {
        self.recv
            .push(ActorStream::Secondary(Box::new(stream.map(|m| m.into()))));
    }

    pub fn launch(self) -> ActorClient<A> {
        let Self { send, recv, state } = self;
        let runner = ActorRunner::new(state, recv);
        runner.launch();
        ActorClient::new(send)
    }
}

pub struct Scheduler<A: ActorState> {
    recv: SendableWrapper<SelectAll<ActorStream<A>>>,
    #[allow(clippy::type_complexity)]
    queue: SendableWrapper<FuturesUnordered<Pin<Box<dyn SendableFuture<Output = A::Message>>>>>,
    tasks: SendableWrapper<FuturesUnordered<Pin<Box<dyn SendableFuture<Output = ()>>>>>,
    // TODO:
    //  - Add a `FuturesUnordered` for futures that are 'static  + Send and yield nothing
    //  - Add a queue for timers so that they are not lumped in the `queue`.
    //    - Make those timers cancelable by assoicating each on with an id (usize)
}

#[pin_project]
pub struct Timer<T> {
    #[pin]
    deadline: Sleep,
    msg: Option<T>,
}

impl<T> Timer<T> {
    pub fn new(deadline: Instant, msg: T) -> Self {
        Self {
            deadline: sleep_until(deadline),
            msg: Some(msg),
        }
    }
}

impl<T> Future for Timer<T> {
    type Output = T;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.deadline
            .poll_unpin(cx)
            .map(|_| self.msg.take().unwrap())
    }
}

impl<A: ActorState> ActorRunner<A> {
    fn new(state: A, recvs: impl IntoIterator<Item = ActorStream<A>>) -> Self {
        let scheduler = Scheduler::new(recvs);
        Self { state, scheduler }
    }

    fn launch(self) {
        spawn_task(self.run())
    }

    async fn run(mut self) -> ! {
        self.state.start_up(&mut self.scheduler).await;
        loop {
            tokio::select! {
                msg = self.scheduler.recv.next() => {
                    self.state.process(&mut self.scheduler, msg.unwrap()).await;
                },
                msg = self.scheduler.queue.next(), if !self.scheduler.queue.is_empty() => {
                    self.state.process(&mut self.scheduler, msg.unwrap()).await;
                },
                _ = self.scheduler.tasks.next(), if !self.scheduler.tasks.is_empty() => {},
            }
        }
    }
}

impl<A: ActorState> Scheduler<A> {
    fn new(recv: impl IntoIterator<Item = ActorStream<A>>) -> Self {
        let recv = SendableWrapper::new(select_all(recv));
        let queue = SendableWrapper::new(FuturesUnordered::new());
        let tasks = SendableWrapper::new(FuturesUnordered::new());
        Self { recv, queue, tasks }
    }

    pub fn add_task<F, I>(&mut self, fut: F)
    where
        F: SendableFuture<Output = I>,
        I: 'static + Into<A::Message>,
    {
        self.queue.push(Box::pin(fut.map(Into::into)));
    }

    pub fn process<F>(&mut self, fut: F)
    where
        F: SendableFuture<Output = ()>,
    {
        self.tasks.push(Box::pin(fut));
    }

    pub fn add_stream<S, I>(&mut self, stream: S)
    where
        S: SendableStream<Item = I>,
        I: Into<A::Message>,
    {
        self.recv
            .push(ActorStream::Secondary(Box::new(stream.map(|m| m.into()))));
    }

    pub fn schedule<M>(&mut self, deadline: Instant, msg: M)
    where
        M: 'static + Into<A::Message>,
    {
        self.queue.push(Box::pin(Timer::new(deadline, msg.into())));
    }
}

impl<A: ActorState> Stream for Scheduler<A> {
    type Item = A::Message;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let digest = self.recv.poll_next_unpin(cx);
        if digest.is_ready() {
            return digest;
        }
        let digest = self.queue.poll_next_unpin(cx);
        match &digest {
            Poll::Pending | Poll::Ready(None) => Poll::Pending,
            Poll::Ready(_) => digest,
        }
    }
}

impl<A: ActorState> From<UnboundedReceiver<A::Message>> for ActorStream<A> {
    fn from(value: UnboundedReceiver<A::Message>) -> Self {
        Self::Main(UnboundedReceiverStream::new(value))
    }
}

impl<A: ActorState> ActorClient<A> {
    fn new(send: UnboundedSender<A::Message>) -> Self {
        let send = SendableWrapper::new(send);
        Self { send }
    }

    pub fn builder(state: A) -> ActorBuilder<A> {
        ActorBuilder::new(state)
    }

    pub fn send(&self, msg: impl Into<A::Message>) {
        // This returns a result. It only errors when the connected actor panics. Should we "bubble
        // up" that panic?
        let _ = self.send.send(msg.into());
    }

    pub fn track<M, T>(&self, msg: M) -> Tracker<T>
    where
        A::Message: From<(M, OneshotSender<T>)>,
    {
        let (send, recv) = oneshot_channel();
        let msg = A::Message::from((msg, send));
        self.send(msg);
        Tracker::new(recv)
    }
}

pub struct Tracker<T> {
    recv: OneshotReceiver<T>,
}

impl<T> Tracker<T> {
    pub fn new(recv: OneshotReceiver<T>) -> Self {
        Self { recv }
    }
}

impl<T> Future for Tracker<T> {
    type Output = T;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.recv).poll(cx).map(Result::unwrap)
    }
}

impl<A: ActorState> Debug for ActorClient<A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, r#"ActorClient {{ "send": {:?} }}"#, &*self.send)
    }
}

impl<A> Default for ActorClient<A>
where
    A: ActorState + Default,
{
    fn default() -> Self {
        Self::builder(A::default()).launch()
    }
}
