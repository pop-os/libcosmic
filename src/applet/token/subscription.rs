use crate::iced;
use crate::iced_futures::futures;
use cctk::sctk::reexports::calloop;
use futures::{
    SinkExt, StreamExt,
    channel::mpsc::{UnboundedReceiver, unbounded},
};
use iced::Subscription;
use iced_futures::stream;
use std::{fmt::Debug, hash::Hash, thread::JoinHandle};

use super::wayland_handler::wayland_handler;

pub fn activation_token_subscription<I: 'static + Hash + Copy + Send + Sync + Debug>(
    id: I,
) -> iced::Subscription<TokenUpdate> {
    Subscription::run_with_id(
        id,
        stream::channel(50, move |mut output| async move {
            let mut state = State::Ready;

            loop {
                state = start_listening(state, &mut output).await;
            }
        }),
    )
}

pub enum State {
    Ready,
    Waiting(
        UnboundedReceiver<TokenUpdate>,
        calloop::channel::Sender<TokenRequest>,
        JoinHandle<()>,
    ),
    Finished,
}

async fn start_listening(
    state: State,
    output: &mut futures::channel::mpsc::Sender<TokenUpdate>,
) -> State {
    match state {
        State::Ready => {
            let (calloop_tx, calloop_rx) = calloop::channel::channel();
            let (toplevel_tx, toplevel_rx) = unbounded();
            let handle = std::thread::spawn(move || {
                wayland_handler(toplevel_tx, calloop_rx);
            });
            let tx = calloop_tx.clone();
            _ = output.send(TokenUpdate::Init(tx)).await;
            State::Waiting(toplevel_rx, calloop_tx, handle)
        }
        State::Waiting(mut rx, tx, handle) => {
            if handle.is_finished() {
                _ = output.send(TokenUpdate::Finished).await;
                return State::Finished;
            }
            if let Some(u) = rx.next().await {
                _ = output.send(u).await;
                State::Waiting(rx, tx, handle)
            } else {
                _ = output.send(TokenUpdate::Finished).await;
                State::Finished
            }
        }
        State::Finished => iced::futures::future::pending().await,
    }
}

#[derive(Clone, Debug)]
pub enum TokenUpdate {
    Init(calloop::channel::Sender<TokenRequest>),
    Finished,
    ActivationToken { token: Option<String>, exec: String },
}

#[derive(Clone, Debug)]
pub struct TokenRequest {
    pub app_id: String,
    pub exec: String,
}
