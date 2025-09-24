// Copyright 2025 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use super::Action;
#[cfg(feature = "winit")]
use crate::Application;

use iced::window;
use std::{any::Any, sync::Arc};

/// Used to produce a destroy popup message from within a widget.
#[cfg(feature = "wayland")]
#[must_use]
pub fn destroy_popup(id: iced_core::window::Id) -> Action {
    Action::DestroyPopup(id)
}

#[cfg(feature = "wayland")]
#[must_use]
pub fn destroy_subsurface(id: iced_core::window::Id) -> Action {
    Action::DestroySubsurface(id)
}

#[cfg(feature = "wayland")]
#[must_use]
pub fn destroy_window(id: iced_core::window::Id) -> Action {
    Action::DestroyWindow(id)
}

#[cfg(all(feature = "wayland", feature = "winit"))]
#[must_use]
pub fn app_window<App: Application>(
    settings: impl Fn(&mut App) -> window::Settings + Send + Sync + 'static,
    view: Option<
        Box<
            dyn for<'a> Fn(&'a App) -> crate::Element<'a, crate::Action<App::Message>>
                + Send
                + Sync
                + 'static,
        >,
    >,
) -> (window::Id, Action) {
    let id = window::Id::unique();

    let boxed: Box<dyn Fn(&mut App) -> window::Settings + Send + Sync + 'static> =
        Box::new(settings);
    let boxed: Box<dyn Any + Send + Sync + 'static> = Box::new(boxed);

    (
        id,
        Action::AppWindow(
            id,
            Arc::new(boxed),
            view.map(|view| {
                let boxed: Box<dyn Any + Send + Sync + 'static> = Box::new(view);
                Arc::new(boxed)
            }),
        ),
    )
}

/// Used to create a window message from within a widget.
#[cfg(all(feature = "wayland", feature = "winit"))]
#[must_use]
pub fn simple_window<Message: 'static>(
    settings: impl Fn() -> window::Settings + Send + Sync + 'static,
    view: Option<
        impl Fn() -> crate::Element<'static, crate::Action<Message>> + Send + Sync + 'static,
    >,
) -> (window::Id, Action) {
    let id = window::Id::unique();

    let boxed: Box<dyn Fn() -> window::Settings + Send + Sync + 'static> = Box::new(settings);
    let boxed: Box<dyn Any + Send + Sync + 'static> = Box::new(boxed);

    (
        id,
        Action::Window(
            id,
            Arc::new(boxed),
            view.map(|view| {
                let boxed: Box<
                    dyn Fn() -> crate::Element<'static, crate::Action<Message>>
                        + Send
                        + Sync
                        + 'static,
                > = Box::new(view);
                let boxed: Box<dyn Any + Send + Sync + 'static> = Box::new(boxed);
                Arc::new(boxed)
            }),
        ),
    )
}

#[cfg(all(feature = "wayland", feature = "winit"))]
#[must_use]
pub fn app_popup<App: Application>(
    settings: impl Fn(&mut App) -> iced_runtime::platform_specific::wayland::popup::SctkPopupSettings
    + Send
    + Sync
    + 'static,
    view: Option<
        Box<
            dyn for<'a> Fn(&'a App) -> crate::Element<'a, crate::Action<App::Message>>
                + Send
                + Sync
                + 'static,
        >,
    >,
) -> Action {
    let boxed: Box<
        dyn Fn(&mut App) -> iced_runtime::platform_specific::wayland::popup::SctkPopupSettings
            + Send
            + Sync
            + 'static,
    > = Box::new(settings);
    let boxed: Box<dyn Any + Send + Sync + 'static> = Box::new(boxed);

    Action::AppPopup(
        Arc::new(boxed),
        view.map(|view| {
            let boxed: Box<dyn Any + Send + Sync + 'static> = Box::new(view);
            Arc::new(boxed)
        }),
    )
}

/// Used to create a subsurface message from within a widget.
#[cfg(all(feature = "wayland", feature = "winit"))]
#[must_use]
pub fn simple_subsurface<Message: 'static, V>(
    settings: impl Fn() -> iced_runtime::platform_specific::wayland::subsurface::SctkSubsurfaceSettings
    + Send
    + Sync
    + 'static,
    view: Option<
        Box<dyn Fn() -> crate::Element<'static, crate::Action<Message>> + Send + Sync + 'static>,
    >,
) -> Action {
    let boxed: Box<
        dyn Fn() -> iced_runtime::platform_specific::wayland::subsurface::SctkSubsurfaceSettings
            + Send
            + Sync
            + 'static,
    > = Box::new(settings);
    let boxed: Box<dyn Any + Send + Sync + 'static> = Box::new(boxed);

    Action::Subsurface(
        Arc::new(boxed),
        view.map(|view| {
            let boxed: Box<dyn Any + Send + Sync + 'static> = Box::new(view);
            Arc::new(boxed)
        }),
    )
}

/// Used to create a popup message from within a widget.
#[cfg(all(feature = "wayland", feature = "winit"))]
#[must_use]
pub fn simple_popup<Message: 'static>(
    settings: impl Fn() -> iced_runtime::platform_specific::wayland::popup::SctkPopupSettings
    + Send
    + Sync
    + 'static,
    view: Option<
        impl Fn() -> crate::Element<'static, crate::Action<Message>> + Send + Sync + 'static,
    >,
) -> Action {
    let boxed: Box<
        dyn Fn() -> iced_runtime::platform_specific::wayland::popup::SctkPopupSettings
            + Send
            + Sync
            + 'static,
    > = Box::new(settings);
    let boxed: Box<dyn Any + Send + Sync + 'static> = Box::new(boxed);

    Action::Popup(
        Arc::new(boxed),
        view.map(|view| {
            let boxed: Box<
                dyn Fn() -> crate::Element<'static, crate::Action<Message>> + Send + Sync + 'static,
            > = Box::new(view);
            let boxed: Box<dyn Any + Send + Sync + 'static> = Box::new(boxed);
            Arc::new(boxed)
        }),
    )
}

#[cfg(all(feature = "wayland", feature = "winit"))]
#[must_use]
pub fn subsurface<App: Application>(
    settings: impl Fn(
        &mut App,
    )
        -> iced_runtime::platform_specific::wayland::subsurface::SctkSubsurfaceSettings
    + Send
    + Sync
    + 'static,
    // XXX Boxed trait object is required for less cumbersome type inference, but we box it anyways.
    view: Option<
        Box<
            dyn for<'a> Fn(&'a App) -> crate::Element<'a, crate::Action<App::Message>>
                + Send
                + Sync
                + 'static,
        >,
    >,
) -> Action {
    let boxed: Box<
        dyn Fn(
                &mut App,
            )
                -> iced_runtime::platform_specific::wayland::subsurface::SctkSubsurfaceSettings
            + Send
            + Sync
            + 'static,
    > = Box::new(settings);
    let boxed: Box<dyn Any + Send + Sync + 'static> = Box::new(boxed);

    Action::AppSubsurface(
        Arc::new(boxed),
        view.map(|view| {
            let boxed: Box<dyn Any + Send + Sync + 'static> = Box::new(view);
            Arc::new(boxed)
        }),
    )
}
