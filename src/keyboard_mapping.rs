use std::str::FromStr;
use std::io::Write;
use strum::IntoEnumIterator;

#[allow(dead_code)]
#[derive(Copy, Clone, PartialEq, Eq, EnumString, EnumIter, Display)]
pub enum Keycode {
    None = 0x00,
    Escape = 0x01,
    Key1 = 0x02,
    Key2 = 0x03,
    Key3 = 0x04,
    Key4 = 0x05,
    Key5 = 0x06,
    Key6 = 0x07,
    Key7 = 0x08,
    Key8 = 0x09,
    Key9 = 0x0A,
    Key0 = 0x0B,
    Minus = 0x0C,
    Equals = 0x0D,
    Back = 0x0E,
    Tab = 0x0F,
    Q = 0x10,
    W = 0x11,
    E = 0x12,
    R = 0x13,
    T = 0x14,
    Y = 0x15,
    U = 0x16,
    I = 0x17,
    O = 0x18,
    P = 0x19,
    LBracket = 0x1A,
    RBracket = 0x1B,
    Return = 0x1C,
    LControl = 0x1D,
    A = 0x1E,
    S = 0x1F,
    D = 0x20,
    F = 0x21,
    G = 0x22,
    H = 0x23,
    J = 0x24,
    K = 0x25,
    L = 0x26,
    Semicolon = 0x27,
    Apostrophe = 0x28,
    Gravis = 0x29,
    LShift = 0x2A,
    Backslash = 0x2B,
    Z = 0x2C,
    X = 0x2D,
    C = 0x2E,
    V = 0x2F,
    B = 0x30,
    N = 0x31,
    M = 0x32,
    Comma = 0x33,
    Period = 0x34,
    Slash = 0x35,
    RShift = 0x36,
    Multiply = 0x37,
    LAlt = 0x38,
    Space = 0x39,
    CapsLock = 0x3A,
    F1 = 0x3B,
    F2 = 0x3C,
    F3 = 0x3D,
    F4 = 0x3E,
    F5 = 0x3F,
    F6 = 0x40,
    F7 = 0x41,
    F8 = 0x42,
    F9 = 0x43,
    F10 = 0x44,
    NumLock = 0x45,
    ScrollLock = 0x46,
    Numpad7 = 0x47,
    Numpad8 = 0x48,
    Numpad9 = 0x49,
    Subtract = 0x4A,
    Numpad4 = 0x4B,
    Numpad5 = 0x4C,
    Numpad6 = 0x4D,
    Add = 0x4E,
    Numpad1 = 0x4F,
    Numpad2 = 0x50,
    Numpad3 = 0x51,
    Numpad0 = 0x52,
    Delete = 0x53,
    Home = 0xC7,
    Up = 0xC8,
    PageUp = 0xC9,
    Left = 0xCB,
    Right = 0xCD,
    End = 0xCF,
    Down = 0xD0,
    PageDown = 0xD1,
    Insert = 0xD2
}

pub struct KeyLayout {
    pub render_colum: u8,
    pub render_row: u8,
    pub position_x: u8,
    pub position_y: u8
}

const KEY_LAYOUT: [KeyLayout; 96] = [
    KeyLayout { render_colum: 0, render_row: 0, position_x: 0xFF, position_y: 0xFF },
    KeyLayout { render_colum: 125, render_row: 3, position_x: 19, position_y: 0 },
    KeyLayout { render_colum: 23, render_row: 3, position_x: 3, position_y: 0 },
    KeyLayout { render_colum: 29, render_row: 3, position_x: 4, position_y: 0 },
    KeyLayout { render_colum: 35, render_row: 3, position_x: 5, position_y: 0 },
    KeyLayout { render_colum: 41, render_row: 3, position_x: 6, position_y: 0 },
    KeyLayout { render_colum: 47, render_row: 3, position_x: 7, position_y: 0 },
    KeyLayout { render_colum: 53, render_row: 3, position_x: 8, position_y: 0 },
    KeyLayout { render_colum: 59, render_row: 3, position_x: 9, position_y: 0 },
    KeyLayout { render_colum: 65, render_row: 3, position_x: 10, position_y: 0 },
    KeyLayout { render_colum: 71, render_row: 3, position_x: 11, position_y: 0 },
    KeyLayout { render_colum: 77, render_row: 3, position_x: 12, position_y: 0 },
    KeyLayout { render_colum: 83, render_row: 3, position_x: 13, position_y: 0 },
    KeyLayout { render_colum: 89, render_row: 3, position_x: 14, position_y: 0 },
    KeyLayout { render_colum: 96, render_row: 3, position_x: 15, position_y: 0 },
    KeyLayout { render_colum: 18, render_row: 6, position_x: 2, position_y: 1 },
    KeyLayout { render_colum: 25, render_row: 6, position_x: 3, position_y: 1 },
    KeyLayout { render_colum: 31, render_row: 6, position_x: 4, position_y: 1 },
    KeyLayout { render_colum: 37, render_row: 6, position_x: 5, position_y: 1 },
    KeyLayout { render_colum: 43, render_row: 6, position_x: 6, position_y: 1 },
    KeyLayout { render_colum: 49, render_row: 6, position_x: 7, position_y: 1 },
    KeyLayout { render_colum: 55, render_row: 6, position_x: 8, position_y: 1 },
    KeyLayout { render_colum: 61, render_row: 6, position_x: 9, position_y: 1 },
    KeyLayout { render_colum: 67, render_row: 6, position_x: 10, position_y: 1 },
    KeyLayout { render_colum: 73, render_row: 6, position_x: 11, position_y: 1 },
    KeyLayout { render_colum: 79, render_row: 6, position_x: 12, position_y: 1 },
    KeyLayout { render_colum: 85, render_row: 6, position_x: 13, position_y: 1 },
    KeyLayout { render_colum: 91, render_row: 6, position_x: 14, position_y: 1 },
    KeyLayout { render_colum: 94, render_row: 9, position_x: 15, position_y: 2 },
    KeyLayout { render_colum: 18, render_row: 9, position_x: 2, position_y: 2 },
    KeyLayout { render_colum: 26, render_row: 9, position_x: 3, position_y: 2 },
    KeyLayout { render_colum: 32, render_row: 9, position_x: 4, position_y: 2 },
    KeyLayout { render_colum: 38, render_row: 9, position_x: 5, position_y: 2 },
    KeyLayout { render_colum: 44, render_row: 9, position_x: 6, position_y: 2 },
    KeyLayout { render_colum: 50, render_row: 9, position_x: 7, position_y: 2 },
    KeyLayout { render_colum: 56, render_row: 9, position_x: 8, position_y: 2 },
    KeyLayout { render_colum: 62, render_row: 9, position_x: 9, position_y: 2 },
    KeyLayout { render_colum: 68, render_row: 9, position_x: 10, position_y: 2 },
    KeyLayout { render_colum: 74, render_row: 9, position_x: 11, position_y: 2 },
    KeyLayout { render_colum: 80, render_row: 9, position_x: 12, position_y: 2 },
    KeyLayout { render_colum: 86, render_row: 9, position_x: 13, position_y: 2 },
    KeyLayout { render_colum: 17, render_row: 3, position_x: 2, position_y: 0 },
    KeyLayout { render_colum: 20, render_row: 12, position_x: 2, position_y: 3 },
    KeyLayout { render_colum: 97, render_row: 6, position_x: 15, position_y: 1 },
    KeyLayout { render_colum: 29, render_row: 12, position_x: 3, position_y: 3 },
    KeyLayout { render_colum: 35, render_row: 12, position_x: 4, position_y: 3 },
    KeyLayout { render_colum: 41, render_row: 12, position_x: 5, position_y: 3 },
    KeyLayout { render_colum: 47, render_row: 12, position_x: 6, position_y: 3 },
    KeyLayout { render_colum: 53, render_row: 12, position_x: 7, position_y: 3 },
    KeyLayout { render_colum: 59, render_row: 12, position_x: 8, position_y: 3 },
    KeyLayout { render_colum: 65, render_row: 12, position_x: 9, position_y: 3 },
    KeyLayout { render_colum: 71, render_row: 12, position_x: 10, position_y: 3 },
    KeyLayout { render_colum: 77, render_row: 12, position_x: 11, position_y: 3 },
    KeyLayout { render_colum: 83, render_row: 12, position_x: 12, position_y: 3 },
    KeyLayout { render_colum: 93, render_row: 12, position_x: 15, position_y: 3 },
    KeyLayout { render_colum: 143, render_row: 6, position_x: 22, position_y: 1 },
    KeyLayout { render_colum: 18, render_row: 15, position_x: 2, position_y: 4 },
    KeyLayout { render_colum: 57, render_row: 15, position_x: 7, position_y: 4 },
    KeyLayout { render_colum: 96, render_row: 15, position_x: 15, position_y: 4 },
    KeyLayout { render_colum: 3, render_row: 3, position_x: 0, position_y: 0 },
    KeyLayout { render_colum: 9, render_row: 3, position_x: 1, position_y: 0 },
    KeyLayout { render_colum: 3, render_row: 6, position_x: 0, position_y: 1 },
    KeyLayout { render_colum: 9, render_row: 6, position_x: 1, position_y: 1 },
    KeyLayout { render_colum: 3, render_row: 9, position_x: 0, position_y: 2 },
    KeyLayout { render_colum: 9, render_row: 9, position_x: 1, position_y: 2 },
    KeyLayout { render_colum: 3, render_row: 12, position_x: 0, position_y: 3 },
    KeyLayout { render_colum: 9, render_row: 12, position_x: 1, position_y: 3 },
    KeyLayout { render_colum: 3, render_row: 15, position_x: 0, position_y: 4 },
    KeyLayout { render_colum: 9, render_row: 15, position_x: 1, position_y: 4 },
    KeyLayout { render_colum: 131, render_row: 3, position_x: 20, position_y: 0 },
    KeyLayout { render_colum: 137, render_row: 3, position_x: 21, position_y: 0 },
    KeyLayout { render_colum: 125, render_row: 6, position_x: 19, position_y: 1 },
    KeyLayout { render_colum: 131, render_row: 6, position_x: 20, position_y: 1 },
    KeyLayout { render_colum: 137, render_row: 6, position_x: 21, position_y: 1 },
    KeyLayout { render_colum: 143, render_row: 9, position_x: 22, position_y: 2 },
    KeyLayout { render_colum: 125, render_row: 9, position_x: 19, position_y: 2 },
    KeyLayout { render_colum: 131, render_row: 9, position_x: 20, position_y: 2 },
    KeyLayout { render_colum: 137, render_row: 9, position_x: 21, position_y: 2 },
    KeyLayout { render_colum: 143, render_row: 12, position_x: 22, position_y: 3 },
    KeyLayout { render_colum: 125, render_row: 12, position_x: 19, position_y: 3 },
    KeyLayout { render_colum: 131, render_row: 12, position_x: 20, position_y: 3 },
    KeyLayout { render_colum: 137, render_row: 12, position_x: 21, position_y: 3 },
    KeyLayout { render_colum: 125, render_row: 15, position_x: 19, position_y: 4 },
    KeyLayout { render_colum: 105, render_row: 6, position_x: 16, position_y: 1 },
    KeyLayout { render_colum: 111, render_row: 3, position_x: 17, position_y: 0 },
    KeyLayout { render_colum: 111, render_row: 12, position_x: 17, position_y: 3 },
    KeyLayout { render_colum: 117, render_row: 3, position_x: 18, position_y: 0 },
    KeyLayout { render_colum: 0, render_row: 0, position_x: 0xFF, position_y: 0xFF },
    KeyLayout { render_colum: 105, render_row: 15, position_x: 16, position_y: 4 },
    KeyLayout { render_colum: 0, render_row: 0, position_x: 0xFF, position_y: 0xFF },
    KeyLayout { render_colum: 117, render_row: 15, position_x: 18, position_y: 4 },
    KeyLayout { render_colum: 0, render_row: 0, position_x: 0xFF, position_y: 0xFF },
    KeyLayout { render_colum: 111, render_row: 6, position_x: 17, position_y: 1 },
    KeyLayout { render_colum: 111, render_row: 15, position_x: 17, position_y: 4 },
    KeyLayout { render_colum: 117, render_row: 6, position_x: 18, position_y: 1 },
    KeyLayout { render_colum: 105, render_row: 3, position_x: 16, position_y: 0 }
];

const BACKGROUND: [&'static str; 21] = [
    "┌─────┬─────┐ ┌─────┬─────┬─────┬─────┬─────┬─────┬─────┬─────┬─────┬─────┬─────┬─────┬─────┬───────┐ ┌─────┬─────┬─────┐ ┌─────┬─────┬─────┬─────┐",
    "│ F1  │ F2  │ │  `  │  1  │  2  │  3  │  4  │  5  │  6  │  7  │  8  │  9  │  0  │  -  │  =  │   ⌫   │ │ INS │  ↖  │  ⇞  │ │ ESC │ NUM │ SCR │ SYS │",
    "│     │     │ │     │     │     │     │     │     │     │     │     │     │     │     │     │       │ │     │     │     │ │     │     │     │     │",
    "├─────┼─────┤ ├─────┴─┬───┴─┬───┴─┬───┴─┬───┴─┬───┴─┬───┴─┬───┴─┬───┴─┬───┴─┬───┴─┬───┴─┬───┴─┬─────┤ ├─────┼─────┼─────┤ ├─────┼─────┼─────┼─────┤",
    "│ F3  │ F4  │ │   ⇥   │  Q  │  W  │  E  │  R  │  T  │  Y  │  U  │  I  │  O  │  P  │  [  │  ]  │  \\  │ │ DEL │  ↘  │  ⇟  │ │  7  │  8  │  9  │  *  │",
    "│     │     │ │       │     │     │     │     │     │     │     │     │     │     │     │     │     │ │     │     │     │ │     │     │     │     │",
    "├─────┼─────┤ ├───────┴┬────┴┬────┴┬────┴┬────┴┬────┴┬────┴┬────┴┬────┴┬────┴┬────┴┬────┴┬────┴─────┤ └─────┴─────┴─────┘ ├─────┼─────┼─────┼─────┤",
    "│ F5  │ F6  │ │  CTRL  │  A  │  S  │  D  │  F  │  G  │  H  │  J  │  K  │  L  │  ;  │  '  │    ↵     │                     │  4  │  5  │  6  │  -  │",
    "│     │     │ │        │     │     │     │     │     │     │     │     │     │     │     │          │                     │     │     │     │     │",
    "├─────┼─────┤ ├────────┴──┬──┴──┬──┴──┬──┴──┬──┴──┬──┴──┬──┴──┬──┴──┬──┴──┬──┴──┬──┴──┬──┴──────────┤       ┌─────┐       ├─────┼─────┼─────┼─────┤",
    "│ F7  │ F8  │ │     ⇧     │  Z  │  X  │  C  │  V  │  B  │  N  │  M  │  ,  │  .  │  /  │      ⇧      │       │  ↑  │       │  1  │  2  │  3  │  +  │",
    "│     │     │ │           │     │     │     │     │     │     │     │     │     │     │             │       │     │       │     │     │     │     │",
    "├─────┼─────┤ ├───────┬───┴──┬──┴─────┴─────┴─────┴─────┴─────┴─────┴─────┴─────┴─────┼─────┬───────┤ ┌─────┼─────┼─────┐ ├─────┴─────┼─────┤     │",
    "│ F9  │ F10 │ │  ALT  │      │                         SPACE                          │     │   ⇪   │ │  ←  │  ↓  │  →  │ │  0        │  .  │     │",
    "│     │     │ │       │      │                                                        │     │       │ │     │     │     │ │           │     │     │",
    "└─────┴─────┘ └───────┘      └────────────────────────────────────────────────────────┘     └───────┘ └─────┴─────┴─────┘ └───────────┴─────┴─────┘",
    "",
    "Type here to control the keybinding process and type in the video window to enter a scancode at the selection.",
    "Escape: Leave the keyboard-mapping-tool",
    "Arrow Keys: Navigate / select",
    "Backspace: Unregister selected entry"
];

pub struct KeyboardMapping {
    pub mapping_tool_is_active: bool,
    keycode_translation: [Keycode; 256],
    selected_keycode: Keycode
}

impl KeyboardMapping {
    pub fn new() -> Self {
        Self {
            mapping_tool_is_active: false,
            keycode_translation: unsafe { std::mem::zeroed() },
            selected_keycode: Keycode::Escape
        }
    }

    pub fn load_config(&mut self, config: &crate::config::Config) {
        for (key_name, scancode) in &config.keymap {
            self.keycode_translation[scancode.as_integer().unwrap() as usize] = Keycode::from_str(key_name.as_str()).unwrap();
        }
    }

    pub fn save_config(&mut self, config: &mut crate::config::Config) {
        config.keymap.clear();
        for scancode in 0..self.keycode_translation.len() {
            let keycode = self.keycode_translation[scancode];
            if keycode != Keycode::None {
                config.keymap.insert(keycode.to_string(), toml::Value::Integer(scancode as i64));
            }
        }
    }

    pub fn handle_gui_key(&mut self, cpu: &mut crate::cpu::CPU, bus: &mut crate::bus::BUS, stdout: &mut termion::raw::RawTerminal<std::io::Stdout>, scancode: u8, pressed: bool) {
        if !self.mapping_tool_is_active {
            let mut keycode = self.keycode_translation[scancode as usize] as u8&0x7F;
            if !pressed {
                keycode |= 0x80;
            }
            bus.ps2_controller.push_data(cpu, &mut bus.pic, &mut bus.handler_schedule, keycode);
            return;
        }
        if pressed {
            self.set_scancode_of_keycode(stdout, self.selected_keycode, scancode, false);
            let mut is_next = false;
            for keycode in Keycode::iter() {
                if self.selected_keycode == keycode {
                    is_next = true;
                } else if is_next {
                    self.set_selection(stdout, keycode);
                    break;
                }
            }
        }
    }

    pub fn handle_cli_key(&mut self, stdout: &mut termion::raw::RawTerminal<std::io::Stdout>, key: termion::event::Key) {
        match key {
            termion::event::Key::Esc => { self.deactivate(stdout); },
            termion::event::Key::Backspace => {
                self.set_scancode_of_keycode(stdout, self.selected_keycode, 0, true);
            },
            termion::event::Key::Left => {
                self.navigate(stdout, 0, -1, 0);
            },
            termion::event::Key::Right => {
                self.navigate(stdout, 0, 1, 22);
            },
            termion::event::Key::Up => {
                self.navigate(stdout, 1, -1, 0);
            },
            termion::event::Key::Down => {
                self.navigate(stdout, 1, 1, 4);
            },
            _ => {}
        }
    }

    fn get_layout_of_keycode(keycode: Keycode) -> &'static KeyLayout {
        let mut index = keycode as usize;
        if index&0x80 != 0 {
            index = (index&0x7F)+13;
        }
        &KEY_LAYOUT[index]
    }

    fn get_key_layout_at_position(&mut self, x: u8, y: u8) -> Keycode {
        for keycode in Keycode::iter() {
            let key_layout: &KeyLayout = Self::get_layout_of_keycode(keycode);
            if key_layout.position_x == x && key_layout.position_y == y {
                return keycode;
            }
        }
        return Keycode::None;
    }

    fn set_selection(&mut self, stdout: &mut termion::raw::RawTerminal<std::io::Stdout>, keycode: Keycode) {
        let prev_selected_keycode = self.selected_keycode;
        self.selected_keycode = keycode;
        self.render_key_field(stdout, prev_selected_keycode);
        self.render_key_field(stdout, self.selected_keycode);
        stdout.flush().unwrap();
    }

    fn navigate(&mut self, stdout: &mut termion::raw::RawTerminal<std::io::Stdout>, dimension: u8, direction: i8, limit: u8) {
        let key_layout = Self::get_layout_of_keycode(self.selected_keycode);
        let mut position = if dimension == 0 { key_layout.position_x } else { key_layout.position_y };
        while (direction == -1 && position > 0) || (direction == 1 && position < limit) {
            position = (position as i8+direction) as u8;
            let keycode = if dimension == 0 { self.get_key_layout_at_position(position, key_layout.position_y) } else { self.get_key_layout_at_position(key_layout.position_x, position) };
            if keycode != Keycode::None {
                self.set_selection(stdout, keycode);
                break;
            }
        }
    }

    fn set_scancode_of_keycode(&mut self, stdout: &mut termion::raw::RawTerminal<std::io::Stdout>, keycode: Keycode, new_scancode: u8, reset: bool) {
        loop {
            match self.keycode_translation.iter().position(|value| *value == keycode) {
                Some(scancode) => { self.keycode_translation[scancode as usize] = Keycode::None; },
                None => break
            }
        }
        if !reset {
            let prev_keycode = self.keycode_translation[new_scancode as usize];
            self.keycode_translation[new_scancode as usize] = keycode;
            self.render_key_field(stdout, prev_keycode);
        }
        self.render_key_field(stdout, keycode);
        stdout.flush().unwrap();
    }

    fn render_key_field(&mut self, stdout: &mut termion::raw::RawTerminal<std::io::Stdout>, keycode: Keycode) {
        let key_layout = Self::get_layout_of_keycode(keycode);
        if key_layout.position_x == 0xFF && key_layout.position_y == 0xFF {
            return;
        }
        let scancode_str = match self.keycode_translation.iter().position(|value| *value == keycode) {
            Some(scancode) => format!("{:03}", scancode),
            None => "   ".to_string()
        };
        if self.selected_keycode == keycode {
            write!(stdout, "{}{}{}{}", termion::cursor::Goto(key_layout.render_colum as u16, key_layout.render_row as u16), termion::style::Invert, scancode_str, termion::style::Reset).unwrap();
        } else {
            write!(stdout, "{}{}", termion::cursor::Goto(key_layout.render_colum as u16, key_layout.render_row as u16), scancode_str).unwrap();
        }
    }

    pub fn activate(&mut self, stdout: &mut termion::raw::RawTerminal<std::io::Stdout>) {
        if self.mapping_tool_is_active {
            return;
        }
        self.mapping_tool_is_active = true;
        stdout.activate_raw_mode().unwrap();
        write!(stdout, "{}{}{}", termion::cursor::Hide, termion::cursor::Goto(1, 1), termion::clear::All).unwrap();
        for row in 0..BACKGROUND.len() {
            write!(stdout, "{}{}", termion::cursor::Goto(1, 1+row as u16), BACKGROUND[row]).unwrap();
        }
        for keycode in Keycode::iter() {
            self.render_key_field(stdout, keycode);
        }
        stdout.flush().unwrap();
    }

    pub fn deactivate(&mut self, stdout: &mut termion::raw::RawTerminal<std::io::Stdout>) {
        if !self.mapping_tool_is_active {
            return;
        }
        self.mapping_tool_is_active = false;
        write!(stdout, "{}{}{}", termion::cursor::Goto(1, 1), termion::clear::All, termion::cursor::Show).unwrap();
        stdout.suspend_raw_mode().unwrap();
        stdout.flush().unwrap();
    }
}
