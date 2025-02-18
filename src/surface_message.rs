use iced::Size;

use iced::Limits;

pub enum MessageWrapper<M> {
    Surface(SurfaceMessage),
    Message(M),
}

pub trait SurfaceMessageHandler: Sized {
    fn to_surface_message(self) -> MessageWrapper<Self>;
}

#[cfg(not(feature = "wayland"))]
impl<M> SurfaceMessageHandler for M {
    fn to_surface_message(self) -> MessageWrapper<Self> {
        MessageWrapper::Message(self)
    }
}

/// Ignore this message in your application. It will be intercepted.
#[derive(Clone)]
pub enum SurfaceMessage {
    /// Create a subsurface with a view function
    Subsurface(
        std::sync::Arc<Box<dyn std::any::Any + Send + Sync>>,
        Option<std::sync::Arc<Box<dyn std::any::Any + Send + Sync>>>,
    ),
    /// Destroy a subsurface with a view function
    DestroySubsurface(iced::window::Id),
    /// Create a popup with a view function accepting the App as a parameter
    AppPopup(
        std::sync::Arc<Box<dyn std::any::Any + Send + Sync>>,
        Option<std::sync::Arc<Box<dyn std::any::Any + Send + Sync>>>,
    ),
    /// Create a popup
    Popup(
        std::sync::Arc<Box<dyn std::any::Any + Send + Sync>>,
        Option<std::sync::Arc<Box<dyn std::any::Any + Send + Sync>>>,
    ),
    /// Destroy a subsurface with a view function
    DestroyPopup(iced::window::Id),
    /// Responsive menu bar update
    ResponsiveMenuBar {
        /// Id of the menu bar
        menu_bar: crate::widget::Id,
        /// Limits of the menu bar
        limits: Limits,
        /// Requested Full Size for expanded menu bar
        size: Size,
    },
}

impl std::fmt::Debug for SurfaceMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Subsurface(arg0, arg1) => {
                f.debug_tuple("Subsurface").field(arg0).field(arg1).finish()
            }
            Self::DestroySubsurface(arg0) => {
                f.debug_tuple("DestroySubsurface").field(arg0).finish()
            }
            Self::AppPopup(arg0, arg1) => {
                f.debug_tuple("AppPopup").field(arg0).field(arg1).finish()
            }
            Self::Popup(arg0, arg1) => f.debug_tuple("Popup").field(arg0).field(arg1).finish(),
            Self::DestroyPopup(arg0) => f.debug_tuple("DestroyPopup").field(arg0).finish(),
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
        }
    }
}
