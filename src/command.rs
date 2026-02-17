// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

#[cfg(feature = "xdg-portal")]
use std::os::fd::AsFd;

use iced::window;

/// Initiates a window drag.
pub fn drag<M>(id: window::Id) -> iced::Task<crate::Action<M>> {
    iced_runtime::window::drag(id)
}

/// Maximizes the window.
pub fn maximize<M>(id: window::Id, maximized: bool) -> iced::Task<crate::Action<M>> {
    iced_runtime::window::maximize(id, maximized)
}

/// Minimizes the window.
pub fn minimize<M>(id: window::Id) -> iced::Task<crate::Action<M>> {
    iced_runtime::window::minimize(id, true)
}

/// Sets the title of a window.
#[allow(unused_variables, clippy::needless_pass_by_value)]
pub fn set_title<M>(id: window::Id, title: String) -> iced::Task<crate::Action<M>> {
    iced::Task::none()
}

#[cfg(feature = "winit")]
pub fn set_scaling_factor<M: Send + 'static>(factor: f32) -> iced::Task<crate::Action<M>> {
    iced::Task::done(crate::app::Action::ScaleFactor(factor)).map(crate::Action::Cosmic)
}

#[cfg(feature = "winit")]
pub fn set_theme<M: Send + 'static>(theme: crate::Theme) -> iced::Task<crate::Action<M>> {
    iced::Task::done(crate::app::Action::AppThemeChange(theme)).map(crate::Action::Cosmic)
}

/// Sets the window mode to windowed.
pub fn set_windowed<M>(id: window::Id) -> iced::Task<crate::Action<M>> {
    iced_runtime::window::change_mode(id, window::Mode::Windowed)
}

/// Toggles the windows' maximize state.
pub fn toggle_maximize<M>(id: window::Id) -> iced::Task<crate::Action<M>> {
    iced_runtime::window::toggle_maximize(id)
}

#[cfg(feature = "xdg-portal")]
pub fn file_transfer_send(
    writeable: bool,
    auto_stop: bool,
    files: Vec<impl AsFd + Send + Sync + 'static>,
) -> iced::Task<ashpd::Result<String>> {
    iced::Task::future(async move {
        let file_transfer = ashpd::documents::FileTransfer::new().await?;
        let key = file_transfer.start_transfer(writeable, auto_stop).await?;
        file_transfer.add_files(&key, &files).await?;
        Ok(key)
    })
}

/// Receive the files offered over the xdg share portal using the `key`.
/// Returns a list of file paths.
#[cfg(feature = "xdg-portal")]
pub fn file_transfer_receive(key: String) -> iced::Task<ashpd::Result<Vec<String>>> {
    dbg!(&key);
    iced::Task::future(async move {
        let file_transfer = ashpd::documents::FileTransfer::new().await?;
        file_transfer.retrieve_files(&key).await
    })
}
