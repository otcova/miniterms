use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};

#[derive(Copy, Clone)]
pub enum Key {
    Up = 0,
    Down,
    Left,
    Right,
    Space,
}

#[derive(Copy, Clone)]
pub struct Keys {
    just_pressed: u8,
    pressing: u8,
}

impl Keys {
    pub fn new() -> Self {
        Self {
            just_pressed: 0,
            pressing: 0,
        }
    }

    pub fn update(&mut self) {
        self.just_pressed = 0;
    }

    pub fn press(&mut self, key: Key) {
        self.just_pressed |= key.mask();
        self.pressing |= key.mask();
    }

    pub fn release(&mut self, key: Key) {
        self.pressing &= !key.mask();
    }

    pub fn handle_key_event(&mut self, key_event: KeyEvent) {
        let Some(key) = Key::from_code(key_event.code) else {
            return;
        };

        match key_event.kind {
            KeyEventKind::Press => self.press(key),
            KeyEventKind::Release => self.release(key),
            KeyEventKind::Repeat => {}
        }
    }

    pub fn just_pressed(&self, key: Key) -> bool {
        (self.just_pressed & key.mask()) != 0
    }

    pub fn pressing(&self, key: Key) -> bool {
        let pressing = (self.pressing & key.mask()) != 0;
        self.just_pressed(key) || pressing
    }

    pub fn any_pressed(&self) -> bool {
        self.just_pressed != 0 || self.pressing != 0
    }
}

impl Key {
    fn mask(self) -> u8 {
        1 << self as u8
    }

    pub fn from_code(code: KeyCode) -> Option<Key> {
        use KeyCode::*;
        match code {
            Down | Char('j') | Char('J') | Char('s') | Char('S') => Some(Key::Down),
            Up | Char('k') | Char('K') | Char('w') | Char('W') => Some(Key::Up),
            Right | Char('l') | Char('L') | Char('d') | Char('D') => Some(Key::Right),
            Left | Char('h') | Char('H') | Char('a') | Char('A') => Some(Key::Left),
            Enter | Char(' ') => Some(Key::Space),
            _ => None,
        }
    }

    pub fn from_u8(n: u8) -> Key {
        match n {
            0 => Key::Up,
            1 => Key::Down,
            2 => Key::Left,
            3 => Key::Right,
            4 => Key::Space,
            _ => panic!("Invalid Key u8"),
        }
    }
}
