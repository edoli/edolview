use eframe::egui::{Key, KeyboardShortcut, ModifierNames, Modifiers};

type Ksc = KeyboardShortcut;

pub const IS_MAC: bool = cfg!(target_os = "macos");

pub const SELECT_ALL_SC: Ksc = Ksc::new(Modifiers::COMMAND, Key::A);
pub const SELECT_NONE_SC: Ksc = Ksc::new(Modifiers::NONE, Key::Escape);
pub const COPY_SC: Ksc = Ksc::new(Modifiers::COMMAND, Key::C);
pub const OPEN_FROM_CLIPBOARD_SC: Ksc = Ksc::new(Modifiers::CTRL.plus(Modifiers::SHIFT), Key::V);
pub const SAVE_IMAGE_SC: Ksc = Ksc::new(Modifiers::COMMAND, Key::S);
pub const BOOKMARK_PANEL_TOGGLE: Ksc = Ksc::new(Modifiers::NONE, Key::B);
pub const BOOKMARK_ADD: Ksc = Ksc::new(Modifiers::COMMAND, Key::B);
pub const BOOKMARK_PREV: Ksc = Ksc::new(Modifiers::NONE, Key::OpenBracket);
pub const BOOKMARK_NEXT: Ksc = Ksc::new(Modifiers::NONE, Key::CloseBracket);

pub const RESET_VIEW: Ksc = Ksc::new(Modifiers::NONE, Key::R);

pub const FULLSCREEN_TOGGLE: Ksc = Ksc::new(Modifiers::NONE, Key::F11);

pub const ZOOM_IN: Ksc = Ksc::new(Modifiers::NONE, Key::Equals);
pub const ZOOM_OUT: Ksc = Ksc::new(Modifiers::NONE, Key::Minus);

pub const NAVIGATE_PREV: Ksc = Ksc::new(Modifiers::NONE, Key::ArrowLeft);
pub const NAVIGATE_NEXT: Ksc = Ksc::new(Modifiers::NONE, Key::ArrowRight);
pub const NAVIGATE_ASSET_PREV: Ksc = Ksc::new(Modifiers::NONE, Key::PageUp);
pub const NAVIGATE_ASSET_NEXT: Ksc = Ksc::new(Modifiers::NONE, Key::PageDown);
pub const NAVIGATE_ASSET_PREV_TAB: Ksc = Ksc::new(Modifiers::CTRL.plus(Modifiers::SHIFT), Key::Tab);
pub const NAVIGATE_ASSET_NEXT_TAB: Ksc = Ksc::new(Modifiers::CTRL, Key::Tab);

pub const PRESET_KEYS: [Key; 9] = [
    Key::Num1,
    Key::Num2,
    Key::Num3,
    Key::Num4,
    Key::Num5,
    Key::Num6,
    Key::Num7,
    Key::Num8,
    Key::Num9,
];

const fn preset_shortcuts(modifiers: Modifiers) -> [Ksc; PRESET_KEYS.len()] {
    let mut shortcuts = [Ksc::new(Modifiers::NONE, Key::Num0); PRESET_KEYS.len()];
    let mut index = 0;
    while index < PRESET_KEYS.len() {
        shortcuts[index] = Ksc::new(modifiers, PRESET_KEYS[index]);
        index += 1;
    }
    shortcuts
}

pub const PRESET_SAVE_SHORTCUTS: [Ksc; PRESET_KEYS.len()] = preset_shortcuts(Modifiers::COMMAND);
pub const PRESET_APPLY_SHORTCUTS: [Ksc; PRESET_KEYS.len()] = preset_shortcuts(Modifiers::ALT);

pub const MODIFIER_NAMES: ModifierNames = ModifierNames {
    is_short: false,
    alt: "Alt",
    ctrl: "Ctrl",
    shift: "Shift",
    mac_cmd: "⌘",
    mac_alt: "Option",
    concat: "+",
};

pub trait KeyboardShortcutExt {
    fn format_sys(&self) -> String;
}

impl KeyboardShortcutExt for KeyboardShortcut {
    fn format_sys(&self) -> String {
        self.format(&MODIFIER_NAMES, IS_MAC)
    }
}
