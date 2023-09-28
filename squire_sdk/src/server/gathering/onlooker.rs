use std::{
    pin::Pin,
    task::{Context, Poll},
};

use axum::{
    extract::ws::{Message, WebSocket},
    Error as AxumError,
};
use futures::{
    stream::{FusedStream, SplitSink, SplitStream},
    Sink, SinkExt, Stream, StreamExt,
};

use crate::{api::AuthUser, server::session::SessionWatcher, sync::ClientBoundMessage};

/// This structure captures messages being sent to a person that is in some way participating in
/// the tournament. This person could be a spectator, player, judge, or admin. Messages they pass
/// in are often operations to the tournament that are processed and then forwarded to other.
#[derive(Debug)]
pub struct Crier {
    stream: SplitStream<WebSocket>,
    user: AuthUser,
    session: SessionWatcher,
    is_done: bool,
}

impl Crier {
    pub fn new(stream: SplitStream<WebSocket>, user: AuthUser, session: SessionWatcher) -> Self {
        Self {
            stream,
            session,
            user,
            is_done: false,
        }
    }
}

/// This structure captures messages being sent to a person that is in some way participating in
/// the tournament. This person could be a spectator, player, judge, or admin. Messages passed to
/// them are usually from other users that are submitting operations to the tournament.
#[derive(Debug)]
pub struct Onlooker(SplitSink<WebSocket, Message>);

impl Onlooker {
    pub fn new(sink: SplitSink<WebSocket, Message>) -> Self {
        Self(sink)
    }

    pub async fn send_msg(&mut self, msg: &ClientBoundMessage) -> Result<(), AxumError> {
        let bytes = Message::Binary(postcard::to_allocvec(msg).unwrap());
        self.send(bytes).await
    }
}

#[derive(Debug, Clone)]
pub enum CrierMessage {
    NoAuthMessage(AuthUser, Vec<u8>),
    AuthMessage(AuthUser, Vec<u8>),
    ClosingFrame(AuthUser),
}

/// A `Crier` is a simple wrapper around an account and a websocket connection. We only support
/// binary-encoded messages (using `postcard`). All other messages types are ignored. Moreover,
/// this stream will send exactly one `None` value. This corresponds to the closing frame set by the
/// Websocket when the connection is closed. After that, this stream will return `Poll::Pending`
/// forever.
impl Stream for Crier {
    type Item = CrierMessage;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // If the channel is marked as done, return `Poll::Ready(None)`, i.e. this is a fused
        // stream.
        if self.is_done {
            return Poll::Ready(None);
        }
        match self.stream.poll_next_unpin(cx) {
            Poll::Ready(Some(Err(_))) => Poll::Pending,
            Poll::Ready(None) => {
                self.is_done = true;
                Poll::Ready(Some(CrierMessage::ClosingFrame(self.user.clone())))
            }
            Poll::Ready(Some(Ok(Message::Binary(val)))) => {
                if self.session.is_valid() {
                    Poll::Ready(Some(CrierMessage::AuthMessage(self.user.clone(), val)))
                } else {
                    Poll::Ready(Some(CrierMessage::NoAuthMessage(self.user.clone(), val)))
                }
            }
            _ => Poll::Pending,
        }
    }
}

impl FusedStream for Crier {
    fn is_terminated(&self) -> bool {
        self.is_done
    }
}

impl Sink<Message> for Onlooker {
    type Error = AxumError;

    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Pin::new(&mut self.0).poll_ready(cx)
    }

    fn start_send(mut self: Pin<&mut Self>, item: Message) -> Result<(), Self::Error> {
        Pin::new(&mut self.0).start_send(item)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Pin::new(&mut self.0).poll_flush(cx)
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Pin::new(&mut self.0).poll_close(cx)
    }
}
