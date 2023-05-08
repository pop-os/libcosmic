// TODO If possible, calloop could poll inotify/kqueue without a thread

use calloop::channel;

use crate::{Config, Error};

pub struct ConfigWatchSource {
    channel: channel::Channel<(Config, Vec<String>)>,
    _watcher: notify::RecommendedWatcher,
}

impl ConfigWatchSource {
    pub fn new(config: &Config) -> Result<Self, Error> {
        let (sender, channel) = channel::sync_channel(32);
        let _watcher = config.watch(move |config, keys| {
            let _ = sender.send((config.clone(), keys.to_owned()));
        })?;
        Ok(Self { channel, _watcher })
    }
}

impl calloop::EventSource for ConfigWatchSource {
    type Event = (Config, Vec<String>);
    type Metadata = ();
    type Ret = ();
    type Error = calloop::channel::ChannelError;

    fn process_events<F>(
        &mut self,
        readiness: calloop::Readiness,
        token: calloop::Token,
        mut cb: F,
    ) -> Result<calloop::PostAction, Self::Error>
    where
        F: FnMut(Self::Event, &mut Self::Metadata) -> Self::Ret,
    {
        self.channel
            .process_events(readiness, token, |event, ()| match event {
                calloop::channel::Event::Msg(msg) => cb(msg, &mut ()),
                calloop::channel::Event::Closed => {}
            })
    }

    fn register(
        &mut self,
        poll: &mut calloop::Poll,
        token_factory: &mut calloop::TokenFactory,
    ) -> Result<(), calloop::Error> {
        self.channel.register(poll, token_factory)
    }

    fn reregister(
        &mut self,
        poll: &mut calloop::Poll,
        token_factory: &mut calloop::TokenFactory,
    ) -> Result<(), calloop::Error> {
        self.channel.reregister(poll, token_factory)
    }

    fn unregister(&mut self, poll: &mut calloop::Poll) -> Result<(), calloop::Error> {
        self.channel.unregister(poll)
    }
}
