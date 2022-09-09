use models::{Card, Rank, Suit, get_rank, get_suit, suits};

use platform_types::{Command, Kind, PaletteIndex, Rect, FONT_WIDTH, screen};

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
        w: u8,
        h: u8,
        display_x: u8,
        display_y: u8,
    ) {
        self.commands.push(
            Command {
                kind: Kind::Gfx((sprite_x, sprite_y)),
                rect: Rect {
                    x: display_x,
                    y: display_y,
                    w,
                    h,
                },
            }
        );
    }

    fn print_char_raw(
        &mut self,
        sprite_xy: (u8, u8),
        (w, h): (u8, u8),
        (display_x, display_y): (u8, u8),
        colour: PaletteIndex,
    ) {
        self.commands.push(
            Command {
                kind: Kind::Font(sprite_xy, colour),
                rect: Rect {
                    x: display_x,
                    y: display_y,
                    w,
                    h,
                },
            }
        );
    }

    pub fn clear_to(&mut self, colour: PaletteIndex) {
        self.commands.push(
            Command {
                kind: Kind::Colour(colour),
                rect: Rect {
                    x: 0,
                    y: 0,
                    w: screen::WIDTH,
                    h: screen::HEIGHT,
                },
            }
        );
    }

    pub fn print_char(&mut self, character: u8, x: u8, y: u8, colour: u8) {
        let (sprite_x, sprite_y) = get_char_xy(character);
        self.print_char_raw(
            (sprite_x, sprite_y),
            (FONT_SIZE, FONT_SIZE),
            (x, y),
            colour
        );
    }

    pub fn draw_card(&mut self, card: Card, x: u8, y: u8) {
        self.sspr(
            card::FRONT_SPRITE_X,
            card::FRONT_SPRITE_Y,
            card::WIDTH,
            card::HEIGHT,
            x,
            y,
        );

        let (colour, suit_char) = get_suit_colour_and_char(get_suit(card));

        let rank_char = get_rank_char(card);

        self.print_char(
            rank_char,
            x + card::LEFT_RANK_X,
            y + card::LEFT_RANK_Y,
            colour,
        );
        self.print_char(
            suit_char,
            x + card::LEFT_SUIT_X,
            y + card::LEFT_SUIT_Y,
            colour,
        );

        self.print_char(
            rank_char | FONT_FLIP,
            x + card::RIGHT_RANK_X,
            y + card::RIGHT_RANK_Y,
            colour,
        );
        self.print_char(
            suit_char | FONT_FLIP,
            x + card::RIGHT_SUIT_X,
            y + card::RIGHT_SUIT_Y,
            colour,
        );
    }
}

pub fn get_char_xy(sprite_number: u8) -> (u8, u8) {
    const SPRITES_PER_ROW: u8 = FONT_WIDTH as u8 / FONT_SIZE;

    (
        (sprite_number % SPRITES_PER_ROW) * FONT_SIZE,
        (sprite_number / SPRITES_PER_ROW) * FONT_SIZE,
    )
}

pub mod card {
    use super::*;

    pub const WIDTH: u8 = 20;
    pub const HEIGHT: u8 = 30;

    pub const FRONT_SPRITE_X: u8 = 2;
    pub const FRONT_SPRITE_Y: u8 = 1;

    pub const LEFT_RANK_X: u8 = 3;
    pub const LEFT_RANK_Y: u8 = 3;

    pub const LEFT_SUIT_X: u8 = 1;
    pub const LEFT_SUIT_Y: u8 = 10;

    pub const RIGHT_RANK_X: u8 = WIDTH - (LEFT_RANK_X + FONT_SIZE);
    pub const RIGHT_RANK_Y: u8 = HEIGHT - (LEFT_RANK_Y + FONT_SIZE);

    pub const RIGHT_SUIT_X: u8 = WIDTH - (LEFT_SUIT_X + FONT_SIZE);
    pub const RIGHT_SUIT_Y: u8 = HEIGHT - (LEFT_SUIT_Y + FONT_SIZE);
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

pub const FONT_SIZE: u8 = 8;
pub const FONT_FLIP: u8 = 128;

