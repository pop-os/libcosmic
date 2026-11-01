use crate::{Config, ConfigGet, ConfigSet, Error};

pub trait App {
    const ID: &'static str;
    // XXX how to handle versioning?
    const VERSION: u64;
}

pub trait Setting<A: App> {
    const NAME: &'static str;
    // TODO can't use &str to set? Need to serialize owned value.
    type Type: serde::Serialize + serde::de::DeserializeOwned;
}

pub struct AppConfig<A: App> {
    config: Config,
    _app: std::marker::PhantomData<A>,
}

impl<A: App> AppConfig<A> {
    pub fn new() -> Result<Self, Error> {
        Ok(Self {
            config: Config::new(A::ID, A::VERSION)?,
            _app: std::marker::PhantomData,
        })
    }

    // XXX default value, if none set?
    pub fn get<S: Setting<A>>(&self) -> Result<S::Type, Error> {
        self.config.get(S::NAME)
    }

    pub fn set<S: Setting<A>>(&self, value: S::Type) -> Result<(), Error> {
        self.config.set(S::NAME, value)
    }
}
