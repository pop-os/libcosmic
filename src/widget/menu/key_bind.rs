use iced_core::keyboard::{Key, Modifiers, key::Code, key::Physical};
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

    pub fn matches_physical(&self, modifiers: Modifiers, physical_key: Physical) -> bool {
        let key_eq = match (&self.key, physical_key) {
            (Key::Character(expected), Physical::Code(actual)) => physical_key_eq(expected, actual),
            _ => false,
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

fn physical_key_eq(expected: &str, code: Code) -> bool {
    match code {
        Code::KeyA => expected.eq_ignore_ascii_case("a"),
        Code::KeyB => expected.eq_ignore_ascii_case("b"),
        Code::KeyC => expected.eq_ignore_ascii_case("c"),
        Code::KeyD => expected.eq_ignore_ascii_case("d"),
        Code::KeyE => expected.eq_ignore_ascii_case("e"),
        Code::KeyF => expected.eq_ignore_ascii_case("f"),
        Code::KeyG => expected.eq_ignore_ascii_case("g"),
        Code::KeyH => expected.eq_ignore_ascii_case("h"),
        Code::KeyI => expected.eq_ignore_ascii_case("i"),
        Code::KeyJ => expected.eq_ignore_ascii_case("j"),
        Code::KeyK => expected.eq_ignore_ascii_case("k"),
        Code::KeyL => expected.eq_ignore_ascii_case("l"),
        Code::KeyM => expected.eq_ignore_ascii_case("m"),
        Code::KeyN => expected.eq_ignore_ascii_case("n"),
        Code::KeyO => expected.eq_ignore_ascii_case("o"),
        Code::KeyP => expected.eq_ignore_ascii_case("p"),
        Code::KeyQ => expected.eq_ignore_ascii_case("q"),
        Code::KeyR => expected.eq_ignore_ascii_case("r"),
        Code::KeyS => expected.eq_ignore_ascii_case("s"),
        Code::KeyT => expected.eq_ignore_ascii_case("t"),
        Code::KeyU => expected.eq_ignore_ascii_case("u"),
        Code::KeyV => expected.eq_ignore_ascii_case("v"),
        Code::KeyW => expected.eq_ignore_ascii_case("w"),
        Code::KeyX => expected.eq_ignore_ascii_case("x"),
        Code::KeyY => expected.eq_ignore_ascii_case("y"),
        Code::KeyZ => expected.eq_ignore_ascii_case("z"),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::{KeyBind, Modifier};
    use iced_core::keyboard::{Key, Modifiers, key::Code, key::Physical};

    #[test]
    fn physical_shortcut_matches_layout_independent_key() {
        let key_bind = KeyBind {
            modifiers: vec![Modifier::Ctrl],
            key: Key::Character("c".into()),
        };

        assert!(key_bind.matches_physical(Modifiers::CTRL, Physical::Code(Code::KeyC)));
    }

    #[test]
    fn physical_shortcut_rejects_wrong_physical_key() {
        let key_bind = KeyBind {
            modifiers: vec![Modifier::Ctrl],
            key: Key::Character("c".into()),
        };

        assert!(!key_bind.matches_physical(Modifiers::CTRL, Physical::Code(Code::KeyV)));
    }

    #[test]
    fn physical_shortcut_rejects_wrong_modifiers() {
        let key_bind = KeyBind {
            modifiers: vec![Modifier::Ctrl],
            key: Key::Character("c".into()),
        };

        assert!(!key_bind.matches_physical(Modifiers::SHIFT, Physical::Code(Code::KeyC)));
    }

    #[test]
    fn logical_shortcut_still_matches_by_character() {
        let key_bind = KeyBind {
            modifiers: vec![Modifier::Ctrl],
            key: Key::Character(",".into()),
        };

        assert!(key_bind.matches(Modifiers::CTRL, &Key::Character(",".into())));
    }
}
