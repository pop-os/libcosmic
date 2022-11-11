// SPDX-License-Identifier: GPL-3.0-or-later
use crate::dbus::PowerDaemonProxy;
use zbus::Result;

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum Graphics {
    Integrated,
    Hybrid,
    Nvidia,
    Compute,
}

pub async fn get_current_graphics(daemon: PowerDaemonProxy<'_>) -> Result<Graphics> {
    let graphics = daemon.get_graphics().await?;
    match graphics.as_str() {
        "integrated" => Ok(Graphics::Integrated),
        "hybrid" => Ok(Graphics::Hybrid),
        "nvidia" => Ok(Graphics::Nvidia),
        "compute" => Ok(Graphics::Compute),
        _ => panic!("Unknown graphics profile: {}", graphics),
    }
}

pub async fn set_graphics(daemon: PowerDaemonProxy<'_>, graphics: Graphics) -> Result<()> {
    let graphics_str = match graphics {
        Graphics::Integrated => "integrated",
        Graphics::Hybrid => "hybrid",
        Graphics::Nvidia => "nvidia",
        Graphics::Compute => "compute",
    };
    daemon.set_graphics(graphics_str).await
}
