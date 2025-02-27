use iced::Size;

use iced::Limits;

pub enum MessageWrapper<M> {
    Surface(SurfaceMessage),
    Message(M),
}

#[cfg(not(feature = "surface-message"))]
impl<M> From<M> for MessageWrapper<M> {
    fn from(value: M) -> Self {
        MessageWrapper::Message(value)
    }
}

/// Ignore this message in your application. It will be intercepted.
#[derive(Clone)]
pub enum SurfaceMessage {
    /// Create a subsurface with a view function accepting the App as a parameter
    AppSubsurface(
        std::sync::Arc<Box<dyn std::any::Any + Send + Sync>>,
        Option<std::sync::Arc<Box<dyn std::any::Any + Send + Sync>>>,
    ),
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
    Ignore,
}

impl std::fmt::Debug for SurfaceMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AppSubsurface(arg0, arg1) => f
                .debug_tuple("AppSubsurface")
                .field(arg0)
                .field(arg1)
                .finish(),
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
            Self::Ignore => write!(f, "Ignore"),
        }
    }
}
