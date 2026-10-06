// Copyright 2025 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use crate::direction::Direction;
use iced::Vector;
use iced::event::gamepad::{Axis, Button, Event};
use iced::window;
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub const DEADZONE: f32 = 0.2;

pub const SCROLL_SPEED: f32 = 1200.0;

pub const SCROLL_INTERVAL: Duration = Duration::from_millis(16);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ButtonIntent {
    /// Move focus in this direction.
    Move(Direction),
    /// Activate the focused widget.
    Activate,
    /// Cancel, or go back from, the focused widget.
    Cancel,
}

const DEFAULT_BUTTONS: &[(Button, ButtonIntent)] = &[
    (Button::DPadUp, ButtonIntent::Move(Direction::Up)),
    (Button::DPadDown, ButtonIntent::Move(Direction::Down)),
    (Button::DPadLeft, ButtonIntent::Move(Direction::Left)),
    (Button::DPadRight, ButtonIntent::Move(Direction::Right)),
    (Button::South, ButtonIntent::Activate),
    (Button::RightTrigger, ButtonIntent::Activate),
    (Button::East, ButtonIntent::Cancel),
];

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Config {
    /// Stick movement below this is ignored.
    pub deadzone: f32,
    /// Pixels per second while the left stick is at its full extent.
    pub scroll_speed: f32,
    /// Time between scrolls while the left stick is held.
    pub scroll_interval: Duration,
    /// What each button asks for when it is pressed.
    pub buttons: Vec<(Button, ButtonIntent)>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            deadzone: DEADZONE,
            scroll_speed: SCROLL_SPEED,
            scroll_interval: SCROLL_INTERVAL,
            buttons: DEFAULT_BUTTONS.to_vec(),
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct LeftStick(Vector);

impl LeftStick {
    pub fn update(&mut self, config: &Config, axis: Axis, value: f32) -> Vector {
        let value = if value.abs() < config.deadzone {
            0.0
        } else {
            value
        };

        match axis {
            Axis::LeftStickX | Axis::DPadX => self.0.x = value,
            Axis::LeftStickY | Axis::DPadY => self.0.y = -value,
            _ => return Vector::ZERO,
        }

        self.step(config, config.scroll_interval)
    }

    #[must_use]
    pub fn step(self, config: &Config, elapsed: Duration) -> Vector {
        self.0 * config.scroll_speed * elapsed.as_secs_f32()
    }

    #[must_use]
    pub fn scrolling(self) -> bool {
        self.0 != Vector::ZERO
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Intent {
    Move(Direction, Button),
    Stick(Axis, f32),
    Activate,
    Cancel,
}

#[must_use]
pub fn intent(event: &iced::Event, config: Option<&Config>) -> Option<Intent> {
    let iced::Event::Gamepad(event) = event else {
        return None;
    };

    match event {
        Event::ButtonPressed { button, .. } => {
            let toolkit = crate::config::gamepad();
            let config = config.unwrap_or(&toolkit);

            config
                .buttons
                .iter()
                .find(|(mapped, _)| mapped == button)
                .map(|(_, intent)| match intent {
                    ButtonIntent::Move(direction) => Intent::Move(*direction, *button),
                    ButtonIntent::Activate => Intent::Activate,
                    ButtonIntent::Cancel => Intent::Cancel,
                })
        }
        Event::AxisChanged { axis, value, .. } => Some(Intent::Stick(*axis, *value)),
        _ => None,
    }
}

#[must_use]
pub fn is_activate(event: &iced::Event) -> bool {
    intent(event, None) == Some(Intent::Activate)
}

#[must_use]
pub fn is_cancel(event: &iced::Event) -> bool {
    intent(event, None) == Some(Intent::Cancel)
}

#[must_use]
pub fn moves(event: &iced::Event, direction: Direction) -> bool {
    matches!(intent(event, None), Some(Intent::Move(moved, _)) if moved == direction)
}

pub(crate) fn scroll_subscription<Message: Send + 'static>(
    window_id: window::Id,
    interval: Duration,
) -> iced::Subscription<crate::Action<Message>> {
    #[cfg(all(feature = "winit", any(feature = "tokio", feature = "smol")))]
    {
        fn tick_action<Message: Send + 'static>(
            (window_id, _): (window::Id, iced::time::Instant),
        ) -> crate::Action<Message> {
            crate::Action::Cosmic(crate::app::Action::ScrollTick(window_id))
        }

        iced::time::every(interval)
            .with(window_id)
            .map(tick_action::<Message>)
    }

    #[cfg(not(all(feature = "winit", any(feature = "tokio", feature = "smol"))))]
    {
        let _ = (window_id, interval);

        iced::Subscription::none()
    }
}
