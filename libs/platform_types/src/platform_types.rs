///! Values are in pixels.
pub mod screen {
    pub const WIDTH: u8 = 128;
    pub const HEIGHT: u8 = 128;
}

pub const GFX_WIDTH: usize = 128;
pub const GFX_HEIGHT: usize = 128;
pub const GFX_LENGTH: usize = GFX_WIDTH * GFX_HEIGHT;

pub const FONT_WIDTH: usize = 128;
pub const FONT_HEIGHT: usize = 128;
pub const FONT_LENGTH: usize = FONT_WIDTH * FONT_HEIGHT;

pub type PaletteIndex = u8;

pub mod command {
    pub type X = u8;
    pub type Y = u8;
    pub type XY = (X, Y);
}

#[derive(Clone, Copy, Debug)]
pub enum Kind {
    Gfx(command::XY),
    Font(command::XY, PaletteIndex),
    Colour(PaletteIndex),
}

#[derive(Clone, Copy, Debug)]
pub struct Rect {
    pub x: u8,
    pub y: u8,
    pub w: u8,
    pub h: u8,
}

#[derive(Clone, Copy, Debug)]
pub struct Command {
    pub rect: Rect,
    pub kind: Kind,
}

#[derive(Clone, Copy, Default, Debug)]
pub struct Input {
    pub gamepad: Button,
    pub previous_gamepad: Button,
}

impl Input {
    #[allow(unused)]
    pub fn pressed_this_frame(&self, buttons: Button) -> bool {
        !self.previous_gamepad.contains(buttons) && self.gamepad.contains(buttons)
    }

    #[allow(unused)]
    pub fn released_this_frame(&self, buttons: Button) -> bool {
        self.previous_gamepad.contains(buttons) && !self.gamepad.contains(buttons)
    }
}

#[derive(Clone, Copy, Debug)]
pub enum SFX {
    CardPlace,
    CardSlide,
    ButtonPress,
}

pub struct Speaker {
    requests: Vec<SFX>,
}

impl Default for Speaker {
    fn default() -> Self {
        Speaker {
            requests: Vec::with_capacity(8),
        }
    }
}

impl Speaker {
    pub fn clear(&mut self) {
        self.requests.clear();
    }

    pub fn request_sfx(&mut self, sfx: SFX) {
        self.requests.push(sfx);
    }

    pub fn slice(&self) -> &[SFX] {
        &self.requests
    }
}

// These values are deliberately picked to be the same as the ones in NES' input registers.
pub mod button {
    #[derive(Copy, Clone, Default, Debug, PartialEq, Eq)]
    pub struct Button(u8);

    impl Button {
        pub const A     : Self = Self(1 << 0);
        pub const B     : Self = Self(1 << 1);
        pub const SELECT: Self = Self(1 << 2);
        pub const START : Self = Self(1 << 3);
        pub const UP    : Self = Self(1 << 4);
        pub const DOWN  : Self = Self(1 << 5);
        pub const LEFT  : Self = Self(1 << 6);
        pub const RIGHT : Self = Self(1 << 7);

        pub const fn contains(&self, other: Self) -> bool {
            self.0 & other.0 == other.0
        }

        pub fn insert(&mut self, other: Self) {
            self.0 |= other.0;
        }

        pub fn remove(&mut self, other: Self) {
            self.0 &= !other.0;
        }
    }
}
pub use button::Button;

pub type Logger = Option<fn(&str) -> ()>;

pub type StateParams = ([u8; 16], Logger, Logger);

pub trait State {
    fn frame(&mut self) -> (&[Command], &[SFX]);

    fn press(&mut self, button: Button);

    fn release(&mut self, button: Button);
}
