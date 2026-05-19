// Copyright 2025 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use super::Action;
#[cfg(feature = "winit")]
use crate::Application;

use iced::{Rectangle, window};
#[cfg(all(feature = "wayland", target_os = "linux", feature = "winit"))]
use iced_runtime::platform_specific::wayland::CornerRadius;
#[cfg(all(feature = "wayland", target_os = "linux", feature = "winit"))]
use iced_runtime::platform_specific::wayland::layer_surface::IcedMargin;
use std::any::Any;
use std::sync::Arc;

/// Used to produce a destroy popup message from within a widget.
#[cfg(all(feature = "wayland", target_os = "linux"))]
#[must_use]
pub fn destroy_popup(id: iced_core::window::Id) -> Action {
    Action::DestroyPopup(id)
}

#[cfg(all(feature = "wayland", target_os = "linux"))]
#[must_use]
pub fn destroy_subsurface(id: iced_core::window::Id) -> Action {
    Action::DestroySubsurface(id)
}

#[cfg(all(feature = "wayland", target_os = "linux"))]
#[must_use]
pub fn destroy_window(id: iced_core::window::Id) -> Action {
    Action::DestroyWindow(id)
}

#[cfg(all(feature = "wayland", target_os = "linux"))]
#[must_use]
pub fn destroy_layer_shell(id: iced_core::window::Id) -> Action {
    Action::DestroyLayerShell(id)
}

#[derive(Debug, Default, Copy, Clone)]
pub struct LiveSettings {
    #[cfg(all(feature = "wayland", target_os = "linux", feature = "winit"))]
    /// Override the surface padding value for the surface type.
    pub padding: Option<IcedMargin>,
    #[cfg(all(feature = "wayland", target_os = "linux", feature = "winit"))]
    /// Override the default corner radius value for the surface type.
    pub corners: Option<CornerRadius>,
    /// Override the default blur setting for the surface type.
    pub blur: Option<bool>,
}

#[cfg(all(feature = "wayland", target_os = "linux", feature = "winit"))]
type BoxedView<App> = Option<
    Box<
        dyn Fn(&App) -> crate::Element<'_, crate::Action<<App as Application>::Message>>
            + Send
            + Sync
            + 'static,
    >,
>;

#[cfg(all(feature = "wayland", target_os = "linux", feature = "winit"))]
#[must_use]
pub fn app_window<App: Application>(
    live_settings: impl Fn(&App) -> LiveSettings + Send + Sync + 'static,
    settings: impl Fn(&mut App) -> window::Settings + Send + Sync + 'static,
    view: BoxedView<App>,
) -> (window::Id, Action) {
    let id = window::Id::unique();

    let boxed: Box<dyn Fn(&mut App) -> window::Settings + Send + Sync + 'static> =
        Box::new(settings);
    let boxed: Box<dyn Any + Send + Sync + 'static> = Box::new(boxed);

    let boxed_live: Box<dyn Fn(&App) -> LiveSettings + Send + Sync + 'static> =
        Box::new(live_settings);
    let boxed_live: Box<dyn Any + Send + Sync + 'static> = Box::new(boxed_live);

    (
        id,
        Action::AppWindow(
            id,
            Arc::new(boxed),
            Arc::new(boxed_live),
            view.map(|view| {
                let boxed: Box<dyn Any + Send + Sync + 'static> = Box::new(view);
                Arc::new(boxed)
            }),
        ),
    )
}

/// Used to create a window message from within a widget.
#[cfg(all(feature = "wayland", target_os = "linux", feature = "winit"))]
#[must_use]
pub fn simple_window<Message: 'static>(
    live_settings: impl Fn() -> LiveSettings + Send + Sync + 'static,
    settings: impl Fn() -> window::Settings + Send + Sync + 'static,
    view: Option<
        impl Fn() -> crate::Element<'static, crate::Action<Message>> + Send + Sync + 'static,
    >,
) -> (window::Id, Action) {
    let id = window::Id::unique();

    let boxed: Box<dyn Fn() -> window::Settings + Send + Sync + 'static> = Box::new(settings);
    let boxed: Box<dyn Any + Send + Sync + 'static> = Box::new(boxed);

    let boxed_live: Box<dyn Fn() -> LiveSettings + Send + Sync + 'static> = Box::new(live_settings);
    let boxed_live: Box<dyn Any + Send + Sync + 'static> = Box::new(boxed_live);

    (
        id,
        Action::Window(
            id,
            Arc::new(boxed),
            Arc::new(boxed_live),
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

#[cfg(all(feature = "wayland", target_os = "linux", feature = "winit"))]
#[must_use]
pub fn app_popup<App: Application>(
    live_settings: impl Fn(&App) -> LiveSettings + Send + Sync + 'static,
    settings: impl Fn(&mut App) -> iced_runtime::platform_specific::wayland::popup::SctkPopupSettings
    + Send
    + Sync
    + 'static,
    view: BoxedView<App>,
) -> Action {
    let boxed: Box<
        dyn Fn(&mut App) -> iced_runtime::platform_specific::wayland::popup::SctkPopupSettings
            + Send
            + Sync
            + 'static,
    > = Box::new(settings);
    let boxed: Box<dyn Any + Send + Sync + 'static> = Box::new(boxed);

    let boxed_live: Box<dyn Fn(&App) -> LiveSettings + Send + Sync + 'static> =
        Box::new(live_settings);
    let boxed_live: Box<dyn Any + Send + Sync + 'static> = Box::new(boxed_live);

    Action::AppPopup(
        Arc::new(boxed),
        Arc::new(boxed_live),
        view.map(|view| {
            let boxed: Box<dyn Any + Send + Sync + 'static> = Box::new(view);
            Arc::new(boxed)
        }),
    )
}

/// Used to create a subsurface message from within a widget.
#[cfg(all(feature = "wayland", target_os = "linux", feature = "winit"))]
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
        Arc::new(Box::new(LiveSettings::default)),
        view.map(|view| {
            let boxed: Box<dyn Any + Send + Sync + 'static> = Box::new(view);
            Arc::new(boxed)
        }),
    )
}

/// Used to create a popup message from within a widget.
#[cfg(all(feature = "wayland", target_os = "linux", feature = "winit"))]
#[must_use]
pub fn simple_popup<Message: 'static>(
    live_settings: impl Fn() -> LiveSettings + Send + Sync + 'static,
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

    let boxed_live: Box<dyn Fn() -> LiveSettings + Send + Sync + 'static> = Box::new(live_settings);
    let boxed_live: Box<dyn Any + Send + Sync + 'static> = Box::new(boxed_live);

    Action::Popup(
        Arc::new(boxed),
        Arc::new(boxed_live),
        view.map(|view| {
            let boxed: Box<
                dyn Fn() -> crate::Element<'static, crate::Action<Message>> + Send + Sync + 'static,
            > = Box::new(view);
            let boxed: Box<dyn Any + Send + Sync + 'static> = Box::new(boxed);
            Arc::new(boxed)
        }),
    )
}

#[cfg(all(feature = "wayland", target_os = "linux", feature = "winit"))]
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
    view: BoxedView<App>,
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
        Arc::new(Box::new(|_: &App| LiveSettings::default())),
        view.map(|view| {
            let boxed: Box<dyn Any + Send + Sync + 'static> = Box::new(view);
            Arc::new(boxed)
        }),
    )
}

#[cfg(all(feature = "wayland", target_os = "linux", feature = "winit"))]
#[must_use]
pub fn simple_layer_shell<Message: 'static>(
    live_settings: impl Fn() -> LiveSettings + Send + Sync + 'static,
    settings: impl Fn()
        -> iced_runtime::platform_specific::wayland::layer_surface::SctkLayerSurfaceSettings
    + Send
    + Sync
    + 'static,
    view: Option<
        impl Fn() -> crate::Element<'static, crate::Action<Message>> + Send + Sync + 'static,
    >,
) -> Action {
    let boxed: Box<
        dyn Fn()
                -> iced_runtime::platform_specific::wayland::layer_surface::SctkLayerSurfaceSettings
            + Send
            + Sync
            + 'static,
    > = Box::new(settings);
    let boxed: Box<dyn Any + Send + Sync + 'static> = Box::new(boxed);
    let boxed_live: Box<dyn Fn() -> LiveSettings + Send + Sync + 'static> = Box::new(live_settings);
    let boxed_live: Box<dyn Any + Send + Sync + 'static> = Box::new(boxed_live);
    Action::LayerShell(
        Arc::new(boxed),
        Arc::new(boxed_live),
        view.map(|view| {
            let boxed: Box<
                dyn Fn() -> crate::Element<'static, crate::Action<Message>> + Send + Sync + 'static,
            > = Box::new(view);
            let boxed: Box<dyn Any + Send + Sync + 'static> = Box::new(boxed);
            Arc::new(boxed)
        }),
    )
}

#[cfg(all(feature = "wayland", target_os = "linux", feature = "winit"))]
#[must_use]
pub fn app_layer_shell<App: Application>(
    live_settings: impl Fn(&App) -> LiveSettings + Send + Sync + 'static,
    settings: impl Fn(
        &mut App,
    )
        -> iced_runtime::platform_specific::wayland::layer_surface::SctkLayerSurfaceSettings
    + Send
    + Sync
    + 'static,
    // XXX Boxed trait object is required for less cumbersome type inference, but we box it anyways.
    view: BoxedView<App>,
) -> Action {
    let boxed: Box<
        dyn Fn(
                &mut App,
            )
                -> iced_runtime::platform_specific::wayland::layer_surface::SctkLayerSurfaceSettings
            + Send
            + Sync
            + 'static,
    > = Box::new(settings);
    let boxed: Box<dyn Any + Send + Sync + 'static> = Box::new(boxed);

    let boxed_live: Box<dyn Fn(&App) -> LiveSettings + Send + Sync + 'static> =
        Box::new(live_settings);
    let boxed_live: Box<dyn Any + Send + Sync + 'static> = Box::new(boxed_live);

    Action::AppLayerShell(
        Arc::new(boxed),
        Arc::new(boxed_live),
        view.map(|view| {
            let boxed: Box<dyn Any + Send + Sync + 'static> = Box::new(view);
            Arc::new(boxed)
        }),
    )
}
