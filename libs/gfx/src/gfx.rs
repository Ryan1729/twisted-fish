use models::{Card, Rank, Suit, get_rank, get_suit, suits};

use platform_types::{Command, Kind, PaletteIndex, FONT_WIDTH, unscaled::{self, Rect}};

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
        sprite_x: u8,
        sprite_y: u8,
        rect: unscaled::Rect,
    ) {
        self.commands.push(
            Command {
                kind: Kind::Gfx((sprite_x, sprite_y)),
                rect,
            }
        );
    }

    fn print_char_raw(
        &mut self,
        sprite_xy: (u8, u8),
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
        let (sprite_x, sprite_y) = get_char_xy(character);
        self.print_char_raw(
            (sprite_x, sprite_y),
            colour,
            Rect {
                x,
                y,
                w: CHAR_W,
                h: CHAR_H,
            }
        );
    }

    pub fn draw_card(
        &mut self,
        card: Card,
        x: unscaled::X,
        y: unscaled::Y
    ) {
        self.sspr(
            card::FRONT_SPRITE_X,
            card::FRONT_SPRITE_Y,
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
            rank_char | FONT_FLIP,
            x + card::RIGHT_RANK_EDGE_W,
            y + card::RIGHT_RANK_EDGE_H,
            colour,
        );
        self.print_char(
            suit_char | FONT_FLIP,
            x + card::RIGHT_SUIT_EDGE_W,
            y + card::RIGHT_SUIT_EDGE_H,
            colour,
        );
    }
}

pub fn get_char_xy(sprite_number: u8) -> (u8, u8) {
    const SPRITES_PER_ROW: u8 = FONT_WIDTH as u8 / CHAR_SIZE;

    (
        (sprite_number % SPRITES_PER_ROW) * CHAR_SIZE,
        (sprite_number / SPRITES_PER_ROW) * CHAR_SIZE,
    )
}

pub mod card {
    use super::*;

    use unscaled::{W, H, w_const_add, w_const_sub, h_const_add, h_const_sub};

    pub const WIDTH: W = W(20);
    pub const HEIGHT: H = H(30);

    pub const FRONT_SPRITE_X: u8 = 2;
    pub const FRONT_SPRITE_Y: u8 = 1;

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

pub const CHAR_SIZE: u8 = 8;
pub const CHAR_W: unscaled::W = unscaled::W(CHAR_SIZE);
pub const CHAR_H: unscaled::H = unscaled::H(CHAR_SIZE);

pub const FONT_FLIP: u8 = 128;

