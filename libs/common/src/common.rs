
pub use features::*;

mod rendering;
pub use self::rendering::*;

use xs::Xs;

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

pub const RANK_COUNT: u8 = 13;
pub const SUIT_COUNT: u8 = 4;
pub const DECK_SIZE: u8 = RANK_COUNT * SUIT_COUNT;

pub type Card = u8;

pub fn gen_card(rng: &mut Xs) -> Card {
    xs::range(rng, 0..DECK_SIZE as _) as Card
}

pub type Suit = u8;

pub mod suits {
    use super::*;

    pub const CLUBS: Suit = 0;
    pub const DIAMONDS: Suit = 1;
    pub const HEARTS: Suit = 2;
    pub const SPADES: Suit = 3;
}

pub fn get_suit(card: Card) -> Suit {
    card / RANK_COUNT
}

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

pub type Rank = u8;

pub fn get_rank(card: Card) -> Rank {
    card % RANK_COUNT
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

