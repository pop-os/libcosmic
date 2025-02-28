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
