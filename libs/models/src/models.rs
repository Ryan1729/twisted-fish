use xs::Xs;

pub const RANK_COUNT: u8 = 13;
pub const SUIT_COUNT: u8 = 5;
pub const FISH_COUNT: u8 = RANK_COUNT * SUIT_COUNT;
pub const ZINGER_COUNT: u8 = 8;
pub const DECK_SIZE: u8 = FISH_COUNT + ZINGER_COUNT;

pub type Card = u8;

pub fn gen_card(rng: &mut Xs) -> Card {
    xs::range(rng, 0..DECK_SIZE as _) as Card
}

pub type Rank = u8;

pub mod ranks {
    use super::*;

    // Worth 5 points {
    pub const BARNACLE: Suit = 0;
    pub const CRAB: Suit = 1;
    pub const DOGFISH: Suit = 2;
    pub const EEL: Suit = 3;
    pub const FLYING_FISH: Suit = 4;
    pub const HAMMERHEAD: Suit = 5;
    pub const JELLYFISH: Suit = 6;
    pub const SHRIMP: Suit = 7;
    // }
    // Worth 10 points {
    pub const BLOWFISH: Suit = 8;
    pub const CLOWNFISH: Suit = 9;
    pub const STARFISH: Suit = 10;
    pub const WHALE: Suit = 11;
    // }
    // Worth 15 points {
    pub const CARD_SHARK: Suit = 12;
    // }
}

pub fn get_rank(card: Card) -> Option<Rank> {
    if card >= FISH_COUNT {
        None
    } else {
        Some(card % RANK_COUNT)
    }
}

pub type Suit = u8;

pub mod suits {
    use super::*;

    pub const RED: Suit = 0;
    pub const GREEN: Suit = 1;
    pub const BLUE: Suit = 2;
    pub const YELLOW: Suit = 3;
    pub const PURPLE: Suit = 4;
}

pub fn get_suit(card: Card) -> Option<Suit> {
    if card >= FISH_COUNT {
        None
    } else {
        Some((card / RANK_COUNT) % SUIT_COUNT)
    }
}

pub type Zinger = u8;

pub mod zingers {
    use super::*;

    pub const DEAD_SCUBA_DIVER: Zinger = 0;
    pub const DIVINE_INTERVENTION: Zinger = 1;
    pub const GLASS_BOTTOM_BOAT: Zinger = 2;
    pub const NO_FISHING: Zinger = 3;
    pub const THE_GAME_WARDEN: Zinger = 4;
    pub const THE_LURE: Zinger = 5;
    pub const THE_NET: Zinger = 6;
    pub const TWO_FISTED_FISHERMAN: Zinger = 7;
}

pub fn get_zinger(card: Card) -> Option<Zinger> {
    if card >= FISH_COUNT && card < DECK_SIZE {
        Some((card / RANK_COUNT) % SUIT_COUNT)
    } else {
        None
    }
}