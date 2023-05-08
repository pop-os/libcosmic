use iced::{
    futures::{
        channel::mpsc::{unbounded, UnboundedReceiver},
        StreamExt,
    },
    subscription, Rectangle,
};
use std::{collections::HashMap, fmt::Debug, hash::Hash};

use super::RectangleTracker;

pub fn rectangle_tracker_subscription<
    I: 'static + Hash + Copy + Send + Sync + Debug,
    R: 'static + Hash + Copy + Send + Sync + Debug + Eq,
>(
    id: I,
) -> iced::Subscription<Option<(I, RectangleUpdate<R>)>> {
    subscription::unfold(id, State::Ready, move |state| start_listening(id, state))
}

pub enum State<I> {
    Ready,
    Waiting(UnboundedReceiver<(I, Rectangle)>, HashMap<I, Rectangle>),
    Finished,
}

async fn start_listening<I: Copy, R: 'static + Hash + Copy + Send + Sync + Debug + Eq>(
    id: I,
    state: State<R>,
) -> (Option<(I, RectangleUpdate<R>)>, State<R>) {
    match state {
        State::Ready => {
            let (tx, rx) = unbounded();

            (
                Some((id, RectangleUpdate::Init(RectangleTracker { tx }))),
                State::Waiting(rx, HashMap::new()),
            )
        }
        State::Waiting(mut rx, mut map) => match rx.next().await {
            Some(u) => {
                if let Some(prev) = map.get(&u.0) {
                    let new = u.1;
                    if prev.width != new.width
                        || prev.height != new.height
                        || prev.x != new.x
                        || prev.y != new.y
                    {
                        map.insert(u.0, new);
                        return (
                            Some((id, RectangleUpdate::Rectangle(u))),
                            State::Waiting(rx, map),
                        );
                    }
                } else {
                    map.insert(u.0, u.1);
                    return (
                        Some((id, RectangleUpdate::Rectangle(u))),
                        State::Waiting(rx, map),
                    );
                }
                (None, State::Waiting(rx, map))
            }
            None => (None, State::Finished),
        },
        State::Finished => iced::futures::future::pending().await,
    }
}

#[derive(Clone, Debug)]
pub enum RectangleUpdate<I>
where
    I: 'static + Hash + Copy + Send + Sync + Debug,
{
    Rectangle((I, Rectangle)),
    Init(RectangleTracker<I>),
}
