use xs::Xs;

macro_rules! compile_time_assert {
    ($assertion: expr) => (
        #[allow(unknown_lints, clippy::eq_op)]
        // Based on the const_assert macro from static_assertions;
        const _: [(); 0 - !{$assertion} as usize] = [];
    )
}

pub const RANK_COUNT: u8 = 5;
pub const SUIT_COUNT: u8 = 5;
pub const DECK_SIZE: u8 = RANK_COUNT * SUIT_COUNT;

pub type CardInner = u8;

pub type Card = CardInner;

pub fn fish_card(rank: Rank, suit: Suit) -> Card {
    suit as CardInner * RANK_COUNT + (rank as CardInner)
}

pub fn gen_card(rng: &mut Xs) -> Card {
    xs::range(rng, 0..DECK_SIZE as _) as Card
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
struct CardOption(CardInner);

compile_time_assert!{
    CardInner::MAX > DECK_SIZE
}

impl core::fmt::Debug for CardOption {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f, 
            "{:?} ({:?})",
            self.option(),
            self.option()
                .and_then(|card| 
                    get_rank(card)
                        .map(|r| (r, get_suit(card).unwrap()))
                )
        )
    }
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

// TODO? Tighter representation that still allows representing Dead Scuba Diver
// but doesn't allow non-matched cards?
pub type Basket = [Card; Suit::COUNT as usize];

pub type AlmostCompleteBasket = [CardIndex; (Suit::COUNT - 1) as _];

pub type AlmostCompleteBaskets = [Option<AlmostCompleteBasket>; Rank::COUNT as _];

#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Rank {
    #[default]
    Crab, // Red
    Eel, // Green
    Barnacle, // Blue
    Jellyfish, // Yellow
    Shrimp, // Purple
}

impl From<Rank> for u8 {
    fn from(rank: Rank) -> u8 {
        rank as u8
    }
}

impl Rank {
    pub const COUNT: u8 = 5;

    pub const ALL: [Rank; Rank::COUNT as usize] = [
        Rank::Crab,
        Rank::Eel,
        Rank::Barnacle,
        Rank::Jellyfish,
        Rank::Shrimp,
    ];

    pub const TEXT: [&[u8]; Rank::COUNT as usize] = [
        b"Crab",
        b"Eel",
        b"Barnacle",
        b"Jellyfish",
        b"Shrimp",
    ];

    pub fn wrapping_dec(mut self, acb: AlmostCompleteBaskets) -> Self {
        for _ in 0..acb.len() {
            self = match self {
                Rank::Crab => Rank::Shrimp,
                Rank::Eel => Rank::Crab,
                Rank::Barnacle => Rank::Eel,
                Rank::Jellyfish => Rank::Barnacle,
                Rank::Shrimp => Rank::Jellyfish,
            };

            if acb[(self as u8) as usize].is_some() {
                break
            }
        }

        self
    }

    pub fn wrapping_inc(mut self, acb: AlmostCompleteBaskets) -> Self {
        for _ in 0..acb.len() {
            self = match self {
                Rank::Crab => Rank::Eel,
                Rank::Eel => Rank::Barnacle,
                Rank::Barnacle => Rank::Jellyfish,
                Rank::Jellyfish => Rank::Shrimp,
                Rank::Shrimp => Rank::Crab,
            };

            if acb[(self as u8) as usize].is_some() {
                break
            }
        }

        self
    }

}

pub fn get_rank(card: Card) -> Option<Rank> {
    if card >= DECK_SIZE {
        None
    } else {
        use Rank::*;
        match card % RANK_COUNT {
            0 => Some(Crab),
            1 => Some(Eel),
            2 => Some(Barnacle),
            3 => Some(Jellyfish),
            4 => Some(Shrimp),
            _ => None,
        }
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

impl From<Suit> for u8 {
    fn from(suit: Suit) -> u8 {
        suit as u8
    }
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

    pub fn from_rng(xs: &mut Xs) -> Suit {
        match xs::range(xs, 0..Suit::COUNT as u32) {
            1 => Self::ALL[1],
            2 => Self::ALL[2],
            3 => Self::ALL[3],
            4 => Self::ALL[4],
            _ => Self::ALL[0],
        }
    }

    pub fn wrapping_dec(self) -> Suit {
        match self {
            Suit::Red => Suit::Purple,
            Suit::Green => Suit::Red,
            Suit::Blue => Suit::Green,
            Suit::Yellow => Suit::Blue,
            Suit::Purple => Suit::Yellow,
        }
    }

    pub fn wrapping_inc(self) -> Suit {
        match self {
            Suit::Red => Suit::Green,
            Suit::Green => Suit::Blue,
            Suit::Blue => Suit::Yellow,
            Suit::Yellow => Suit::Purple,
            Suit::Purple => Suit::Red,
        }
    }
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
    if card >= DECK_SIZE {
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

pub type HandIndex = u8;
pub type HandLen = u8;

/// An ordered collection of cards that can hold at leat one copy of each card.
#[derive(Clone, Debug)]
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

    pub fn last(&self) -> Option<Card> {
        self.len()
            .checked_sub(1)
            .and_then(|i| {
                self.0
                    .get(i as usize)
                    .and_then(|o: &CardOption| o.option())
            })
    }

    pub fn last_index(&self) -> HandIndex {
        self.len().saturating_sub(1)
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

    pub fn shuffle(&mut self, rng: &mut Xs) {
        if let Some(i) = self.len().checked_sub(1) {
            xs::shuffle(rng, &mut self.0[0..i as _]);
        }
    }

    pub fn contains(&self, needle: Card) -> bool {
        for card in self.iter() {
            if card == needle {
                return true
            }
        }
        false
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

#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum HandId {
    #[default]
    Player,
    Cpu1,
    Cpu2,
    Cpu3,
}

impl From<CpuId> for HandId {
    fn from(cpu_id: CpuId) -> Self {
        match cpu_id {
            CpuId::One => Self::Cpu1,
            CpuId::Two => Self::Cpu2,
            CpuId::Three => Self::Cpu3,
        }
    }
}

impl From<&CpuId> for HandId {
    fn from(cpu_id: &CpuId) -> Self {
        (*cpu_id).into()
    }
}

macro_rules! hand_id_text {
    ("0") => { "Player" };
    ("1") => { "Cpu 1" };
    ("2") => { "Cpu 2" };
    ("3") => { "Cpu 3" };
    (b"0") => { b"Player" };
    (b"1") => { b"Cpu 1" };
    (b"2") => { b"Cpu 2" };
    (b"3") => { b"Cpu 3" };
}

// TODO? macro for this, I guess?
impl HandId {
    pub const COUNT: u8 = 4;

    pub const ALL: [HandId; Self::COUNT as usize] = [
        HandId::Player,
        HandId::Cpu1,
        HandId::Cpu2,
        HandId::Cpu3,
    ];

    pub const CPUS: [HandId; (Self::COUNT - 1) as usize] = [
        HandId::Cpu1,
        HandId::Cpu2,
        HandId::Cpu3,
    ];

    pub const TEXT: [&[u8]; Self::COUNT as usize] = [
        hand_id_text!(b"0"),
        hand_id_text!(b"1"),
        hand_id_text!(b"2"),
        hand_id_text!(b"3"),
    ];

    pub fn besides(self) -> [HandId; (Self::COUNT - 1) as usize] {
        match self {
            HandId::Player => Self::CPUS,
            HandId::Cpu1 => [
                HandId::Cpu2,
                HandId::Cpu3,
                HandId::Player,
            ],
            HandId::Cpu2 => [
                HandId::Cpu3,
                HandId::Player,
                HandId::Cpu1,
            ],
            HandId::Cpu3 => [
                HandId::Player,
                HandId::Cpu1,
                HandId::Cpu2,
            ],
        }
    }

    pub fn next_to_current(self) -> [HandId; Self::COUNT as usize] {
        let [a, b, c] = self.besides();

        [a, b, c, self]
    }

    pub fn next_looping(self) -> Self {
        let mut i = 0;
        loop {
            if Self::ALL[i] == self {
                return Self::ALL[(i + 1) % Self::ALL.len()];
            }
            i += 1;
        }
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CpuId {
    #[default]
    One,
    Two,
    Three,
}

impl TryFrom<HandId> for CpuId {
    type Error = ();

    fn try_from(hand_id: HandId) -> Result<Self, Self::Error> {
        match hand_id {
            HandId::Player => Err(()),
            HandId::Cpu1 => Ok(CpuId::One),
            HandId::Cpu2 => Ok(CpuId::Two),
            HandId::Cpu3 => Ok(CpuId::Three),
        }
    }
}

impl Iterator for CpuId {
    type Item = CpuId;

    fn next(&mut self) -> Option<Self::Item> {
        match *self {
            CpuId::One => {
                *self = CpuId::Two;
                Some(*self)
            },
            CpuId::Two => {
                *self = CpuId::Three;
                Some(*self)
            },
            CpuId::Three => None,
        }
    }
}

macro_rules! has_suffix {
    () => {" has"}
}

impl CpuId {
    pub const COUNT: u8 = 3;

    pub const ALL: [Self; Self::COUNT as usize] = [
        Self::One,
        Self::Two,
        Self::Three,
    ];

    pub const TEXT: [&[u8]; Self::COUNT as usize] = [
        HandId::TEXT[1],
        HandId::TEXT[2],
        HandId::TEXT[3],
    ];

    pub const HAS_TEXT: [&[u8]; Self::COUNT as usize] = [
        concat!(hand_id_text!("1"), has_suffix!()).as_bytes(),
        concat!(hand_id_text!("2"), has_suffix!()).as_bytes(),
        concat!(hand_id_text!("3"), has_suffix!()).as_bytes(),
    ];

    pub fn wrapping_inc(self) -> Self {
        match self {
            CpuId::One => CpuId::Two,
            CpuId::Two => CpuId::Three,
            CpuId::Three => CpuId::One,
        }
    }

    pub fn wrapping_dec(self) -> Self {
        match self {
            CpuId::One => CpuId::Three,
            CpuId::Two => CpuId::One,
            CpuId::Three => CpuId::Two,
        }
    }
}