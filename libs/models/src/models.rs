use xs::Xs;

macro_rules! compile_time_assert {
    ($assertion: expr) => (
        #[allow(unknown_lints, clippy::eq_op)]
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

pub fn fish_card(rank: Rank, suit: Suit) -> Card {
    suit as CardInner * RANK_COUNT + rank
}

#[test]
fn get_rank_and_get_suit_then_fish_card_works() {
    for card in 0..FISH_COUNT {
        let rank = get_rank(card).unwrap();
        let suit = get_suit(card).unwrap();

        assert_eq!(
            fish_card(
                rank,
                suit,
            ),
            card,
            "expected {rank:?}, {suit:?} -> {card:?}"
        );
    }
}

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
    pub const BARNACLE: Rank = 0;
    pub const CRAB: Rank = 1;
    pub const DOGFISH: Rank = 2;
    pub const EEL: Rank = 3;
    pub const FLYING_FISH: Rank = 4;
    pub const HAMMERHEAD: Rank = 5;
    pub const JELLYFISH: Rank = 6;
    pub const SHRIMP: Rank = 7;
    // }
    // Worth 10 points {
    pub const BLOWFISH: Rank = 8;
    pub const CLOWNFISH: Rank = 9;
    pub const STARFISH: Rank = 10;
    pub const WHALE: Rank = 11;
    // }
    // Worth 15 points {
    pub const CARD_SHARK: Rank = 12;
    // }

    pub const COUNT: u8 = 13;

    pub const TEXT: [&[u8]; COUNT as usize] = [
        b"Barnacle",
        b"Crab",
        b"Dogfish",
        b"Eel",
        b"Flying Fish",
        b"Hammerhead",
        b"Jellyfish",
        b"Shrimp",
        b"Blowfish",
        b"Clownfish",
        b"Starfish",
        b"Whale",
        b"Card Shark",
    ];
}

pub fn get_rank(card: Card) -> Option<Rank> {
    if card >= FISH_COUNT {
        None
    } else {
        Some(card % RANK_COUNT)
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Suit {
    #[default]
    Red,
    Green,
    Blue,
    Yellow,
    Purple,
}

impl Suit {
    pub const COUNT: u8 = 5;

    pub const ALL: [Suit; Suit::COUNT as usize] = [
        Suit::Red,
        Suit::Green,
        Suit::Blue,
        Suit::Yellow,
        Suit::Purple,
    ];

    pub const TEXT: [&[u8]; Suit::COUNT as usize] = [
        b"Red",
        b"Green",
        b"Blue",
        b"Yellow",
        b"Purple",
    ];
}

pub type SuitNumber = u8;

pub mod suits {
    use super::*;

    pub const RED: SuitNumber = 0;
    pub const GREEN: SuitNumber = 1;
    pub const BLUE: SuitNumber = 2;
    pub const YELLOW: SuitNumber = 3;
    pub const PURPLE: SuitNumber = 4;
}

pub fn get_suit(card: Card) -> Option<Suit> {
    if card >= FISH_COUNT {
        None
    } else {
        match (card / RANK_COUNT) % SUIT_COUNT {
            suits::RED => Some(Suit::Red),
            suits::GREEN => Some(Suit::Green),
            suits::BLUE => Some(Suit::Blue),
            suits::YELLOW => Some(Suit::Yellow),
            suits::PURPLE => Some(Suit::Purple),
            _ => None,
        }
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

    pub fn get(&self, i: CardIndex) -> Option<Card> {
        self.0.get(usize::from(i)).and_then(|co| co.option())
    }

    pub fn remove(&mut self, index: CardIndex) -> Option<Card> {
        let output = self.get(index);
        let slice = self.0.as_mut_slice(); 
        let i = usize::from(index);
        slice.copy_within((i + 1).., i);
        slice[slice.len() - 1] = CardOption::NONE;
        output
    }
}
