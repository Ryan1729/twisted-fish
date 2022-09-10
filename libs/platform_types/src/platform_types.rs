pub mod unscaled {
    ///! Values are in pixels.

    use xs::Xs;

    type Inner = u16;

    pub const WIDTH: Inner = 128;
    pub const HEIGHT: Inner = 128;

    macro_rules! def {
        ($($name: ident, $inner_name: ident)+) => {$(
            pub type $inner_name = Inner;
            #[derive(Copy, Clone, Default, Debug)]
            pub struct $name(pub $inner_name);

            impl $name {
                pub const fn get(self) -> $inner_name {
                    self.0
                }
            }

            impl From<$name> for $inner_name {
                fn from(to_convert: $name) -> $inner_name {
                    $inner_name::from(to_convert.0)
                }
            }
        )*}
    }

    def!{
        X, XInner
        Y, YInner
        W, WInner
        H, HInner
    }

    impl X {
        pub fn gen(rng: &mut Xs) -> X {
            X(xs::range(rng, 0..WIDTH as _) as XInner)
        }
    }

    impl Y {
        pub fn gen(rng: &mut Xs) -> Y {
            Y(xs::range(rng, 0..HEIGHT as _) as YInner)
        }
    }

    pub const fn w_const_add(a: W, b: W) -> W {
        W(a.0 + b.0)
    }

    pub const fn w_const_sub(a: W, b: W) -> W {
        W(a.0 - b.0)
    }

    pub const fn h_const_add(a: H, b: H) -> H {
        H(a.0 + b.0)
    }

    pub const fn h_const_sub(a: H, b: H) -> H {
        H(a.0 - b.0)
    }

    macro_rules! self_add_sub_def {
        ($($name: ident)+) => {$(
            impl core::ops::AddAssign for $name {
                fn add_assign(&mut self, other: Self) {
                    self.0 += other.0;
                }
            }
        
            impl core::ops::Add for $name {
                type Output = Self;
        
                fn add(mut self, other: Self) -> Self::Output {
                    self += other;
                    self
                }
            }
        
            impl core::ops::SubAssign for $name {
                fn sub_assign(&mut self, other: Self) {
                    self.0 -= other.0;
                }
            }
        
            impl core::ops::Sub for $name {
                type Output = Self;
        
                fn sub(mut self, other: Self) -> Self::Output {
                    self -= other;
                    self
                }
            }
        )*}
    }

    self_add_sub_def!{W H}

    impl core::ops::AddAssign<W> for X {
        fn add_assign(&mut self, other: W) {
            self.0 += other.0;
        }
    }

    impl core::ops::Add<W> for X {
        type Output = Self;

        fn add(mut self, other: W) -> Self::Output {
            self += other;
            self
        }
    }

    impl core::ops::AddAssign<H> for Y {
        fn add_assign(&mut self, other: H) {
            self.0 += other.0;
        }
    }

    impl core::ops::Add<H> for Y {
        type Output = Self;

        fn add(mut self, other: H) -> Self::Output {
            self += other;
            self
        }
    }

    #[derive(Clone, Copy, Debug)]
    pub struct Rect {
        pub x: X,
        pub y: Y,
        pub w: W,
        pub h: H,
    }
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
pub struct Command {
    pub rect: unscaled::Rect,
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
