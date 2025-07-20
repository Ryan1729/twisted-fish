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
    suit as CardInner * RANK_COUNT + (rank as CardInner)
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
    // Worth 5 points {
    #[default]
    Barnacle,
    Crab,
    Dogfish,
    Eel,
    FlyingFish,
    Hammerhead,
    Jellyfish,
    Shrimp,
    // }
    // Worth 10 points {
    Blowfish,
    Clownfish,
    Starfish,
    Whale,
    // }
    // Worth 15 points {
    CardShark,
    // }
}

impl From<Rank> for u8 {
    fn from(rank: Rank) -> u8 {
        rank as u8
    }
}

macro_rules! rank_text {
    (0) => { "Barnacle" };
    (1) => { "Crab" };
    (2) => { "Dogfish" };
    (3) => { "Eel" };
    (4) => { "Flying Fish" };
    (5) => { "Hammerhead" };
    (6) => { "Jellyfish" };
    (7) => { "Shrimp" };
    (8) => { "Blowfish" };
    (9) => { "Clownfish" };
    (10) => { "Starfish" };
    (11) => { "Whale" };
    (12) => { "Card Shark" };
}

impl Rank {
    pub const COUNT: u8 = 13;

    pub const ALL: [Rank; Rank::COUNT as usize] = [
        Rank::Barnacle,
        Rank::Crab,
        Rank::Dogfish,
        Rank::Eel,
        Rank::FlyingFish,
        Rank::Hammerhead,
        Rank::Jellyfish,
        Rank::Shrimp,
        Rank::Blowfish,
        Rank::Clownfish,
        Rank::Starfish,
        Rank::Whale,
        Rank::CardShark,
    ];

    pub const TEXT: [&[u8]; Rank::COUNT as usize] = [
        rank_text!(0).as_bytes(),
        rank_text!(1).as_bytes(),
        rank_text!(2).as_bytes(),
        rank_text!(3).as_bytes(),
        rank_text!(4).as_bytes(),
        rank_text!(5).as_bytes(),
        rank_text!(6).as_bytes(),
        rank_text!(7).as_bytes(),
        rank_text!(8).as_bytes(),
        rank_text!(9).as_bytes(),
        rank_text!(10).as_bytes(),
        rank_text!(11).as_bytes(),
        rank_text!(12).as_bytes(),
    ];

    pub fn wrapping_dec(mut self, acb: AlmostCompleteBaskets) -> Self {
        for _ in 0..acb.len() {
            self = match self {
                Rank::Barnacle => Rank::CardShark,
                Rank::Crab => Rank::Barnacle,
                Rank::Dogfish => Rank::Crab,
                Rank::Eel => Rank::Dogfish,
                Rank::FlyingFish => Rank::Eel,
                Rank::Hammerhead => Rank::FlyingFish,
                Rank::Jellyfish => Rank::Hammerhead,
                Rank::Shrimp => Rank::Jellyfish,
                Rank::Blowfish => Rank::Shrimp,
                Rank::Clownfish => Rank::Blowfish,
                Rank::Starfish => Rank::Clownfish,
                Rank::Whale => Rank::Starfish,
                Rank::CardShark => Rank::Whale,
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
                Rank::Barnacle => Rank::Crab,
                Rank::Crab => Rank::Dogfish,
                Rank::Dogfish => Rank::Eel,
                Rank::Eel => Rank::FlyingFish,
                Rank::FlyingFish => Rank::Hammerhead,
                Rank::Hammerhead => Rank::Jellyfish,
                Rank::Jellyfish => Rank::Shrimp,
                Rank::Shrimp => Rank::Blowfish,
                Rank::Blowfish => Rank::Clownfish,
                Rank::Clownfish => Rank::Starfish,
                Rank::Starfish => Rank::Whale,
                Rank::Whale => Rank::CardShark,
                Rank::CardShark => Rank::Barnacle,
            };

            if acb[(self as u8) as usize].is_some() {
                break
            }
        }

        self
    }

}

pub fn get_rank(card: Card) -> Option<Rank> {
    if card >= FISH_COUNT {
        None
    } else {
        use Rank::*;
        match card % RANK_COUNT {
            0 => Some(Barnacle),
            1 => Some(Crab),
            2 => Some(Dogfish),
            3 => Some(Eel),
            4 => Some(FlyingFish),
            5 => Some(Hammerhead),
            6 => Some(Jellyfish),
            7 => Some(Shrimp),
            8 => Some(Blowfish),
            9 => Some(Clownfish),
            10 => Some(Starfish),
            11 => Some(Whale),
            12 => Some(CardShark),
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

macro_rules! suit_text {
    (0) => { "Red" };
    (1) => { "Green" };
    (2) => { "Blue" };
    (3) => { "Yellow" };
    (4) => { "Purple" };
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
        suit_text!(0).as_bytes(),
        suit_text!(1).as_bytes(),
        suit_text!(2).as_bytes(),
        suit_text!(3).as_bytes(),
        suit_text!(4).as_bytes(),
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

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Zinger {
    DeadScubaDiver,
    DivineIntervention,
    GlassBottomBoat,
    NoFishing,
    TheGameWarden,
    TheLure,
    TheNet,
    TwoFistedFisherman,
}

pub fn get_zinger(card: Card) -> Option<Zinger> {
    if (FISH_COUNT..DECK_SIZE).contains(&card) {
        use Zinger::*;
        Some(match card - FISH_COUNT {
            0 => DeadScubaDiver,
            1 => DivineIntervention,
            2 => GlassBottomBoat,
            3 => NoFishing,
            4 => TheGameWarden,
            5 => TheLure,
            6 => TheNet,
            7 => TwoFistedFisherman,
            _ => return None,
        })
    } else {
        None
    }
}

pub mod zingers {
    use super::*;

    pub const DEAD_SCUBA_DIVER: Card = zinger_card(Zinger::DeadScubaDiver);
    pub const DIVINE_INTERVENTION: Card = zinger_card(Zinger::DivineIntervention);
    pub const GLASS_BOTTOM_BOAT: Card = zinger_card(Zinger::GlassBottomBoat);
    pub const NO_FISHING: Card = zinger_card(Zinger::NoFishing);
    pub const THE_GAME_WARDEN: Card = zinger_card(Zinger::TheGameWarden);
    pub const THE_LURE: Card = zinger_card(Zinger::TheLure);
    pub const THE_NET: Card = zinger_card(Zinger::TheNet);
    pub const TWO_FISTED_FISHERMAN: Card = zinger_card(Zinger::TwoFistedFisherman);

    pub const ALL: [Card; 8] = [
        DEAD_SCUBA_DIVER,
        DIVINE_INTERVENTION,
        GLASS_BOTTOM_BOAT,
        NO_FISHING,
        THE_GAME_WARDEN,
        THE_LURE,
        THE_NET,
        TWO_FISTED_FISHERMAN,
    ];
}

pub const fn zinger_card(zinger: Zinger) -> Card {
    zinger as Card + FISH_COUNT
}

pub type HandLen = u8;

/// An ordered collection of cards that can hold at least one copy of each card.
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

    #[allow(dead_code)]
    // Currently used only in tests
    pub fn swap_insert_top(&mut self, card: Card) {
        let old_first = self.0[0];
        self.0[0] = CardOption::some(card);

        if let Some(old_first_card) = old_first.option() {
            self.push(old_first_card);
        }
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

    pub fn ordering_iter(&self, ordering: HandOrdering) -> impl Iterator<Item = Card> + '_ {
        compile_time_assert!{DECK_SIZE as u64 <= CardIndex::MAX as u64}

        let mut ordering_index = 0;
        std::iter::from_fn(move || {
            let i = ordering.0[ordering_index];

            ordering_index += 1;

            self.get(i)
        })
    }
}

#[derive(Clone, Debug)]
pub struct HandOrdering([CardIndex; DECK_SIZE as usize]);

impl Default for HandOrdering {
    fn default() -> Self {
        let mut indexes = [0; DECK_SIZE as usize];

        for i in 1..DECK_SIZE {
            indexes[i as usize] = i;
        }

        Self(indexes)
    }
}

impl HandOrdering {
    pub fn move_to_first(&mut self, to_move: CardIndex) {
        let mut currently_at = 0;
        for (i, e) in self.0.iter().enumerate() {
            if *e == to_move {
                currently_at = i;
                break
            }
        }

        let temp = self.0[currently_at];
        for i in (1..=currently_at).rev() {
            self.0[i] = self.0[i - 1];
        }
        self.0[0] = temp;

        self.assert_counts();
    }

    #[track_caller]
    fn assert_counts(&self) {
        let mut counts = [0; DECK_SIZE as usize];
        for &e in self.0.iter() {
            counts[e as usize] += 1;
        }

        for (i, &e) in counts.iter().enumerate() {
            assert_eq!(e, 1, "index {i} showed up not exactly once. {self:?} {counts:?}");
        }
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

pub type TurnId = HandId;

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

    pub fn with_target(self, target: HandId) -> Targeting {
        Targeting {
            source: self,
            target,
        }
    }
}

// TODO? Prevent self-targeting from being representable?
#[derive(Clone, Copy, Debug)]
pub struct Targeting {
    pub source: HandId,
    pub target: HandId,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NetPredicate {
    Suit(Suit),
    Rank(Rank),
}

impl Default for NetPredicate {
    fn default() -> Self {
        Self::Suit(<_>::default())
    }
}

impl NetPredicate {
    pub fn matches(self, card: Card) -> bool {
        use NetPredicate::*;
        match self {
            Suit(suit) => Some(suit) == get_suit(card),
            Rank(rank) => Some(rank) == get_rank(card),
        }
    }

    pub fn display_card(self) -> Card {
        zingers::THE_NET
    }

    pub const COUNT: u8 = 18;

    pub const ALL: [Self; Self::COUNT as usize] = [
        Self::Suit(Suit::ALL[0]),
        Self::Suit(Suit::ALL[1]),
        Self::Suit(Suit::ALL[2]),
        Self::Suit(Suit::ALL[3]),
        Self::Suit(Suit::ALL[4]),
        Self::Rank(Rank::ALL[0]),
        Self::Rank(Rank::ALL[1]),
        Self::Rank(Rank::ALL[2]),
        Self::Rank(Rank::ALL[3]),
        Self::Rank(Rank::ALL[4]),
        Self::Rank(Rank::ALL[5]),
        Self::Rank(Rank::ALL[6]),
        Self::Rank(Rank::ALL[7]),
        Self::Rank(Rank::ALL[8]),
        Self::Rank(Rank::ALL[9]),
        Self::Rank(Rank::ALL[10]),
        Self::Rank(Rank::ALL[11]),
        Self::Rank(Rank::ALL[12]),
    ];

    pub const TEXT: [&[u8]; Self::COUNT as usize] = [
        Suit::TEXT[0],
        Suit::TEXT[1],
        Suit::TEXT[2],
        Suit::TEXT[3],
        Suit::TEXT[4],
        Rank::TEXT[0],
        Rank::TEXT[1],
        Rank::TEXT[2],
        Rank::TEXT[3],
        Rank::TEXT[4],
        Rank::TEXT[5],
        Rank::TEXT[6],
        Rank::TEXT[7],
        Rank::TEXT[8],
        Rank::TEXT[9],
        Rank::TEXT[10],
        Rank::TEXT[11],
        Rank::TEXT[12],
    ];

    pub fn wrapping_inc(&mut self) {
        let index = self.index_of();
        *self = Self::ALL[if index >= Self::ALL.len() - 1 {
            0
        } else {
            index + 1
        }];
    }

    pub fn wrapping_dec(&mut self) {
        let index = self.index_of();
        *self = Self::ALL[if index == 0 {
            Self::ALL.len() - 1
        } else {
            index - 1
        }];
    }

    pub fn index_of(&self) -> usize {
        for i in 0..Self::ALL.len() {
            if Self::ALL[i] == *self {
                return i;
            }
        }

        unreachable!()
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LurePredicate {
    pub rank: Rank,
    pub suit: Suit,
}

macro_rules! l {
    ($suit_index: tt $rank_index: tt) => {
        Self {
            suit: Suit::ALL[$suit_index],
            rank: Rank::ALL[$rank_index],
        }
    }
}

macro_rules! t {
    ($suit_index: tt $rank_index: tt) => {
        concat!(suit_text!($suit_index), " ", rank_text!($rank_index)).as_bytes()
    }
}

impl LurePredicate {
    pub const COUNT: u8 = Suit::COUNT * Rank::COUNT;

    pub const ALL: [Self; Self::COUNT as usize] = [
        l!(0 0),
        l!(0 1),
        l!(0 2),
        l!(0 3),
        l!(0 4),
        l!(0 5),
        l!(0 6),
        l!(0 7),
        l!(0 8),
        l!(0 9),
        l!(0 10),
        l!(0 11),
        l!(0 12),
        l!(1 0),
        l!(1 1),
        l!(1 2),
        l!(1 3),
        l!(1 4),
        l!(1 5),
        l!(1 6),
        l!(1 7),
        l!(1 8),
        l!(1 9),
        l!(1 10),
        l!(1 11),
        l!(1 12),
        l!(2 0),
        l!(2 1),
        l!(2 2),
        l!(2 3),
        l!(2 4),
        l!(2 5),
        l!(2 6),
        l!(2 7),
        l!(2 8),
        l!(2 9),
        l!(2 10),
        l!(2 11),
        l!(2 12),
        l!(3 0),
        l!(3 1),
        l!(3 2),
        l!(3 3),
        l!(3 4),
        l!(3 5),
        l!(3 6),
        l!(3 7),
        l!(3 8),
        l!(3 9),
        l!(3 10),
        l!(3 11),
        l!(3 12),
        l!(4 0),
        l!(4 1),
        l!(4 2),
        l!(4 3),
        l!(4 4),
        l!(4 5),
        l!(4 6),
        l!(4 7),
        l!(4 8),
        l!(4 9),
        l!(4 10),
        l!(4 11),
        l!(4 12),
    ];

    pub const TEXT: [&[u8]; Self::COUNT as usize] = [
        t!(0 0),
        t!(0 1),
        t!(0 2),
        t!(0 3),
        t!(0 4),
        t!(0 5),
        t!(0 6),
        t!(0 7),
        t!(0 8),
        t!(0 9),
        t!(0 10),
        t!(0 11),
        t!(0 12),
        t!(1 0),
        t!(1 1),
        t!(1 2),
        t!(1 3),
        t!(1 4),
        t!(1 5),
        t!(1 6),
        t!(1 7),
        t!(1 8),
        t!(1 9),
        t!(1 10),
        t!(1 11),
        t!(1 12),
        t!(2 0),
        t!(2 1),
        t!(2 2),
        t!(2 3),
        t!(2 4),
        t!(2 5),
        t!(2 6),
        t!(2 7),
        t!(2 8),
        t!(2 9),
        t!(2 10),
        t!(2 11),
        t!(2 12),
        t!(3 0),
        t!(3 1),
        t!(3 2),
        t!(3 3),
        t!(3 4),
        t!(3 5),
        t!(3 6),
        t!(3 7),
        t!(3 8),
        t!(3 9),
        t!(3 10),
        t!(3 11),
        t!(3 12),
        t!(4 0),
        t!(4 1),
        t!(4 2),
        t!(4 3),
        t!(4 4),
        t!(4 5),
        t!(4 6),
        t!(4 7),
        t!(4 8),
        t!(4 9),
        t!(4 10),
        t!(4 11),
        t!(4 12),
    ];

    pub fn wrapping_inc(&mut self) {
        let index = self.index_of();
        *self = Self::ALL[if index >= Self::ALL.len() - 1 {
            0
        } else {
            index + 1
        }];
    }

    pub fn wrapping_dec(&mut self) {
        let index = self.index_of();
        *self = Self::ALL[if index == 0 {
            Self::ALL.len() - 1
        } else {
            index - 1
        }];
    }

    pub fn index_of(&self) -> usize {
        for i in 0..Self::ALL.len() {
            if Self::ALL[i] == *self {
                return i;
            }
        }

        unreachable!()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Predicate {
    RankSuit(Rank, Suit),
    Net(NetPredicate),
}

impl Predicate {
    pub fn matches(self, card: Card) -> bool {
        use Predicate::*;
        match self {
            RankSuit(rank, suit) => card == fish_card(rank, suit),
            Net(net_predicate) => net_predicate.matches(card),
        }
    }

    pub fn display_card(self) -> Card {
        use Predicate::*;
        match self {
            RankSuit(rank, suit) => fish_card(rank, suit),
            Net(net_predicate) => net_predicate.display_card(),
        }
    }
}