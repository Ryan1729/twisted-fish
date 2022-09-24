use models::{Card, Rank, Suit, get_rank, get_suit, suits};

use platform_types::{Command, Kind, PaletteIndex, sprite, unscaled::{self, Rect}, CHAR_W, CHAR_H, CHAR_WIDTH, CHAR_HEIGHT, FONT_WIDTH};

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
            card::FRONT_SPRITE_XY,
            Rect {
                x,
                y,
                w: card::WIDTH,
                h: card::HEIGHT,
            }
        );

        let (colour, suit_char) = get_suit_colour_and_char(get_suit(card));

        let rank_char = get_rank_char(card);

        self.print_char(
            rank_char,
            x + card::LEFT_RANK_EDGE_W,
            y + card::LEFT_RANK_EDGE_H,
            colour,
        );
        self.print_char(
            suit_char,
            x + card::LEFT_SUIT_EDGE_W,
            y + card::LEFT_SUIT_EDGE_H,
            colour,
        );

        self.print_char(
            rank_char,
            x + card::RIGHT_RANK_EDGE_W,
            y + card::RIGHT_RANK_EDGE_H,
            colour,
        );
        self.print_char(
            suit_char,
            x + card::RIGHT_SUIT_EDGE_W,
            y + card::RIGHT_SUIT_EDGE_H,
            colour,
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

    use unscaled::{W, H, w_const_add, w_const_sub, h_const_add, h_const_sub};

    pub const WIDTH: W = W(74);
    pub const HEIGHT: H = H(105);

    type ImageW = u16;
    type ImageH = u16;

    const IMAGE_W: ImageW = 72;
    #[allow(unused)]
    const IMAGE_H: ImageH = 72;

    pub const FRONT_SPRITE_X: sprite::X = sprite::X(IMAGE_W * 6);
    pub const FRONT_SPRITE_Y: sprite::Y = sprite::Y(0);
    pub const FRONT_SPRITE_XY: sprite::XY = (FRONT_SPRITE_X, FRONT_SPRITE_Y);

    pub const LEFT_RANK_EDGE_W: W = W(3);
    pub const LEFT_RANK_EDGE_H: H = H(3);

    pub const LEFT_SUIT_EDGE_W: W = W(1);
    pub const LEFT_SUIT_EDGE_H: H = H(10);

    pub const RIGHT_RANK_EDGE_W: W = w_const_sub(
        WIDTH,
        w_const_add(LEFT_RANK_EDGE_W, CHAR_W)
    );
    pub const RIGHT_RANK_EDGE_H: H = h_const_sub(
        HEIGHT,
        h_const_add(LEFT_RANK_EDGE_H, CHAR_H)
    );

    pub const RIGHT_SUIT_EDGE_W: W = w_const_sub(
        WIDTH,
        w_const_add(LEFT_SUIT_EDGE_W, CHAR_W)
    );
    pub const RIGHT_SUIT_EDGE_H: H = h_const_sub(
        HEIGHT,
        h_const_add(LEFT_SUIT_EDGE_H, CHAR_H)
    );
}

pub const TEN_CHAR: u8 = 27;

pub const CLUB_CHAR: u8 = 31;
pub const DIAMOND_CHAR: u8 = 29;
pub const HEART_CHAR: u8 = 30;
pub const SPADE_CHAR: u8 = 28;

pub fn get_suit_colour_and_char(suit: Suit) -> (u8, u8) {
    const RED_INDEX: u8 = 2;
    const PURPLE_INDEX: u8 = 4;
    const BLACK_INDEX: u8 = 7;

    match suit {
        suits::CLUBS => (BLACK_INDEX, CLUB_CHAR),
        suits::DIAMONDS => (RED_INDEX, DIAMOND_CHAR),
        suits::HEARTS => (RED_INDEX, HEART_CHAR),
        suits::SPADES => (BLACK_INDEX, SPADE_CHAR),
        _ => (PURPLE_INDEX, 33), //purple "!"
    }
}

pub fn get_rank_char(card: Card) -> u8 {
    get_rank_char_from_rank(get_rank(card))
}

pub fn get_rank_char_from_rank(rank: Rank) -> u8 {
    match rank {
        0 => b'a',
        1 => b'2',
        2 => b'3',
        3 => b'4',
        4 => b'5',
        5 => b'6',
        6 => b'7',
        7 => b'8',
        8 => b'9',
        9 => TEN_CHAR,
        10 => b'j',
        11 => b'q',
        12 => b'k',
        _ => b'!',
    }
}

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
