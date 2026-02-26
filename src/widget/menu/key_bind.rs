use iced_core::keyboard::key::{Code, Physical};
use iced_core::keyboard::{Key, Modifiers};
use std::fmt;

/// Represents the modifier keys on a keyboard.
///
/// It has four variants:
/// * `Super`: Represents the Super key (also known as the Windows key on Windows, Command key on macOS).
/// * `Ctrl`: Represents the Control key.
/// * `Alt`: Represents the Alt key.
/// * `Shift`: Represents the Shift key.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Modifier {
    Super,
    Ctrl,
    Alt,
    Shift,
}

/// Represents a combination of a key and modifiers.
/// It is used to define keyboard shortcuts.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct KeyBind {
    /// A vector of modifiers for the key binding.
    pub modifiers: Vec<Modifier>,
    /// The key for the key binding.
    pub key: Key,
}

impl KeyBind {
    /// Checks if the given key and modifiers match the `KeyBind`.
    ///
    /// # Arguments
    ///
    /// * `modifiers` - A `Modifiers` instance representing the current active modifiers.
    /// * `key` - A reference to the `Key` that is being checked.
    ///
    /// # Returns
    ///
    /// * `bool` - `true` if the key and modifiers match the `KeyBind`, `false` otherwise.
    #[deprecated(note = "Use `matches_layout_aware` instead for correct non-Latin keyboard layout support")]
    pub fn matches(&self, modifiers: Modifiers, key: &Key) -> bool {
        let key_eq = match (key, &self.key) {
            // CapsLock and Shift change the case of Key::Character, so we compare these in a case insensitive way
            (Key::Character(a), Key::Character(b)) => a.eq_ignore_ascii_case(b),
            (a, b) => a.eq(b),
        };
        key_eq
            && modifiers.logo() == self.modifiers.contains(&Modifier::Super)
            && modifiers.control() == self.modifiers.contains(&Modifier::Ctrl)
            && modifiers.alt() == self.modifiers.contains(&Modifier::Alt)
            && modifiers.shift() == self.modifiers.contains(&Modifier::Shift)
    }

    /// Checks if the given key and modifiers match the `KeyBind`, with a
    /// fallback to the physical key position for non-Latin keyboard layouts.
    ///
    /// This is the recommended replacement for [`Self::matches`], which does not
    /// handle non-Latin layouts correctly.
    ///
    /// # Arguments
    ///
    /// * `modifiers` - A `Modifiers` instance representing the current active modifiers.
    /// * `key` - A reference to the `Key` that is being checked.
    /// * `physical_key` - An optional reference to the physical key position,
    ///   used as a fallback when the logical `key` does not match (e.g. on
    ///   Cyrillic or other non-Latin layouts). Can be `None` for keys where
    ///   the physical position is not relevant (e.g. `Key::Named`).
    ///
    /// # Returns
    ///
    /// * `bool` - `true` if the key and modifiers match the `KeyBind`, `false` otherwise.
    #[allow(deprecated)]
    pub fn matches_layout_aware(
        &self,
        modifiers: Modifiers,
        key: &Key,
        physical_key: Option<&Physical>,
    ) -> bool {
        self.matches(modifiers, key)
            || physical_key
                .and_then(physical_key_to_latin)
                .map(|latin| self.matches(modifiers, &latin))
                .unwrap_or(false)
    }
}

/// Converts a physical key code to the corresponding US-layout Latin `Key`.
///
/// This mapping is intentionally limited to keys that may produce different
/// characters on non-Latin keyboard layouts (letters and punctuation). Keys
/// like digits are not included because they remain the same across layouts.
///
/// Only used as a fallback when the primary key comparison in
/// [`KeyBind::matches`] does not match.
fn physical_key_to_latin(physical_key: &Physical) -> Option<Key> {
    let code = match physical_key {
        Physical::Code(code) => code,
        Physical::Unidentified(_) => return None,
    };
    let ch = match code {
        Code::KeyA => "a",
        Code::KeyB => "b",
        Code::KeyC => "c",
        Code::KeyD => "d",
        Code::KeyE => "e",
        Code::KeyF => "f",
        Code::KeyG => "g",
        Code::KeyH => "h",
        Code::KeyI => "i",
        Code::KeyJ => "j",
        Code::KeyK => "k",
        Code::KeyL => "l",
        Code::KeyM => "m",
        Code::KeyN => "n",
        Code::KeyO => "o",
        Code::KeyP => "p",
        Code::KeyQ => "q",
        Code::KeyR => "r",
        Code::KeyS => "s",
        Code::KeyT => "t",
        Code::KeyU => "u",
        Code::KeyV => "v",
        Code::KeyW => "w",
        Code::KeyX => "x",
        Code::KeyY => "y",
        Code::KeyZ => "z",
        Code::Minus => "-",
        Code::Equal => "=",
        Code::BracketLeft => "[",
        Code::BracketRight => "]",
        Code::Backslash => "\\",
        Code::Semicolon => ";",
        Code::Quote => "'",
        Code::Backquote => "`",
        Code::Comma => ",",
        Code::Period => ".",
        Code::Slash => "/",
        _ => return None,
    };
    Some(Key::Character(ch.into()))
}

impl fmt::Display for KeyBind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for modifier in self.modifiers.iter() {
            write!(f, "{:?} + ", modifier)?;
        }
        match &self.key {
            Key::Character(c) => write!(f, "{}", c.to_uppercase()),
            Key::Named(named) => write!(f, "{:?}", named),
            other => write!(f, "{:?}", other),
        }
    }
}
