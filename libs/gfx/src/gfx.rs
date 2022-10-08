use models::{Card, Rank, Suit, get_rank, get_suit, get_zinger, suits};

use platform_types::{Command, Kind, PaletteIndex, sprite, unscaled::{self, W, H, Rect}, CHAR_W, CHAR_H, CHAR_WIDTH, CHAR_HEIGHT, FONT_WIDTH};

#[derive(Default)]
pub struct Commands {
    commands: Vec<Command>,
}

impl Commands {
    pub fn slice(&self) -> &[Command] {
        &self.commands
    }

    pub fn clear(&mut self) {
        self.commands.clear();
    }

    pub fn sspr(
        &mut self,
        sprite_xy: sprite::XY,
        rect: unscaled::Rect,
    ) {
        self.commands.push(
            Command {
                kind: Kind::Gfx(sprite_xy),
                rect,
            }
        );
    }

    fn print_char_raw(
        &mut self,
        sprite_xy: sprite::XY,
        colour: PaletteIndex,
        rect: unscaled::Rect,
    ) {
        self.commands.push(
            Command {
                kind: Kind::Font(sprite_xy, colour),
                rect,
            }
        );
    }

    pub fn clear_to(&mut self, colour: PaletteIndex) {
        self.commands.push(
            Command {
                kind: Kind::Colour(colour),
                rect: Rect {
                    x: unscaled::X(0),
                    y: unscaled::Y(0),
                    w: unscaled::W(unscaled::WIDTH),
                    h: unscaled::H(unscaled::HEIGHT),
                },
            }
        );
    }

    pub fn print_char(
        &mut self,
        character: u8,
        x: unscaled::X,
        y: unscaled::Y,
        colour: PaletteIndex
    ) {
        self.print_char_raw(
            get_char_xy(character),
            colour,
            Rect {
                x,
                y,
                w: CHAR_W,
                h: CHAR_H,
            }
        );
    }

    pub fn print_line(
        &mut self,
        bytes: &[u8],
        mut x: unscaled::X,
        y: unscaled::Y,
        colour: PaletteIndex,
    ) {
        for &c in bytes.iter() {
            self.print_char(c, x, y, colour);
            x += CHAR_ADVANCE_W;
        }
    }

    pub fn draw_card(
        &mut self,
        card: Card,
        x: unscaled::X,
        y: unscaled::Y
    ) {
        self.sspr(
            (
                card::BACKING_SPRITE_X,
                card::BACKING_SPRITE_BASE_Y
                + card::HEIGHT
                * sprite::Inner::from(card / models::RANK_COUNT)
            ),
            Rect {
                x,
                y,
                w: card::WIDTH,
                h: card::HEIGHT,
            }
        );

        let image_x = match get_suit(card) {
            Some(suit) => card::IMAGE_BASE_X
                + unscaled::Inner::from(suit)
                * card::IMAGE_W,
            None => card::ZINGER_IMAGE_X,
        };

        let image_y = match get_rank(card) {
            Some(rank) => card::IMAGE_BASE_Y
                + unscaled::Inner::from(rank)
                * card::IMAGE_H,
            None => match get_zinger(card) {
                Some(zinger) => card::IMAGE_BASE_Y
                + unscaled::Inner::from(zinger)
                * card::IMAGE_H,
                None => {
                    debug_assert!(false, "No suit or zinger for card: {card}");
                    card::IMAGE_BASE_Y
                }
            },
        };

        self.sspr(
            (image_x, image_y),
            Rect {
                x: x + W(1),
                y: y + H(4),
                w: card::IMAGE_W,
                h: card::IMAGE_H,
            }
        );
    }
}

pub fn get_char_xy(sprite_number: u8) -> sprite::XY {
    const SPRITES_PER_ROW: u8 = FONT_WIDTH / CHAR_WIDTH;

    (
        sprite::X(Into::into(
            (sprite_number % SPRITES_PER_ROW) * CHAR_WIDTH,
        )),
        sprite::Y(Into::into(
            (sprite_number / SPRITES_PER_ROW) * CHAR_HEIGHT,
        )),
    )
}

pub mod card {
    use super::*;

    use unscaled::{W, H, Inner, w_const_mul};
    use sprite::{x_const_add_w};

    pub const WIDTH: W = W(74);
    pub const HEIGHT: H = H(105);

    pub const IMAGE_W: W = W(72);
    pub const IMAGE_H: H = H(72);

    pub const IMAGE_BASE_X: sprite::X = sprite::X(0);
    pub const IMAGE_BASE_Y: sprite::Y = sprite::Y(0);

    pub const ZINGER_IMAGE_X: sprite::X =
        x_const_add_w(
            IMAGE_BASE_X,
            w_const_mul(IMAGE_W, models::SUIT_COUNT as Inner)
        );

    pub const BACKING_SPRITE_X: sprite::X =
        x_const_add_w(
            ZINGER_IMAGE_X,
            IMAGE_W
        );
    pub const BACKING_SPRITE_BASE_Y: sprite::Y = sprite::Y(0);
}

pub const TEN_CHAR: u8 = 27;

pub const CLUB_CHAR: u8 = 31;
pub const DIAMOND_CHAR: u8 = 29;
pub const HEART_CHAR: u8 = 30;
pub const SPADE_CHAR: u8 = 28;

pub const CHAR_SPACING: u8 = 2;
pub const CHAR_SPACING_W: unscaled::W = unscaled::W(CHAR_SPACING as _);
pub const CHAR_SPACING_H: unscaled::H = unscaled::H(CHAR_SPACING as _);

const CHAR_ADVANCE_WIDTH: unscaled::Inner =
    CHAR_WIDTH as unscaled::Inner
    + CHAR_SPACING as unscaled::Inner;

pub const CHAR_ADVANCE_W: unscaled::W = unscaled::W(CHAR_ADVANCE_WIDTH);

const CHAR_ADVANCE_HEIGHT: unscaled::Inner =
    CHAR_HEIGHT as unscaled::Inner
    + CHAR_SPACING as unscaled::Inner;

pub const CHAR_ADVANCE_H: unscaled::H = unscaled::H(CHAR_ADVANCE_HEIGHT);

// TODO `CharCount` type?
pub const WIDTH_IN_CHARS: unscaled::Inner =
    unscaled::WIDTH
    / CHAR_ADVANCE_WIDTH;
