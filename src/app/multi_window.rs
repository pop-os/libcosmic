// Copyright 2024 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! Create and run daemons that run in the background.
//! Copied from iced 0.13, but adds optional initial window

use iced::application;
use iced::window;
use iced::{
    self, Program,
    program::{self, with_style, with_subscription, with_theme, with_title},
    runtime::{Appearance, DefaultStyle},
};
use iced::{Element, Result, Settings, Subscription, Task};

use std::marker::PhantomData;

pub(crate) struct Instance<State, Message, Theme, Renderer, Update, View, Executor> {
    update: Update,
    view: View,
    _state: PhantomData<State>,
    _message: PhantomData<Message>,
    _theme: PhantomData<Theme>,
    _renderer: PhantomData<Renderer>,
    _executor: PhantomData<Executor>,
}

/// Creates an iced [`MultiWindow`] given its title, update, and view logic.
pub fn multi_window<State, Message, Theme, Renderer, Executor>(
    title: impl Title<State>,
    update: impl application::Update<State, Message>,
    view: impl for<'a> self::View<'a, State, Message, Theme, Renderer>,
) -> MultiWindow<impl Program<State = State, Message = Message, Theme = Theme>>
where
    State: 'static,
    Message: Send + std::fmt::Debug + 'static,
    Theme: Default + DefaultStyle,
    Renderer: program::Renderer,
    Executor: iced::Executor,
{
    use std::marker::PhantomData;

    impl<State, Message, Theme, Renderer, Update, View, Executor> Program
        for Instance<State, Message, Theme, Renderer, Update, View, Executor>
    where
        Message: Send + std::fmt::Debug + 'static,
        Theme: Default + DefaultStyle,
        Renderer: program::Renderer,
        Update: application::Update<State, Message>,
        View: for<'a> self::View<'a, State, Message, Theme, Renderer>,
        Executor: iced::Executor,
    {
        type State = State;
        type Message = Message;
        type Theme = Theme;
        type Renderer = Renderer;
        type Executor = Executor;

        fn update(&self, state: &mut Self::State, message: Self::Message) -> Task<Self::Message> {
            self.update.update(state, message).into()
        }

        fn view<'a>(
            &self,
            state: &'a Self::State,
            window: window::Id,
        ) -> Element<'a, Self::Message, Self::Theme, Self::Renderer> {
            self.view.view(state, window).into()
        }
    }

    MultiWindow {
        raw: Instance {
            update,
            view,
            _state: PhantomData,
            _message: PhantomData,
            _theme: PhantomData,
            _renderer: PhantomData,
            _executor: PhantomData::<Executor>,
        },
        settings: Settings::default(),
        window: None,
    }
    .title(title)
}

/// The underlying definition and configuration of an iced daemon.
///
/// You can use this API to create and run iced applications
/// step by step—without coupling your logic to a trait
/// or a specific type.
///
/// You can create a [`MultiWindow`] with the [`daemon`] helper.
#[derive(Debug)]
pub struct MultiWindow<P: Program> {
    raw: P,
    settings: Settings,
    window: Option<window::Settings>,
}

impl<P: Program> MultiWindow<P> {
    #[cfg(any(feature = "winit", feature = "wayland"))]
    /// Runs the [`MultiWindow`].
    ///
    /// The state of the [`MultiWindow`] must implement [`Default`].
    /// If your state does not implement [`Default`], use [`run_with`]
    /// instead.
    ///
    /// [`run_with`]: Self::run_with
    pub fn run(self) -> Result
    where
        Self: 'static,
        P::State: Default,
    {
        self.raw.run(self.settings, self.window)
    }

    #[cfg(any(feature = "winit", feature = "wayland"))]
    /// Runs the [`MultiWindow`] with a closure that creates the initial state.
    pub fn run_with<I>(self, initialize: I) -> Result
    where
        Self: 'static,
        I: FnOnce() -> (P::State, Task<P::Message>) + 'static,
    {
        self.raw.run_with(self.settings, self.window, initialize)
    }

    /// Sets the [`Settings`] that will be used to run the [`MultiWindow`].
    pub fn settings(self, settings: Settings) -> Self {
        Self { settings, ..self }
    }

    /// Sets the [`Title`] of the [`MultiWindow`].
    pub(crate) fn title(
        self,
        title: impl Title<P::State>,
    ) -> MultiWindow<impl Program<State = P::State, Message = P::Message, Theme = P::Theme>> {
        MultiWindow {
            raw: with_title(self.raw, move |state, window| title.title(state, window)),
            settings: self.settings,
            window: self.window,
        }
    }

    /// Sets the subscription logic of the [`MultiWindow`].
    pub fn subscription(
        self,
        f: impl Fn(&P::State) -> Subscription<P::Message>,
    ) -> MultiWindow<impl Program<State = P::State, Message = P::Message, Theme = P::Theme>> {
        MultiWindow {
            raw: with_subscription(self.raw, f),
            settings: self.settings,
            window: self.window,
        }
    }

    /// Sets the theme logic of the [`MultiWindow`].
    pub fn theme(
        self,
        f: impl Fn(&P::State, window::Id) -> P::Theme,
    ) -> MultiWindow<impl Program<State = P::State, Message = P::Message, Theme = P::Theme>> {
        MultiWindow {
            raw: with_theme(self.raw, f),
            settings: self.settings,
            window: self.window,
        }
    }

    /// Sets the style logic of the [`MultiWindow`].
    pub fn style(
        self,
        f: impl Fn(&P::State, &P::Theme) -> Appearance,
    ) -> MultiWindow<impl Program<State = P::State, Message = P::Message, Theme = P::Theme>> {
        MultiWindow {
            raw: with_style(self.raw, f),
            settings: self.settings,
            window: self.window,
        }
    }

    /// Sets the window settings of the [`MultiWindow`].
    pub fn window(self, window: window::Settings) -> Self {
        Self {
            raw: self.raw,
            settings: self.settings,
            window: Some(window),
        }
    }
}

/// The title logic of some [`MultiWindow`].
///
/// This trait is implemented both for `&static str` and
/// any closure `Fn(&State, window::Id) -> String`.
///
/// This trait allows the [`daemon`] builder to take any of them.
pub trait Title<State> {
    /// Produces the title of the [`MultiWindow`].
    fn title(&self, state: &State, window: window::Id) -> String;
}

impl<State> Title<State> for &'static str {
    fn title(&self, _state: &State, _window: window::Id) -> String {
        (*self).to_string()
    }
}

impl<T, State> Title<State> for T
where
    T: Fn(&State, window::Id) -> String,
{
    fn title(&self, state: &State, window: window::Id) -> String {
        self(state, window)
    }
}

/// The view logic of some [`MultiWindow`].
///
/// This trait allows the [`daemon`] builder to take any closure that
/// returns any `Into<Element<'_, Message>>`.
pub trait View<'a, State, Message, Theme, Renderer> {
    /// Produces the widget of the [`MultiWindow`].
    fn view(
        &self,
        state: &'a State,
        window: window::Id,
    ) -> impl Into<Element<'a, Message, Theme, Renderer>>;
}

impl<'a, T, State, Message, Theme, Renderer, Widget> View<'a, State, Message, Theme, Renderer> for T
where
    T: Fn(&'a State, window::Id) -> Widget,
    State: 'static,
    Widget: Into<Element<'a, Message, Theme, Renderer>>,
{
    fn view(
        &self,
        state: &'a State,
        window: window::Id,
    ) -> impl Into<Element<'a, Message, Theme, Renderer>> {
        self(state, window)
    }
}
