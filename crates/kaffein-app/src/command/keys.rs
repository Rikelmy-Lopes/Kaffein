//! Key bindings expressed in application terms.
//!
//! The UI maps [`Key`] to its own key type; the application layer never names
//! an egui key.

use kaffein_i18n::{keys, Localizer};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Key {
    N,
    O,
    S,
    Z,
    Y,
    Q,
    F5,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KeyBinding {
    pub key: Key,
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
}

impl KeyBinding {
    pub const fn plain(key: Key) -> Self {
        Self {
            key,
            ctrl: false,
            shift: false,
            alt: false,
        }
    }

    pub const fn ctrl(key: Key) -> Self {
        Self {
            key,
            ctrl: true,
            shift: false,
            alt: false,
        }
    }

    pub const fn ctrl_shift(key: Key) -> Self {
        Self {
            key,
            ctrl: true,
            shift: true,
            alt: false,
        }
    }

    pub fn display(self, localizer: &Localizer) -> String {
        let mut parts = Vec::with_capacity(3);
        if self.ctrl {
            parts.push(localizer.text(keys::KEYBINDING_CTRL));
        }
        if self.shift {
            parts.push(localizer.text(keys::KEYBINDING_SHIFT));
        }
        if self.alt {
            parts.push(localizer.text(keys::KEYBINDING_ALT));
        }
        parts.push(match self.key {
            Key::N => "N",
            Key::O => "O",
            Key::S => "S",
            Key::Z => "Z",
            Key::Y => "Y",
            Key::Q => "Q",
            Key::F5 => "F5",
        });
        parts.join("+")
    }
}
