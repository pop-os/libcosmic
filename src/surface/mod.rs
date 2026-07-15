// Copyright 2025 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

pub mod action;
#[cfg(wayland_platform)]
pub mod corner_radius;

use iced::{Limits, Size, Task};
use std::any::Any;
use std::sync::Arc;

type BoxedSetting = Arc<Box<dyn Any + Send + Sync + 'static>>;

/// Ignore this message in your application. It will be intercepted.
#[derive(Clone)]
pub enum Action {
    /// Create a subsurface with a view function accepting the App as a parameter
    AppSubsurface(BoxedSetting, BoxedSetting, Option<BoxedSetting>),
    /// Create a subsurface with a view function
    Subsurface(BoxedSetting, BoxedSetting, Option<BoxedSetting>),
    /// Destroy a subsurface with a view function
    DestroySubsurface(iced::window::Id),
    /// Create a popup with a view function accepting the App as a parameter
    AppPopup(BoxedSetting, BoxedSetting, Option<BoxedSetting>),
    /// Create a popup
    Popup(BoxedSetting, BoxedSetting, Option<BoxedSetting>),
    /// Destroy a subsurface with a view function
    DestroyPopup(iced::window::Id),
    /// Destroys the global tooltip popup subsurface
    DestroyTooltipPopup,

    /// Create a window with a view function accepting the App as a parameter
    AppWindow(
        iced::window::Id,
        BoxedSetting,
        BoxedSetting,
        Option<BoxedSetting>,
    ),
    /// Create a window with a view function
    Window(
        iced::window::Id,
        BoxedSetting,
        BoxedSetting,
        Option<BoxedSetting>,
    ),
    /// Destroy a window
    DestroyWindow(iced::window::Id),

    /// Create a layer shell surface with a view function accepting the App as a parameter
    AppLayerShell(BoxedSetting, BoxedSetting, Option<BoxedSetting>),

    /// Create a layer shell surface with a view function
    LayerShell(BoxedSetting, BoxedSetting, Option<BoxedSetting>),

    /// Destroy a layer shell surface
    DestroyLayerShell(iced::window::Id),

    /// Responsive menu bar update
    ResponsiveMenuBar {
        /// Id of the menu bar
        menu_bar: crate::widget::Id,
        /// Limits of the menu bar
        limits: Limits,
        /// Requested Full Size for expanded menu bar
        size: Size,
    },
    Ignore,
    SyncLiveSettings(iced::window::Id),
    Task(Arc<dyn Fn() -> Task<Action> + Send + Sync>),
}

#[cfg(feature = "winit")]
pub fn surface_task<M: Send + 'static>(action: Action) -> Task<crate::Action<M>> {
    crate::task::message(crate::Action::Cosmic(crate::app::Action::Surface(action)))
}

impl std::fmt::Debug for Action {
    #[cold]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AppSubsurface(arg0, arg1, arg2) => f
                .debug_tuple("AppSubsurface")
                .field(arg0)
                .field(arg1)
                .field(arg2)
                .finish(),
            Self::Subsurface(arg0, arg1, arg2) => f
                .debug_tuple("Subsurface")
                .field(arg0)
                .field(arg1)
                .field(arg2)
                .finish(),
            Self::DestroySubsurface(arg0) => {
                f.debug_tuple("DestroySubsurface").field(arg0).finish()
            }
            Self::AppPopup(arg0, arg1, arg2) => f
                .debug_tuple("AppPopup")
                .field(arg0)
                .field(arg1)
                .field(arg2)
                .finish(),
            Self::Popup(arg0, arg1, arg2) => f
                .debug_tuple("Popup")
                .field(arg0)
                .field(arg1)
                .field(arg2)
                .finish(),
            Self::DestroyPopup(arg0) => f.debug_tuple("DestroyPopup").field(arg0).finish(),
            Self::DestroyTooltipPopup => f.debug_tuple("DestroyTooltipPopup").finish(),
            Self::ResponsiveMenuBar {
                menu_bar,
                limits,
                size,
            } => f
                .debug_struct("ResponsiveMenuBar")
                .field("menu_bar", menu_bar)
                .field("limits", limits)
                .field("size", size)
                .finish(),
            Self::Ignore => write!(f, "Ignore"),
            Self::AppWindow(id, arg0, arg1, arg2) => f
                .debug_tuple("AppWindow")
                .field(id)
                .field(arg0)
                .field(arg1)
                .field(arg2)
                .finish(),
            Self::Window(id, arg0, arg1, arg2) => f
                .debug_tuple("Window")
                .field(id)
                .field(arg0)
                .field(arg1)
                .field(arg2)
                .finish(),
            Self::DestroyWindow(arg0) => f.debug_tuple("DestroyWindow").field(arg0).finish(),
            Self::Task(_) => f.debug_tuple("Future").finish(),
            Self::AppLayerShell(arg, arg1, arg2) => f
                .debug_tuple("AppLayerShell")
                .field(arg)
                .field(arg1)
                .field(arg2)
                .finish(),
            Self::LayerShell(arg, arg1, arg2) => f
                .debug_tuple("LayerShell")
                .field(arg)
                .field(arg1)
                .field(arg2)
                .finish(),
            Self::DestroyLayerShell(arg0) => {
                f.debug_tuple("DestroyLayerShell").field(arg0).finish()
            }
            Self::SyncLiveSettings(arg0) => f.debug_tuple("SyncLiveSettings").field(arg0).finish(),
        }
    }
}
