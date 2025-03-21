use iced::{
    Rectangle,
    futures::{
        StreamExt,
        channel::mpsc::{UnboundedReceiver, unbounded},
        stream,
    },
};
use iced_futures::Subscription;
use std::{collections::HashMap, fmt::Debug, hash::Hash};

use super::RectangleTracker;

#[cold]
pub fn rectangle_tracker_subscription<
    I: 'static + Hash + Copy + Send + Sync + Debug,
    R: 'static + Hash + Copy + Send + Sync + Debug + Eq,
>(
    id: I,
) -> Subscription<(I, RectangleUpdate<R>)> {
    Subscription::run_with_id(
        id,
        stream::unfold(State::Ready, move |state| start_listening(id, state)),
    )
}

pub enum State<I> {
    Ready,
    Waiting(UnboundedReceiver<(I, Rectangle)>, HashMap<I, Rectangle>),
    Finished,
}

async fn start_listening<I: Copy, R: 'static + Hash + Copy + Send + Sync + Debug + Eq>(
    id: I,
    mut state: State<R>,
) -> Option<((I, RectangleUpdate<R>), State<R>)> {
    loop {
        let (update, new_state) = match state {
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
                        if (prev.width - new.width).abs() > 0.1
                            || (prev.height - new.height).abs() > 0.1
                            || (prev.x - new.x).abs() > 0.1
                            || (prev.y - new.y).abs() > 0.1
                        {
                            map.insert(u.0, new);
                            (
                                Some((id, RectangleUpdate::Rectangle(u))),
                                State::Waiting(rx, map),
                            )
                        } else {
                            (None, State::Waiting(rx, map))
                        }
                    } else {
                        map.insert(u.0, u.1);
                        (
                            Some((id, RectangleUpdate::Rectangle(u))),
                            State::Waiting(rx, map),
                        )
                    }
                }
                None => (None, State::Finished),
            },
            State::Finished => return None,
        };
        state = new_state;
        if let Some(u) = update {
            return Some((u, state));
        }
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
