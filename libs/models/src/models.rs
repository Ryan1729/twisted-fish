use xs::Xs;

macro_rules! compile_time_assert {
    ($assertion: expr) => (
        #[allow(unknown_lints, eq_op)]
        // Based on the const_assert macro from static_assertions;
        const _: [(); 0 - !{$assertion} as usize] = [];
    )
}

pub const RANK_COUNT: u8 = 13;
pub const SUIT_COUNT: u8 = 5;
pub const FISH_COUNT: u8 = RANK_COUNT * SUIT_COUNT;
pub const ZINGER_COUNT: u8 = 8;
pub const DECK_SIZE: u8 = FISH_COUNT + ZINGER_COUNT;

pub type CardInner = u8;

pub type Card = CardInner;

pub fn gen_card(rng: &mut Xs) -> Card {
    xs::range(rng, 0..DECK_SIZE as _) as Card
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
struct CardOption(CardInner);

compile_time_assert!{
    CardInner::MAX > DECK_SIZE
}

impl CardOption {
    pub const fn option(self) -> Option<Card> {
        self.0.checked_sub(1)
    }

    pub const fn some(card: Card) -> Self {
        Self(card + 1)
    }

    pub const NONE: Self = Self(0);

    pub fn is_none(self) -> bool {
        self == Self::NONE
    }

    pub fn is_some(self) -> bool {
        self != Self::NONE
    }
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
    if (FISH_COUNT..DECK_SIZE).contains(&card) {
        Some(card - FISH_COUNT)
    } else {
        None
    }
}

pub type HandLen = u8;

/// An ordered collection of cards that can hold at leat one copy of each card.
#[derive(Clone)]
pub struct Hand([CardOption; DECK_SIZE as usize]);

impl Default for Hand {
    fn default() -> Hand {
        Hand([CardOption::NONE; DECK_SIZE as usize])
    }
}

impl Hand {
    pub fn fresh_deck(rng: &mut Xs) -> Self {
        let mut output = [CardOption::NONE; DECK_SIZE as usize];

        for i in 0..DECK_SIZE {
            output[i as usize] = CardOption::some(i);
        }

        xs::shuffle(rng, &mut output);

        Self(output)
    }

    pub fn len(&self) -> HandLen {
        for i in 0..DECK_SIZE {
            if self.0[i as usize].is_none() {
                return i;
            }
        }

        DECK_SIZE
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn push(&mut self, card: Card) {
        let mut pushed = false;

        for i in 0..(DECK_SIZE as usize) {
            if self.0[i].is_none() {
                self.0[i] = CardOption::some(card);
                pushed = true;
                break
            }
        }

        debug_assert!(pushed);
    }

    pub fn draw(&mut self) -> Option<Card> {
        for i in (0..DECK_SIZE as usize).rev() {
            if self.0[i].is_some() {
                let output = self.0[i].option();

                self.0[i] = CardOption::NONE;

                return output;
            }
        }

        None
    }

    pub fn iter(&self) -> impl Iterator<Item = Card> + '_ {
        self.0.iter().filter_map(|co| co.option())
    }
}

pub type CardIndex = u8;

impl Hand {
    pub fn enumerated_iter(&self) -> impl Iterator<Item = (CardIndex, Card)> + '_ {
        compile_time_assert!{DECK_SIZE as u64 <= CardIndex::MAX as u64}

        self.iter()
            .enumerate()
            // We rely on the len never being above `DECK_SIZE`.
            .map(|(i, e)| (CardIndex::try_from(i).unwrap(), e))
    }
}