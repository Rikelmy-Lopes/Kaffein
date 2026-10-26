//! Key bindings expressed in application terms.
//!
//! The UI maps [`Key`] to its own key type; the application layer never names
//! an egui key.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Key {
    N,
    O,
    S,
    Z,
    Y,
    Q,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KeyBinding {
    pub key: Key,
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
}

impl KeyBinding {
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

    pub fn display(self) -> String {
        let mut text = String::new();
        if self.ctrl {
            text.push_str("Ctrl+");
        }
        if self.shift {
            text.push_str("Shift+");
        }
        if self.alt {
            text.push_str("Alt+");
        }
        text.push(match self.key {
            Key::N => 'N',
            Key::O => 'O',
            Key::S => 'S',
            Key::Z => 'Z',
            Key::Y => 'Y',
            Key::Q => 'Q',
        });
        text
    }
}
