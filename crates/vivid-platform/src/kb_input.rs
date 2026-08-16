use log::trace;
use winit::event::ElementState;
use winit::keyboard::KeyCode;

macro_rules! define_keys {
    ($($name:ident),* $(,)?) => {
        #[repr(u8)]
        #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
        pub enum Key { $($name),* }

        impl Key {
            pub const COUNT: usize = [$(Key::$name),*].len();

            /// Boundary translation. [`None`] = "vivid doesn't track this key".
            pub const fn from_winit(code: KeyCode) -> Option<Key> {
                match code {
                    $(KeyCode::$name => Some(Key::$name),)*
                    _ => None,
                }
            }
        }
    };
}

define_keys! {
    // Letters
    KeyA, KeyB, KeyC, KeyD, KeyE, KeyF, KeyG, KeyH, KeyI, KeyJ, KeyK, KeyL,
    KeyM, KeyN, KeyO, KeyP, KeyQ, KeyR, KeyS, KeyT, KeyU, KeyV, KeyW, KeyX,
    KeyY, KeyZ,
    // Top-row digits
    Digit0, Digit1, Digit2, Digit3, Digit4, Digit5, Digit6, Digit7, Digit8,
    Digit9,
    // Function row
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,
    // Arrows
    ArrowUp, ArrowDown, ArrowLeft, ArrowRight,
    // Whitespace & control
    Space, Enter, Tab, Backspace, Escape,
    // Modifiers
    ShiftLeft, ShiftRight, ControlLeft, ControlRight, AltLeft, AltRight,
    SuperLeft, SuperRight, CapsLock,
    // Punctuation (US-physical positions)
    Minus, Equal, BracketLeft, BracketRight, Backslash, Semicolon, Quote,
    Backquote, Comma, Period, Slash,
    // Navigation cluster
    Insert, Delete, Home, End, PageUp, PageDown,
}

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
struct KeySet {
    words: [u64; Key::COUNT.div_ceil(64)],
}

impl KeySet {
    const fn split(key: Key) -> (usize, u64) {
        let i = key as usize;
        (i / 64, 1u64 << (i % 64))
    }

    const fn insert(&mut self, key: Key) -> bool {
        let (w, bit) = Self::split(key);
        let was_set = self.words[w] & bit != 0;
        self.words[w] |= bit;
        !was_set
    }

    const fn remove(&mut self, key: Key) {
        let (w, bit) = Self::split(key);
        self.words[w] &= !bit;
    }

    const fn contains(self, key: Key) -> bool {
        let (w, bit) = Self::split(key);
        self.words[w] & bit != 0
    }

    const fn clear(&mut self) {
        self.words = [0; Key::COUNT.div_ceil(64)];
    }

    fn union(&mut self, other: &Self) {
        for (a, b) in self.words.iter_mut().zip(other.words.iter()) {
            *a |= *b;
        }
    }
}

/// The state of the keyboard.
#[derive(Clone, Copy, Default, Debug)]
pub struct KeyboardInput {
    /// The keys being hold down now.
    held: KeySet,
    /// The keys that are pressed (went down since the last pump).
    pressed: KeySet,
}

impl KeyboardInput {
    pub fn held(&self, key: Key) -> bool {
        self.held.contains(key)
    }

    /// True only on the first pump after the key went down.
    pub fn pressed(&self, key: Key) -> bool {
        self.pressed.contains(key)
    }

    pub(crate) fn begin_frame(&mut self) {
        self.pressed.clear();
    }

    pub(crate) fn on_key(&mut self, key: Key, state: ElementState, repeat: bool) {
        match state {
            ElementState::Pressed => {
                if !repeat && self.held.insert(key) {
                    self.pressed.insert(key);
                    trace!("key pressed: {key:?}");
                }
            }
            ElementState::Released => {
                self.held.remove(key);
                trace!("key released: {key:?}");
            }
        }
    }

    /// OR another frame's key state into this one. Used by the engine to
    /// accumulate presses across pumps that ran no simulation tick, so an
    /// edge-triggered input is never silently dropped.
    pub fn merge(&mut self, other: &Self) {
        self.held = other.held;
        self.pressed.union(&other.pressed);
    }

    /// Drop accumulated press edges, keeping current held state.
    pub fn clear_presses(&mut self) {
        self.pressed.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn count_fits_storage() {
        assert!(Key::COUNT > 0 && (Key::COUNT <= 64 * Key::COUNT.div_ceil(64)));
    }

    #[test]
    fn press_hold_release_lifecycle() {
        let mut input = KeyboardInput::default();

        input.on_key(Key::Space, ElementState::Pressed, false);
        assert!(input.pressed(Key::Space));
        assert!(input.held(Key::Space));

        input.begin_frame();
        assert!(!input.pressed(Key::Space), "pressed is one-frame only");
        assert!(input.held(Key::Space), "held persists across frames");

        input.on_key(Key::Space, ElementState::Released, false);
        assert!(!input.held(Key::Space));
    }

    #[test]
    fn os_repeat_does_not_retrigger_pressed() {
        let mut input = KeyboardInput::default();
        input.on_key(Key::KeyW, ElementState::Pressed, false);
        input.begin_frame();

        input.on_key(Key::KeyW, ElementState::Pressed, true); // OS auto-repeat
        assert!(!input.pressed(Key::KeyW));
        assert!(input.held(Key::KeyW));
    }

    #[test]
    fn high_bit_keys_use_second_word() {
        // Exercise a key with discriminant >= 64 so both words are covered.
        let key = Key::PageDown;
        assert!(key as usize >= 64, "test assumes PageDown lands in word 1");

        let mut set = KeySet::default();
        assert!(set.insert(key));
        assert!(set.contains(key));
        set.remove(key);
        assert!(!set.contains(key));
    }

    #[test]
    fn from_winit_maps_and_filters() {
        assert_eq!(Key::from_winit(KeyCode::Escape), Some(Key::Escape));
        // A key vivid deliberately doesn't track:
        assert_eq!(Key::from_winit(KeyCode::NumLock), None);
    }
}
