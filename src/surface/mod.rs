// Copyright 2025 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

pub mod action;
#[cfg(wayland_platform)]
pub mod corner_radius;

use iced::{Limits, Size, Task};
use std::any::Any;
use std::sync::Arc;

type BoxedSetting = Arc<Box<dyn Any + Send + Sync + 'static>>;

/// Produces the content of a surface created from within a widget.
///
/// Typed on the message the widget publishes.
pub type View<M> =
    Arc<dyn Fn() -> crate::Element<'static, crate::Action<M>> + Send + Sync + 'static>;

/// Ignore this message in your application. It will be intercepted.
///
/// `M` is the message type of whoever created the action. The ones prefixed with `App` take the
/// application itself and are type-erased, the others carry a [`View`] typed on `M`.
#[derive(Clone)]
pub enum Action<M> {
    /// Create a subsurface with a view function accepting the App as a parameter
    AppSubsurface(BoxedSetting, BoxedSetting, Option<BoxedSetting>),
    /// Create a subsurface with a view function
    Subsurface(BoxedSetting, BoxedSetting, Option<View<M>>),
    /// Destroy a subsurface with a view function
    DestroySubsurface(iced::window::Id),
    /// Create a popup with a view function accepting the App as a parameter
    AppPopup(BoxedSetting, BoxedSetting, Option<BoxedSetting>),
    /// Create a popup
    Popup(BoxedSetting, BoxedSetting, Option<View<M>>),
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
        Option<View<M>>,
    ),
    /// Destroy a window
    DestroyWindow(iced::window::Id),

    /// Create a layer shell surface with a view function accepting the App as a parameter
    AppLayerShell(BoxedSetting, BoxedSetting, Option<BoxedSetting>),

    /// Create a layer shell surface with a view function
    LayerShell(BoxedSetting, BoxedSetting, Option<View<M>>),

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
    Task(Arc<dyn Fn() -> Task<Action<M>> + Send + Sync>),
}

impl<M: 'static> Action<M> {
    /// Re-type the action for a component whose messages are wrapped by `f`.
    ///
    /// Similar to [`iced::Element::map`]. A component that maps a widget's messages must
    /// map the widget's surface actions too.
    #[must_use]
    pub fn map<N: 'static>(self, f: impl Fn(M) -> N + Clone + Send + Sync + 'static) -> Action<N> {
        self.map_actions(move |action| action.map(f.clone()))
    }

    fn map_actions<N: 'static>(
        self,
        g: impl Fn(crate::Action<M>) -> crate::Action<N> + Clone + Send + Sync + 'static,
    ) -> Action<N> {
        let map_view = |view: Option<View<M>>| -> Option<View<N>> {
            let view = view?;
            let g = g.clone();
            Some(Arc::new(move || view().map(g.clone())))
        };
        match self {
            Action::AppSubsurface(a, b, c) => Action::AppSubsurface(a, b, c),
            Action::Subsurface(a, b, view) => Action::Subsurface(a, b, map_view(view)),
            Action::DestroySubsurface(id) => Action::DestroySubsurface(id),
            Action::AppPopup(a, b, c) => Action::AppPopup(a, b, c),
            Action::Popup(a, b, view) => Action::Popup(a, b, map_view(view)),
            Action::DestroyPopup(id) => Action::DestroyPopup(id),
            Action::DestroyTooltipPopup => Action::DestroyTooltipPopup,
            Action::AppWindow(id, a, b, c) => Action::AppWindow(id, a, b, c),
            Action::Window(id, a, b, view) => Action::Window(id, a, b, map_view(view)),
            Action::DestroyWindow(id) => Action::DestroyWindow(id),
            Action::AppLayerShell(a, b, c) => Action::AppLayerShell(a, b, c),
            Action::LayerShell(a, b, view) => Action::LayerShell(a, b, map_view(view)),
            Action::DestroyLayerShell(id) => Action::DestroyLayerShell(id),
            Action::ResponsiveMenuBar {
                menu_bar,
                limits,
                size,
            } => Action::ResponsiveMenuBar {
                menu_bar,
                limits,
                size,
            },
            Action::Ignore => Action::Ignore,
            Action::SyncLiveSettings(id) => Action::SyncLiveSettings(id),
            Action::Task(task) => Action::Task(Arc::new(move || {
                let g = g.clone();
                task().map(move |action| action.map_actions(g.clone()))
            })),
        }
    }
}

impl<M: 'static> Action<crate::Action<M>> {
    #[must_use]
    pub fn flatten(self) -> Action<M> {
        self.map_actions(crate::Action::flatten)
    }
}

#[cfg(feature = "winit")]
pub fn surface_task<M: Send + 'static>(action: Action<M>) -> Task<crate::Action<M>> {
    crate::task::message(crate::Action::Surface(action))
}

impl<M> std::fmt::Debug for Action<M> {
    #[cold]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AppSubsurface(arg0, arg1, arg2) => f
                .debug_tuple("AppSubsurface")
                .field(arg0)
                .field(arg1)
                .field(arg2)
                .finish(),
            Self::Subsurface(arg0, arg1, view) => f
                .debug_tuple("Subsurface")
                .field(arg0)
                .field(arg1)
                .field(&view.as_ref().map(|_| "view"))
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
            Self::Popup(arg0, arg1, view) => f
                .debug_tuple("Popup")
                .field(arg0)
                .field(arg1)
                .field(&view.as_ref().map(|_| "view"))
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
            Self::Window(id, arg0, arg1, view) => f
                .debug_tuple("Window")
                .field(id)
                .field(arg0)
                .field(arg1)
                .field(&view.as_ref().map(|_| "view"))
                .finish(),
            Self::DestroyWindow(arg0) => f.debug_tuple("DestroyWindow").field(arg0).finish(),
            Self::Task(_) => f.debug_tuple("Future").finish(),
            Self::AppLayerShell(arg, arg1, arg2) => f
                .debug_tuple("AppLayerShell")
                .field(arg)
                .field(arg1)
                .field(arg2)
                .finish(),
            Self::LayerShell(arg0, arg1, view) => f
                .debug_tuple("LayerShell")
                .field(arg0)
                .field(arg1)
                .field(&view.as_ref().map(|_| "view"))
                .finish(),
            Self::DestroyLayerShell(arg0) => {
                f.debug_tuple("DestroyLayerShell").field(arg0).finish()
            }
            Self::SyncLiveSettings(arg0) => f.debug_tuple("SyncLiveSettings").field(arg0).finish(),
        }
    }
}
