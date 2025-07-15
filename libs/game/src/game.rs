use memories::Memories;
use models::{Basket, Card, CardIndex, CpuId, Hand, HandId, HandOrdering, Predicate, LurePredicate, NetPredicate, Rank, Suit, Targeting, Zinger, DECK_SIZE, fish_card, get_rank, get_suit, zingers};
use gfx::{Commands, CHEVRON_H, WINDOW_CONTENT_OFFSET};
use platform_types::{
    command,
    unscaled::{self, X, Y, XY, W, H, WH, Rect, x_const_add_w, w_const_sub},
    Button,
    Dir,
    Input,
    Speaker,
    SFX,
    CARD_WIDTH,
    CARD_HEIGHT,
    WHITE,
};
use xs::{Xs, Seed};

macro_rules! allow_to_respond {
    ($state: ident) => {
        $state.sub_turn_ids = $state.turn_id.next_to_current();
        $state.sub_turn_index = 0;
        $state.done_something_this_turn = true;
    }
}

macro_rules! cpu_handle_negative_response {
    ($state: ident, $menu: ident, $id: ident, $rank: ident, $suit: expr) => {
        let hand_id = $id.into();

        if can_and_should_play_two_fisted_fisherman(
            &$state.memories,
            $state.cards.hand(hand_id),
            $id,
            $state.cards.active_count(),
        ) {
            discard_two_fisted_fisherman(
                &mut $state.cards,
                &mut $state.animations,
                hand_id,
            );
            *$menu = CpuMenu::WaitingWhenPlayedTwoFistedFisherman;
            $state.done_something_this_turn = true;

            $state.stack.push(Play {
                sub_turn_ids: hand_id.next_to_current(),
                sub_turn_index: 0,
                kind: PlayKind::TwoFistedFisherman {
                    source: hand_id,
                    cancelled: false,
                },
            });
            allow_to_respond!($state);
        } else {
            let suit = $suit;
            let target_card = fish_card($rank, suit);
            let my_len = $state.cards.hand(hand_id).len();

            let card_option = $state.cards.deck.draw();

            if let Some(card) = card_option {
                let at = DECK_XY;

                let target = get_card_insert_position(
                    spread(hand_id),
                    my_len
                );

                $state.animations.push(Animation {
                    card,
                    at,
                    target,
                    action: AnimationAction::AddToHand(hand_id),
                    .. <_>::default()
                });

                if card == target_card {
                    $state.memories.fished_for(hand_id, $rank, suit);

                    *$menu = CpuMenu::WaitingWhenGotWhatWasFishingFor;
                    $state.done_something_this_turn = true;
                } else {
                    $state.stack.push(Play {
                        sub_turn_ids: hand_id.next_to_current(),
                        sub_turn_index: 0,
                        kind: PlayKind::FishedUnsuccessfully {
                            source: hand_id,
                        }
                    });
                    allow_to_respond!($state);
                }
            } else {
                $state.stack.push(Play {
                    sub_turn_ids: hand_id.next_to_current(),
                    sub_turn_index: 0,
                    kind: PlayKind::FishedUnsuccessfully {
                        source: hand_id,
                    }
                });
                allow_to_respond!($state);
            }
        }
    }
}

macro_rules! to_next_turn {
    ($state: ident) => {
        // Time for the next turn.
        $state.turn_id = $state.turn_id.next_looping();
        $state.sub_turn_index = HandId::COUNT + 1;
        $state.selection.card_index = $state.cards.player.len().saturating_sub(1);
        $state.selection.player_menu = Default::default();
        $state.cpu_menu = CpuMenu::default();
        $state.done_something_this_turn = false;
    }
}

const DECK_XY: XY = XY {
    x: X((command::WIDTH - CARD_WIDTH.get()) / 2),
    y: Y((command::HEIGHT - CARD_HEIGHT.get()) / 2),
};

const DISCARD_XY: XY = XY {
    x: X(
        ((command::WIDTH - (CARD_WIDTH.get() * 3)) / 2)
        - 2
    ),
    y: Y((command::HEIGHT - CARD_HEIGHT.get()) / 2),
};

const PLAYER_BASE_XY: XY = XY {
    x: X(CARD_WIDTH.get() * 5 / 4),
    y: Y(command::HEIGHT - CARD_HEIGHT.get()),
};

const CPU1_BASE_XY: XY = XY {
    x: X(0),
    y: Y(CARD_HEIGHT.get() / 2),
};

const CPU2_BASE_XY: XY = XY {
    x: X(CARD_WIDTH.get() * 5 / 4),
    y: Y(0),
};

const CPU3_BASE_XY: XY = XY {
    x: X(command::WIDTH - CARD_WIDTH.get()),
    y: Y(CARD_HEIGHT.get() / 2),
};

fn in_front_of(id: HandId) -> XY {
    const HALF_W: W = W(CARD_WIDTH.get() / 2);
    const HALF_H: H = H(CARD_HEIGHT.get() / 2);

    match spread(id) {
        Spread::LTR((x1, x2), y) => XY {
             x: x1 + ((x2 - x1) / 2) - HALF_W,
             y: match id {
                HandId::Player
                | HandId::Cpu3 => y - HALF_H,
                HandId::Cpu1
                | HandId::Cpu2 => y + HALF_H,
            },
        },
        Spread::TTB((y1, y2), x) => XY {
             x: match id {
                HandId::Player
                | HandId::Cpu3 => x - HALF_W,
                HandId::Cpu1
                | HandId::Cpu2 => x + HALF_W,
            },
             y: y1 + ((y2 - y1) / 2) - HALF_H,
        },
    }
}

pub enum Spread {
    /// Left To Right
    LTR((X, X), Y),
    /// Top To Bottom
    TTB((Y, Y), X),
}

pub const PLAYER_SPREAD: Spread = Spread::LTR(
    (
        PLAYER_BASE_XY.x,
        x_const_add_w(X(0), w_const_sub(command::WIDTH_W, CARD_WIDTH))
    ),
    PLAYER_BASE_XY.y
);

pub const CPU1_SPREAD: Spread = Spread::TTB(
    (
        CPU1_BASE_XY.y,
        Y(command::HEIGHT - (CARD_HEIGHT.get() / 2))
    ),
    CPU1_BASE_XY.x
);

pub const CPU2_SPREAD: Spread = Spread::LTR(
    (
        CPU2_BASE_XY.x,
        x_const_add_w(X(0), w_const_sub(command::WIDTH_W, CARD_WIDTH))
    ),
    CPU2_BASE_XY.y
);

pub const CPU3_SPREAD: Spread = Spread::TTB(
    (
        CPU3_BASE_XY.y,
        Y(command::HEIGHT - (CARD_HEIGHT.get() / 2)),
    ),
    CPU3_BASE_XY.x
);

pub const fn spread(id: HandId) -> Spread {
    match id {
        HandId::Player => PLAYER_SPREAD,
        HandId::Cpu1 => CPU1_SPREAD,
        HandId::Cpu2 => CPU2_SPREAD,
        HandId::Cpu3 => CPU3_SPREAD,
    }
}

#[derive(Clone, Copy)]
pub struct Animations(pub [Animation; DECK_SIZE as usize]);

impl Default for Animations {
    fn default() -> Self {
        Self([Animation::default(); DECK_SIZE as usize])
    }
}

impl Animations {
    pub fn iter(&self) -> impl Iterator<Item = &Animation> + '_ {
        self.0.iter().filter(|anim| !anim.is_done())
    }

    pub fn push(&mut self, anim: Animation) {
        let animations = &mut self.0;
        for i in 0..animations.len() {
            if animations[i].is_done() {
                animations[i] = anim;

                break;
            }
        }
    }

    pub fn all_done(&self) -> bool {
        self.iter().count() == 0
    }
}

pub type Frames = u8;

#[derive(Clone, Copy, Default)]
pub struct Animation {
    pub delay: Frames,
    pub card: Card,
    pub at: XY,
    pub target: XY,
    pub action: AnimationAction,
    pub shown: bool,
}

impl Animation {
    pub fn is_done(&self) -> bool {
        self.at == self.target
    }

    pub fn is_active(&self) -> bool {
        self.delay == 0
    }
}

#[derive(Clone, Copy, Default)]
pub enum AfterDiscard {
    #[default]
    Nothing,
    BackToSelecting(HandId),
    PushPlay(Play),
}

#[derive(Clone, Copy, Default)]
pub enum AnimationAction {
    #[default]
    DoNothing,
    AddToHand(HandId),
    PerformGameWarden,
    AddToDiscard(AfterDiscard),
    AnimateBackToHand(HandId),
}

mod question {
    use super::*;

    #[derive(Clone, Debug)]
    pub struct Question {
        pub target: HandId,
        pub suit: Suit,
        description: Vec<u8>,
    }

    impl Default for Question {
        fn default() -> Self {
            Self {
                target: HandId::Cpu1,
                suit: Suit::default(),
                description: Vec::default(),
            }
        }
    }

    impl Question {
        pub fn fresh_ask_description(
            &mut self,
            rank: Rank
        ) -> &[u8] {
            self.description.clear();
            self.description.reserve(128);

            self.description.extend_from_slice(
                HandId::TEXT[usize::from(self.target as u8)]
            );

            self.description.extend_from_slice(
                b", do you have the "
            );

            self.description.extend_from_slice(
                Suit::TEXT[usize::from(self.suit as u8)]
            );

            self.description.push(b' ');

            self.description.extend_from_slice(
                Rank::TEXT[usize::from(rank as u8)]
            );

            self.description.push(b'?');

            &self.description
        }

        pub fn fresh_cpu_ask_description(
            &mut self,
            rank: Rank,
            me: HandId,
            width: W,
        ) -> &[u8] {
            self.description.clear();
            self.description.reserve(128);

            self.description.extend_from_slice(
                HandId::TEXT[usize::from(me as u8)]
            );

            self.description.extend_from_slice(
                b" asks \""
            );

            self.description.extend_from_slice(
                HandId::TEXT[usize::from(self.target as u8)]
            );

            self.description.extend_from_slice(
                b", do you have the "
            );

            self.description.extend_from_slice(
                Suit::TEXT[usize::from(self.suit as u8)]
            );

            self.description.push(b' ');

            self.description.extend_from_slice(
                Rank::TEXT[usize::from(rank as u8)]
            );

            self.description.push(b'?');
            self.description.push(b'"');

            let width_in_chars = usize::from(width / gfx::CHAR_ADVANCE_W.get().get());

            text::bytes_reflow_in_place(&mut self.description, width_in_chars);

            &self.description
        }

        pub fn fresh_fished_description(
            &mut self,
            rank: Rank,
            drew: Option<Card>,
            width: W,
        ) -> &[u8] {
            self.description.clear();
            self.description.reserve(128);

            self.description.extend_from_slice(
                b"You asked for the "
            );

            self.description.extend_from_slice(
                Suit::TEXT[usize::from(self.suit as u8)]
            );

            self.description.push(b' ');

            self.description.extend_from_slice(
                Rank::TEXT[usize::from(rank as u8)]
            );

            self.description.push(b' ');

            if let Some(card) = drew {
                let target_card = fish_card(rank, self.suit);
                if card == target_card {
                    self.description.extend_from_slice(
                        b"and you got what you asked for!"
                    );
                } else {
                    self.description.extend_from_slice(
                        b"but you didn't get it."
                    );
                }
            } else {
                self.description.extend_from_slice(
                    b"but the fish pond was empty!"
                );
            }

            let width_in_chars = usize::from(width / gfx::CHAR_ADVANCE_W.get().get());

            text::bytes_reflow_in_place(&mut self.description, width_in_chars);

            &self.description
        }
    }
}
use question::Question;

#[derive(Copy, Clone, Default)]
pub struct PlayerSelection {
    target: CpuId,
    card: AnytimeCard,
    declined: bool,
    viewing: Option<Card>,
    rank: Rank,
}

#[derive(Clone)]
pub enum Menu {
    PlayerTurn { selected: CardIndex, menu: PlayerMenu },
    CpuTurn{ id: CpuId, menu: CpuMenu },
    BetweenTurns {
        next_id: HandId,
        player_selection: PlayerSelection,
    },
}

impl Default for Menu {
    fn default() -> Menu {
        Menu::player(CardIndex::default())
    }
}

impl Menu {
    fn player(selected: CardIndex) -> Self {
        Menu::PlayerTurn {
            selected,
            menu: PlayerMenu::default(),
        }
    }
}

#[derive(Clone)]
pub enum PlayerMenu {
    Selecting { sub_menu: PlayerSelectingSubMenu },
    Asking {
        used: Card,
        question: Question,
        sub_menu: PlayerAskingSubMenu,
    },
    Fished {
        used: Card,
        question: Question,
        drew: Option<Card>
    },
    Net {
        target: CpuId,
        predicate: NetPredicate,
    },
    Lure {
        target: CpuId,
        predicate: LurePredicate,
    },
}

impl Default for PlayerMenu {
    fn default() -> PlayerMenu {
        PlayerMenu::Selecting { sub_menu: <_>::default() }
    }
}

#[derive(Clone, Default)]
pub enum PlayerSelectingSubMenu {
    #[default]
    Root,
    Anytime(PlayerSelection, AvailablePlayAnytime),
    Message(Vec<u8>),
}

#[derive(Clone, Default)]
pub enum PlayerAskingSubMenu {
    #[default]
    Root,
    TwoFistedFisherman,
}

#[derive(Clone, Debug, Default)]
pub enum CpuMenu {
    #[default]
    Selecting,
    Asking(Rank, Question),
    DeadInTheWater,
    WaitingForSuccesfulAsk,
    WaitingWhenGotWhatWasFishingFor,
    WaitingWhenPlayedTwoFistedFisherman,
}

#[derive(Clone, Default)]
pub struct Cards {
    pub deck: Hand,
    pub player: Hand,
    pub cpu1: Hand,
    pub cpu2: Hand,
    pub cpu3: Hand,
    pub player_baskets: Hand,
    pub cpu1_baskets: Hand,
    pub cpu2_baskets: Hand,
    pub cpu3_baskets: Hand,
    pub discard: Hand,
}

impl Cards {
    pub fn hand(&self, id: HandId) -> &Hand {
        match id {
            HandId::Player => &self.player,
            HandId::Cpu1 => &self.cpu1,
            HandId::Cpu2 => &self.cpu2,
            HandId::Cpu3 => &self.cpu3,
        }
    }

    pub fn hand_mut(&mut self, id: HandId) -> &mut Hand {
        match id {
            HandId::Player => &mut self.player,
            HandId::Cpu1 => &mut self.cpu1,
            HandId::Cpu2 => &mut self.cpu2,
            HandId::Cpu3 => &mut self.cpu3,
        }
    }

    fn active_count(&self) -> ActiveCardCount {
        let count = self.deck.len()
        + self.player.len()
        + self.cpu1.len()
        + self.cpu2.len()
        + self.cpu3.len();

        if count <= Suit::COUNT * 3 {
            ActiveCardCount::VeryFew
        } else {
            ActiveCardCount::Several
        }
    }

    fn played_zinger_count(&self) -> CardCount {
        let mut count = 0;

        for hand in [
            &self.player_baskets,
            &self.cpu1_baskets,
            &self.cpu2_baskets,
            &self.cpu3_baskets,
            &self.discard,
        ] {
            for card in hand.iter() {
                if let Some(_) = models::get_zinger(card) {
                    count += 1;
                }
            }
        }

        count
    }
}

pub enum FullHandId {
    Deck,
    Player,
    Cpu1,
    Cpu2,
    Cpu3,
    PlayerBaskets,
    Cpu1Baskets,
    Cpu2Baskets,
    Cpu3Baskets,
    Discard,
}

fn force_into_start_of_hand(
    state: &mut State,
    target_card: Card,
    hand_id: FullHandId
) {
    use FullHandId::*;

    let hands = [
        &mut state.cards.deck,
        &mut state.cards.player,
        &mut state.cards.cpu1,
        &mut state.cards.cpu2,
        &mut state.cards.cpu3,
        &mut state.cards.player_baskets,
        &mut state.cards.cpu1_baskets,
        &mut state.cards.cpu2_baskets,
        &mut state.cards.cpu3_baskets,
        &mut state.cards.discard,
    ];

    let mut extracted_card = <_>::default();

    for hand in hands {
        let index_opt = hand.iter().position(|c| c == target_card);

        if let Some(index) = index_opt {
            extracted_card = hand.remove(index.try_into().expect("index should fit in a CardIndex"));
            break
        }
    }

    let target_hand = match hand_id {
        Deck => &mut state.cards.deck,
        Player => &mut state.cards.player,
        Cpu1 => &mut state.cards.cpu1,
        Cpu2 => &mut state.cards.cpu2,
        Cpu3 => &mut state.cards.cpu3,
        PlayerBaskets => &mut state.cards.player_baskets,
        Cpu1Baskets => &mut state.cards.cpu1_baskets,
        Cpu2Baskets => &mut state.cards.cpu2_baskets,
        Cpu3Baskets => &mut state.cards.cpu3_baskets,
        Discard => &mut state.cards.discard,
    };

    target_hand.swap_insert_top(extracted_card.expect("card should have been found"));
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ActiveCardCount {
    Several,
    VeryFew
}

type CardCount = u8;

/// This stores all the possible selections that a player can make as part of their
/// turn, including ones that only make sense when playing a particular card. It is
/// expected that upon transitioning to a game state where the player can make a
/// choice this will be cleared to a sensible default state for that game state.
#[derive(Clone, Default)]
pub struct Selection {
    card_index: CardIndex,
    player_selection: PlayerSelection,
    player_menu: PlayerMenu,
}

#[derive(Clone, Copy, Debug)]
pub enum PlayKind {
    FishedUnsuccessfully {
        source: HandId,
    },
    NoFishing {
        targeting: Targeting,
        predicate: Predicate,
    },
    TwoFistedFisherman {
        source: HandId,
        cancelled: bool,
    },
    TheNet {
        targeting: Targeting,
        predicate: NetPredicate,
    },
}

impl PlayKind {
    fn is_zinger(&self) -> bool {
        match self {
            Self::FishedUnsuccessfully { .. } => false,
            Self::NoFishing { .. }
            | Self::TwoFistedFisherman { .. }
            | Self::TheNet { .. } => true,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Play {
    pub sub_turn_ids: [HandId; HandId::COUNT as usize],
    pub sub_turn_index: u8,
    pub kind: PlayKind,
}

impl Play {
    fn is_zinger(&self) -> bool {
        self.kind.is_zinger()
    }
}

#[derive(Clone, Default)]
pub struct State {
    pub rng: Xs,
    pub cards: Cards,
    pub animations: Animations,
    pub ctx: ui::Context,
    pub memories: Memories,
    pub has_started: bool,
    pub selection: Selection,
    pub turn_id: HandId,
    pub sub_turn_ids: [HandId; HandId::COUNT as usize],
    pub sub_turn_index: u8,
    pub stack: Vec<Play>,
    pub cpu_menu: CpuMenu,
    pub done_something_this_turn: bool,
}

impl State {
    pub fn new(
        #[allow(unused_variables)]
        mut seed: Seed
    ) -> State {
        // For debugging
        #[allow(dead_code)]
        enum HardcodedMode {
            Release,
            PlayerMultipleZingers,
            Cpu1GameWarden,
            PlayerGlassBottomBoat,
            PlayerGameWardenAndGlassBottomBoat,
            Cpu2DeadScubaDiverAndNoFishing,
            PlayerNetAndNoFishing,
            PlayerStuckWithDivineIntervention,
        }
        use HardcodedMode::*;


        let mode = PlayerStuckWithDivineIntervention;

        let mut initial_hand_size: u8 = 8; //16;

        match mode {
            Release => {},
            PlayerMultipleZingers => {
                // Gives player multiple zingers. (8)
                seed = [150, 148, 11, 45, 255, 227, 216, 65, 225, 81, 35, 202, 235, 145, 4, 62];
            },
            Cpu1GameWarden => {
                // Gives Cpu1 the game warden (8)
                seed = [168, 63, 217, 43, 183, 228, 216, 65, 56, 191, 2, 192, 83, 145, 4, 62];
            },
            PlayerGlassBottomBoat => {
                // Gives player glass bottom boat. (8)
                seed = [233, 217, 2, 79, 186, 228, 216, 65, 146, 77, 106, 40, 81, 145, 4, 62];
            },
            PlayerGameWardenAndGlassBottomBoat => {
                // Gives player the game warden and glass bottom boat. (16)
                seed = [162, 35, 66, 102, 63, 230, 216, 65, 211, 81, 226, 193, 15, 144, 4, 62];
                initial_hand_size = 16;
            },
            Cpu2DeadScubaDiverAndNoFishing => {
                // Gives Cpu2 the dead scuba diver and no fishing. (8)
                seed = [146, 115, 135, 54, 37, 236, 216, 65, 70, 182, 129, 14, 50, 139, 4, 62];
            },
            PlayerNetAndNoFishing => {
                // Gives player the net and no fishing. (8)
                seed = [130, 162, 218, 177, 150, 236, 216, 65, 146, 44, 249, 132, 212, 138, 4, 62];
            },
            PlayerStuckWithDivineIntervention => {},
        }

        let mut rng = xs::from_seed(seed);

        let mut state = State {
            rng,
            cards: Cards {
                deck: Hand::fresh_deck(&mut rng),
                .. <_>::default()
            },
            sub_turn_ids: HandId::ALL,
            // Don't start in a sub turn
            sub_turn_index: HandId::COUNT + 1,
            // TODO Randomize starting turn
            turn_id: HandId::Player,
            .. <_>::default()
        };

        match mode {
            PlayerStuckWithDivineIntervention => {
                for zinger in models::zingers::ALL {
                    force_into_start_of_hand(
                        &mut state,
                        zinger,
                        if zinger == zingers::DIVINE_INTERVENTION {
                            FullHandId::Player
                        } else {
                            FullHandId::Discard
                        }
                    );
                }
            },
            _ => {}
        }

        for card_i in 0..initial_hand_size {
            // TODO Once starting turn is randomized, deal cards to first player
            // first.
            for (id_i, id) in HandId::ALL.into_iter().enumerate() {
                let card = match state.cards.deck.draw() {
                    Some(card) => card,
                    None => continue,
                };

                let target = get_card_insert_position(
                    spread(id),
                    card_i + 1,
                );

                state.animations.push(Animation {
                    card,
                    at: DECK_XY,
                    target,
                    action: AnimationAction::AddToHand(id),
                    delay: card_i
                        .saturating_mul(HandId::ALL.len() as u8)
                        .saturating_add(id_i as u8),
                    .. <_>::default()
                });
            }
        }

        state
    }

    pub fn tick(&mut self, speaker: &mut Speaker) {
        use core::cmp::{min, Ordering::*};

        let mut push_after = Vec::new();
        for anim in self.animations.0.iter_mut() {
            if anim.is_done() { continue }

            if anim.delay > 0 {
                anim.delay -= 1;
                continue
            }

            let x_rate = 3;
            let y_rate = 3;
            let d_x = match anim.at.x.cmp(&anim.target.x) {
                Equal => 0,
                Greater => {
                    let x_diff = anim.at.x - anim.target.x;
                    -(min(x_diff.get(), x_rate) as i8)
                },
                Less => {
                    let x_diff = anim.target.x - anim.at.x;
                    min(x_diff.get(), x_rate) as i8
                },
            };
            let d_y = match anim.at.y.cmp(&anim.target.y) {
                Equal => 0,
                Greater => {
                    let y_diff = anim.at.y - anim.target.y;
                    -(min(y_diff.get(), y_rate) as i8)
                },
                Less => {
                    let y_diff = anim.target.y - anim.at.y;
                    min(y_diff.get(), y_rate) as i8
                },
            };

            anim.at.x = match d_x {
                d_x if d_x > 0 => anim.at.x.saturating_add(W(d_x as unscaled::Inner)),
                d_x if d_x < 0 => anim.at.x.saturating_sub(W(d_x.unsigned_abs() as unscaled::Inner)),
                _ => anim.at.x,
            };
            anim.at.y = match d_y {
                d_y if d_y > 0 => anim.at.y.saturating_add(H(d_y as unscaled::Inner)),
                d_y if d_y < 0 => anim.at.y.saturating_sub(H(d_y.unsigned_abs() as unscaled::Inner)),
                _ => anim.at.y,
            };

            if anim.is_done() {
                macro_rules! back_to_selecting {
                    ($id: ident) => ({
                        let hand = match $id {
                            HandId::Player => &mut self.cards.player,
                            HandId::Cpu1 => &mut self.cards.cpu1,
                            HandId::Cpu2 => &mut self.cards.cpu2,
                            HandId::Cpu3 => &mut self.cards.cpu3,
                        };

                        match CpuId::try_from($id) {
                            Err(_) => {
                                self.selection.card_index = hand.len() - 1;
                            },
                            Ok(_) => {
                                if matches!(
                                    self.cpu_menu,
                                    CpuMenu::WaitingForSuccesfulAsk
                                    | CpuMenu::WaitingWhenGotWhatWasFishingFor
                                    | CpuMenu::WaitingWhenPlayedTwoFistedFisherman
                                ) {
                                    self.cpu_menu = CpuMenu::Selecting;
                                    self.done_something_this_turn = true;
                                }
                            }
                        }
                    })
                }

                match anim.action {
                    AnimationAction::DoNothing => {},
                    AnimationAction::AddToHand(id) => {
                        let hand = match id {
                            HandId::Player => &mut self.cards.player,
                            HandId::Cpu1 => &mut self.cards.cpu1,
                            HandId::Cpu2 => &mut self.cards.cpu2,
                            HandId::Cpu3 => &mut self.cards.cpu3,
                        };

                        hand.push(anim.card);

                        let baskets = match id {
                            HandId::Player => &mut self.cards.player_baskets,
                            HandId::Cpu1 => &mut self.cards.cpu1_baskets,
                            HandId::Cpu2 => &mut self.cards.cpu2_baskets,
                            HandId::Cpu3 => &mut self.cards.cpu3_baskets,
                        };

                        fn remove_basket(hand: &mut Hand) -> Option<Basket> {
                            let mut indexes = [None; Suit::COUNT as usize];

                            // TODO? Do we care about this being O(n^2), given that
                            // we know n is bounded by `DECK_SIZE`, and in fact
                            // would be smaller in practice?
                            'outer: for first_card in hand.iter() {
                                let Some(rank) = get_rank(first_card) else {
                                    continue
                                };
                                indexes = [None; Suit::COUNT as usize];
                                for (card_i, card) in hand.enumerated_iter() {
                                    match models::get_rank(card) {
                                        Some(r) if rank == r => {
                                            for i in 0..Suit::COUNT as usize {
                                                if indexes[i].is_none() {
                                                    indexes[i] = Some(card_i);
                                                    if i >= Suit::COUNT as usize - 1 {
                                                        break 'outer
                                                    } else {
                                                        break
                                                    }
                                                }
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                            }

                            match indexes {
                                [Some(a), Some(b), Some(c), Some(d), Some(e)] => {
                                    const MSG: &str = "remove_basket indexes should be valid!";
                                    // We assume that the indexes are in ascending
                                    // order, so removing in reverse order doesn't
                                    // invalidate any indexes.
                                    let c1 = hand.remove(e).expect(MSG);
                                    let c2 = hand.remove(d).expect(MSG);
                                    let c3 = hand.remove(c).expect(MSG);
                                    let c4 = hand.remove(b).expect(MSG);
                                    let c5 = hand.remove(a).expect(MSG);

                                    Some([c1, c2, c3, c4, c5])
                                }
                                _ => None,
                            }
                        }

                        while let Some(basket) = remove_basket(hand) {
                            self.memories.basket_removed(basket);
                            // TODO? animate gathering together and heading to a
                            // separate pile? Or maybe poofing in an expolsion of
                            // particles?
                            for card in basket {
                                baskets.push(card);
                            }
                        }

                        speaker.request_sfx(SFX::CardPlace);

                        back_to_selecting!(id);
                    },
                    AnimationAction::PerformGameWarden => {
                        // TODO Animate all cards in deck moving to random targets
                        // away from, then back to DECK_XY.

                        self.cards.deck.push(anim.card);
                        self.cards.deck.shuffle(&mut self.rng);
                    }
                    AnimationAction::AddToDiscard(after_discard) => {
                        self.cards.discard.push(anim.card);

                        speaker.request_sfx(SFX::CardPlace);

                        match after_discard {
                            AfterDiscard::BackToSelecting(id)
                                => back_to_selecting!(id),
                            AfterDiscard::Nothing => {}
                            AfterDiscard::PushPlay(play) => {
                                self.stack.push(play);
                                allow_to_respond!(self);
                            }
                        }
                    }
                    AnimationAction::AnimateBackToHand(id) => {
                        let target = get_card_insert_position(
                            spread(id),
                            self.cards.hand(id).len(),
                        );

                        push_after.push(Animation {
                            card: anim.card,
                            at: anim.target,
                            target,
                            action: AnimationAction::AddToHand(id),
                            .. <_>::default()
                        })
                    }
                }
            }
        }

        for anim in push_after {
            self.animations.push(anim);
        }
    }
}

mod ui {
    use super::*;

    /// A group of things that are used together to render UI. Naming suggestions
    /// welcome!
    pub(crate) struct Group<'commands, 'ctx, 'speaker> {
        pub commands: &'commands mut Commands,
        pub ctx: &'ctx mut Context,
        pub input: Input,
        pub speaker: &'speaker mut Speaker,
    }

    #[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
    pub enum Id {
        #[default]
        Zero,
        Submit,
        CpuIdSelect,
        AskSuit,
        AnytimeCard,
        RankSelect,
        NetPredicate,
        LurePredicate,
    }

    #[derive(Copy, Clone, Default, Debug)]
    pub struct Context {
        pub active: Id,
        pub hot: Id,
        pub next_hot: Id,
    }

    impl Context {
        pub fn set_not_active(&mut self) {
            self.active = Id::Zero;
        }
        pub fn set_active(&mut self, id: Id) {
            self.active = id;
        }
        pub fn set_next_hot(&mut self, id: Id) {
            self.next_hot = id;
        }
        pub fn set_not_hot(&mut self) {
            self.hot = Id::Zero;
        }
        pub fn frame_init(&mut self) {
            if self.active == Id::Zero {
                self.hot = self.next_hot;
            }
            self.next_hot = Id::Zero;
        }
    }

    pub(crate) struct ButtonSpec<'text> {
        pub id: Id,
        pub rect: unscaled::Rect,
        pub text: &'text [u8],
    }

    pub(crate) fn button_press<'commands, 'ctx, 'speaker>(
        group: &mut Group<'commands, 'ctx, 'speaker>,
        id: Id,
    ) -> bool {
        let mut output = false;

        if group.ctx.active == id {
            if group.input.released_this_frame(Button::A) {
                output = group.ctx.hot == id;

                group.ctx.set_not_active();
            }
            group.ctx.set_next_hot(id);
        } else if group.ctx.hot == id {
            if group.input.pressed_this_frame(Button::A) {
                group.ctx.set_active(id);
                group.speaker.request_sfx(SFX::ButtonPress);
            }
            group.ctx.set_next_hot(id);
        }

        output
    }

    pub(crate) fn do_button<'commands, 'ctx, 'speaker, 'text>(
        group: &mut Group<'commands, 'ctx, 'speaker>,
        spec: ButtonSpec<'text>,
    ) -> bool {
        use gfx::NineSlice as ns;
        let id = spec.id;

        let result = button_press(group, id);

        if group.ctx.active == id && group.input.gamepad.contains(Button::A) {
            group.commands.draw_nine_slice(ns::ButtonPressed, spec.rect);
        } else if group.ctx.hot == id {
            group.commands.draw_nine_slice(ns::ButtonHot, spec.rect);
        } else {
            group.commands.draw_nine_slice(ns::Button, spec.rect);
        }

        let xy = gfx::center_line_in_rect(
            spec.text.len() as _,
            spec.rect,
        );

        //Long labels aren't great UX anyway, I think, so don't bother reflowing.
        group.commands.print_line(
            spec.text,
            xy,
            WHITE
        );

        result
    }

    /// As a user of this `fn` you are expected to have drawn the separate states
    /// that are selected between before calling this, in the given rect.
    pub(crate) fn draw_quick_select<'commands, 'ctx, 'speaker, 'text>(
        group: &mut Group<'commands, 'ctx, 'speaker>,
        rect: Rect,
        id: Id,
    ) {
        let mut highlighted = gfx::Highlighted::No;
        if group.ctx.active == id
        || group.ctx.hot == id {
            group.ctx.set_next_hot(id);
            highlighted = gfx::Highlighted::Yes;
        }

        let x = (rect.x + (rect.w / 2)) - (gfx::CHEVRON_W / 2);

        group.commands.draw_chevron(
            XY {
                x,
                y: rect.y,
            },
            gfx::ChevronDir::Up,
            highlighted,
        );

        group.commands.draw_chevron(
            XY {
                x,
                y: rect.y + rect.h - gfx::CHEVRON_H,
            },
            gfx::ChevronDir::Down,
            highlighted,
        );
    }
}

use ui::{ButtonSpec, Id::*, do_button};

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
#[allow(non_camel_case_types)]
pub enum PlayAnytimeFlags {
    GW = 1,
    GBB = 2,
    GBB_GW = 3,
    DSD = 4,
    DSD_GW = 5,
    DSD_GBB = 6,
    DSD_GBB_GW = 7,
}

impl core::ops::BitOrAssign for PlayAnytimeFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        use PlayAnytimeFlags::*;
        *self = match (*self as u8) | (rhs as u8) {
            1 => GW,
            2 => GBB,
            3 => GBB_GW,
            4 => DSD,
            5 => DSD_GW,
            6 => DSD_GBB,
            7 => DSD_GBB_GW,
            _ => unreachable!()
        };
    }
}

impl core::ops::BitOr for PlayAnytimeFlags {
    type Output = Self;

    fn bitor(mut self, rhs: Self) -> Self::Output {
        self |= rhs;
        self
    }
}

impl PlayAnytimeFlags {
    fn is_single(self) -> bool {
        use PlayAnytimeFlags::*;
        match self {
            GW | GBB | DSD => true,
            GBB_GW | DSD_GW | DSD_GBB | DSD_GBB_GW => false,
        }
    }
}

type AlmostCompleteBasket = [CardIndex; (Suit::COUNT - 1) as _];

type AlmostCompleteBaskets = [Option<AlmostCompleteBasket>; Rank::COUNT as _];

type RankCount = u8;

fn almost_complete_basket_count(baskets: AlmostCompleteBaskets) -> RankCount {
    let mut count = 0;

    for op in baskets {
        if op.is_some() {
            count += 1;
        }
    }

    count
}

#[derive(Clone, Copy, Debug)]
pub struct AvailablePlayAnytime {
    flags: PlayAnytimeFlags,
    warden_i: CardIndex,
    boat_i: CardIndex,
    scuba_i: CardIndex,
    almost_complete_baskets: AlmostCompleteBaskets,
}

impl AvailablePlayAnytime {
    fn game_warden(warden_i: CardIndex) -> Self {
        AvailablePlayAnytime{
            flags: PlayAnytimeFlags::GW,
            warden_i,
            boat_i: CardIndex::default(),
            scuba_i: CardIndex::default(),
            almost_complete_baskets: [None; Rank::COUNT as _],
        }
    }

    fn glass_bottom_boat(boat_i: CardIndex) -> Self {
        AvailablePlayAnytime{
            flags: PlayAnytimeFlags::GBB,
            warden_i: CardIndex::default(),
            boat_i,
            scuba_i: CardIndex::default(),
            almost_complete_baskets: [None; Rank::COUNT as _],
        }
    }

    fn dead_scuba_diver(
        scuba_i: CardIndex,
        almost_complete_baskets: AlmostCompleteBaskets,
    ) -> Self {
        AvailablePlayAnytime{
            flags: PlayAnytimeFlags::DSD,
            warden_i: CardIndex::default(),
            boat_i: CardIndex::default(),
            scuba_i,
            almost_complete_baskets,
        }
    }

    fn in_hand(hand: &Hand) -> Option<AvailablePlayAnytime> {
        let mut output = None;

        for (i, possible_zinger_card) in hand.enumerated_iter() {
            if possible_zinger_card == zingers::THE_GAME_WARDEN {
                match output {
                    None => {
                        output = Some(AvailablePlayAnytime::game_warden(i));
                    },
                    Some(ref mut apa) => {
                        apa.flags |= PlayAnytimeFlags::GW;
                        apa.warden_i = i;
                    },
                }
            }

            if possible_zinger_card == zingers::GLASS_BOTTOM_BOAT {
                match output {
                    None => {
                        output = Some(AvailablePlayAnytime::glass_bottom_boat(i));
                    },
                    Some(ref mut apa) => {
                        apa.flags |= PlayAnytimeFlags::GBB;
                        apa.boat_i = i;
                    },
                }
            }

            if possible_zinger_card == zingers::DEAD_SCUBA_DIVER {
                if let Some(almost_complete) = find_almost_complete_baskets(hand) {
                    match output {
                        None => {
                            output = Some(
                                AvailablePlayAnytime::dead_scuba_diver(
                                    i,
                                    almost_complete
                                )
                            );
                        },
                        Some(ref mut apa) => {
                            apa.flags |= PlayAnytimeFlags::DSD;
                            apa.scuba_i = i;
                            apa.almost_complete_baskets = almost_complete;
                        },
                    }
                }
            }
        }

        output
    }

    fn basket_count(&self) -> u8 {
        debug_assert!(self.almost_complete_baskets.len() <= u8::MAX as usize);

        self
            .almost_complete_baskets
            .into_iter()
            .filter(Option::is_some)
            .count() as u8
    }
}

fn find_almost_complete_baskets(
    hand: &Hand,
) -> Option<AlmostCompleteBaskets> {
    // TODO? better name?
    let mut scratch = [
        [None; (Suit::COUNT - 1) as _];
        Rank::COUNT as _
    ];
    for (card_i, card) in hand.enumerated_iter() {
        if let Some(rank) = get_rank(card) {
            let pile = &mut scratch[rank as u8 as usize];

            let mut found_slot = false;
            for pile_i in 0..pile.len() {
                if pile[pile_i].is_some() {
                    continue;
                }

                pile[pile_i] = Some(card_i);

                found_slot = true;
                break;
            }
            debug_assert!(
                found_slot,
                "Seems like a basket was hanging around in someone's hand"
            );
        }
    }

    let mut almost_complete = [None; Rank::COUNT as _];
    for (i, pile) in scratch.iter().enumerate() {
        match pile {
            [Some(a), Some(b), Some(c), Some(d)] => {
                almost_complete[i] = Some([*a, *b, *c, *d]);
            },
            _ => {}
        }
    }

    let mut found_any = false;
    for entry in almost_complete {
        if entry.is_some() {
            found_any = true;
            break
        }
    }

    found_any.then_some(almost_complete)
}

#[test]
fn find_almost_complete_baskets_works_on_this_previously_panicking_example() {
    let mut hand = Hand::default();
    hand.push(65);
    hand.push(17);
    hand.push(67);
    hand.push(27);
    hand.push(fish_card(Rank::Dogfish, Suit::Yellow));
    hand.push(fish_card(Rank::Dogfish, Suit::Purple));
    hand.push(42);
    hand.push(30);

    assert_eq!(
        find_almost_complete_baskets(&hand),
        None,
    );
}

#[test]
fn find_almost_complete_baskets_works_on_this_simplifed_previously_panicking_example() {
    let mut hand = Hand::default();

    hand.push(fish_card(Rank::Dogfish, Suit::Yellow));
    hand.push(fish_card(Rank::Dogfish, Suit::Purple));

    assert_eq!(
        find_almost_complete_baskets(&hand),
        None,
    );
}

#[test]
fn find_almost_complete_baskets_returns_none_on_this_previously_misbehaving_example() {
    let mut hand = Hand::default();
    hand.push(fish_card(Rank::Dogfish, Suit::Yellow));

    assert_eq!(
        find_almost_complete_baskets(&hand),
        None,
    );
}

#[test]
fn find_almost_complete_baskets_works_on_this_smaller_example() {
    let mut hand = Hand::default();
    hand.push(65);
    hand.push(17);
    hand.push(67);
    hand.push(27);

    assert_eq!(
        find_almost_complete_baskets(&hand),
        None,
    );
}

#[derive(Copy, Clone, Default, PartialEq, Eq)]
enum AnytimeCard {
    #[default]
    GameWarden,
    GlassBottomBoat,
    DeadScubaDiver,
}

impl AnytimeCard {
    fn clamp_to(&mut self, flags: PlayAnytimeFlags) {
        use PlayAnytimeFlags::*;

        *self = match (*self, flags) {
            (AnytimeCard::GameWarden, GW | GBB_GW | DSD_GW | DSD_GBB_GW) => AnytimeCard::GameWarden,
            (AnytimeCard::GameWarden, GBB | DSD_GBB) => AnytimeCard::GlassBottomBoat,
            (AnytimeCard::GameWarden, DSD) => AnytimeCard::DeadScubaDiver,
            (AnytimeCard::GlassBottomBoat, GBB | GBB_GW | DSD_GBB | DSD_GBB_GW) => AnytimeCard::GlassBottomBoat,
            (AnytimeCard::GlassBottomBoat, DSD | DSD_GW) => AnytimeCard::DeadScubaDiver,
            (AnytimeCard::GlassBottomBoat, GW) => AnytimeCard::GameWarden,
            (AnytimeCard::DeadScubaDiver, DSD | DSD_GW | DSD_GBB | DSD_GBB_GW) => AnytimeCard::DeadScubaDiver,
            (AnytimeCard::DeadScubaDiver, GW | GBB_GW) => AnytimeCard::GameWarden,
            (AnytimeCard::DeadScubaDiver, GBB) => AnytimeCard::GlassBottomBoat,
        }
    }

    fn wrapping_inc(self, flags: PlayAnytimeFlags) -> Self {
        use PlayAnytimeFlags::*;

        match (self, flags) {
            (AnytimeCard::GameWarden, GBB | GBB_GW | DSD_GBB | DSD_GBB_GW) => AnytimeCard::GlassBottomBoat,
            (AnytimeCard::GameWarden, DSD | DSD_GW) => AnytimeCard::DeadScubaDiver,
            (AnytimeCard::GameWarden, GW) => AnytimeCard::GameWarden,
            (AnytimeCard::GlassBottomBoat, DSD_GBB | DSD_GBB_GW | DSD | DSD_GW) => AnytimeCard::DeadScubaDiver,
            (AnytimeCard::GlassBottomBoat, GW | GBB_GW) => AnytimeCard::GameWarden,
            (AnytimeCard::GlassBottomBoat, GBB) => AnytimeCard::GlassBottomBoat,
            (AnytimeCard::DeadScubaDiver, GW | GBB_GW | DSD_GW | DSD_GBB_GW) => AnytimeCard::GameWarden,
            (AnytimeCard::DeadScubaDiver, GBB | DSD_GBB) => AnytimeCard::GlassBottomBoat,
            (AnytimeCard::DeadScubaDiver, DSD) => AnytimeCard::DeadScubaDiver,
        }
    }

    fn wrapping_dec(self, flags: PlayAnytimeFlags) -> Self {
        use PlayAnytimeFlags::*;

        match (self, flags) {
            (AnytimeCard::GameWarden, DSD_GBB | DSD_GBB_GW | DSD | DSD_GW) => AnytimeCard::DeadScubaDiver,
            (AnytimeCard::GameWarden, GBB | GBB_GW) => AnytimeCard::GlassBottomBoat,
            (AnytimeCard::GameWarden, GW) => AnytimeCard::GameWarden,
            (AnytimeCard::GlassBottomBoat, GW | GBB_GW | DSD_GW | DSD_GBB_GW) => AnytimeCard::GameWarden,
            (AnytimeCard::GlassBottomBoat, DSD | DSD_GBB) => AnytimeCard::DeadScubaDiver,
            (AnytimeCard::GlassBottomBoat, GBB) => AnytimeCard::GlassBottomBoat,
            (AnytimeCard::DeadScubaDiver, GBB | GBB_GW | DSD_GBB | DSD_GBB_GW) => AnytimeCard::GlassBottomBoat,
            (AnytimeCard::DeadScubaDiver, GW | DSD_GW) => AnytimeCard::GameWarden,
            (AnytimeCard::DeadScubaDiver, DSD) => AnytimeCard::DeadScubaDiver,
        }
    }
}

#[derive(Copy, Clone)]
enum AnytimePlaySelection {
    // Arguably we don't actually want to be able to represent a player targeting
    // themselves. But maybe we won't make those errors in practice.
    GameWarden(HandId),
    GlassBottomBoat(HandId),
    DeadScubaDiver(AlmostCompleteBasket, CardIndex),
    DivineIntervention,
}

#[derive(Copy, Clone)]
struct AnytimePlay {
    selection: AnytimePlaySelection,
}

fn anytime_play(
    rng: &mut Xs,
    stack: &[Play],
    cards: &Cards,
    memories: &Memories,
    responder_id: CpuId,
) -> Option<AnytimePlay> {
    let hand_id = responder_id.into();
    let hand = cards.hand(hand_id);

    for (card_i, card) in hand.enumerated_iter() {
        // TODO? are the conditions for wanting to play these two cards really
        // equal? In particular I suspect that the timing for the glass bottom
        // boat could be improved, or at least more justification for making it
        // the same as the game warden could be described.
        macro_rules! play_perhaps {
            ($card: ident) => {
                let mut others = hand_id.besides();
                xs::shuffle(rng, &mut others);

                for target in others {
                    // Note: It's not fair to look at other's cards besides
                    // counting how many of them there are.
                    let len = cards.hand(target).len();
                    if len == 1 {
                        return Some(AnytimePlay {
                            selection: AnytimePlaySelection::$card(
                                target
                            ),
                        });
                    }
                }

                if let ActiveCardCount::VeryFew = cards.active_count() {
                    for target in others {
                        // Note: It's not fair to look at other's cards besides
                        // counting how many of them there are.
                        if !cards.hand(target).is_empty() {
                            return Some(AnytimePlay {
                                selection: AnytimePlaySelection::$card(
                                    target
                                ),
                            });
                        }
                    }
                }

                for target in others {
                    if let Some(_) = memories
                        .memory(responder_id)
                        .likely_to_fill_basket_soon(target) {
                        return Some(AnytimePlay {
                            selection: AnytimePlaySelection::$card(
                                target
                            ),
                        });
                    }
                }
            }
        }

        if card == zingers::THE_GAME_WARDEN {
            play_perhaps!(GameWarden);
        }

        if card == zingers::GLASS_BOTTOM_BOAT {
            play_perhaps!(GlassBottomBoat);
        }

        if card == zingers::DEAD_SCUBA_DIVER {
            if let Some(almost_complete) = find_almost_complete_baskets(hand) {
                // TODO? Think more carefully about how to make this decision?
                let count = almost_complete_basket_count(almost_complete);
                // For testing; remove later
                if count >= 1
                //if count >= 2
                || cards.active_count() == ActiveCardCount::VeryFew {
                    let mut best_basket = None;

                    // Choose the highest scoring basket
                    for basket in almost_complete.iter().rev() {
                        if let Some(basket) = basket {
                            best_basket = Some(*basket);
                            break
                        }
                    }

                    return Some(AnytimePlay {
                        selection: AnytimePlaySelection::DeadScubaDiver(
                            best_basket.expect("There should be a basket available!"),
                            card_i
                        ),
                    });
                }
            }
        }

        if card == zingers::DIVINE_INTERVENTION {
            match stack.last() {
                Some(Play { kind: PlayKind::FishedUnsuccessfully { .. }, .. })
                | None => {
                    // Nothing to respond to, and since it is not the start of the
                    // turn we cannot discard it.
                }
                Some(Play {
                    kind: PlayKind::NoFishing {
                        targeting: Targeting { target, source },
                        ..
                    },
                    ..
                }) => {
                    if *source == hand_id {
                        // Don't cancel our own play.
                    } else if *target == hand_id {
                        // Targeting me, so they almost certainly have something
                        // I want.
                        // TODO random chance to skip to reduce exploitabilty?
                        return Some(AnytimePlay {
                            selection: AnytimePlaySelection::DivineIntervention,
                        });
                    } else {
                        if should_shed_zingers(
                            &cards,
                            &hand,
                            &stack
                        ) {
                            // It's probably time to use this up. Let's avoid
                            // needing to skip our own turn.
                            return Some(AnytimePlay {
                                selection: AnytimePlaySelection::DivineIntervention,
                            });
                        }
                    };
                }
                Some(Play {
                    kind: PlayKind::TwoFistedFisherman {
                        source,
                        ..
                    },
                    ..
                })
                | Some(Play {
                    kind: PlayKind::TheNet {
                        targeting: Targeting { source, .. },
                        ..
                    },
                    ..
                })
                /*| Some(Play {
                    kind: PlayKind::TheLure {
                        source,
                        ..
                    },
                    ..
                })*/ => {
                    if *source == hand_id {
                        // Don't cancel our own play.
                    } else if should_shed_zingers(
                        &cards,
                        &hand,
                        &stack
                    ) {
                        // It's probably time to use this up. Let's avoid
                        // needing to skip our own turn.
                        return Some(AnytimePlay {
                            selection: AnytimePlaySelection::DivineIntervention,
                        });
                    } else if memories.memory(responder_id)
                        .likely_to_fill_basket_soon(
                            *source
                        ).is_some() {
                        // We know it will likely hurt to miss this chance to ask.
                        return Some(AnytimePlay {
                            selection: AnytimePlaySelection::DivineIntervention,
                        });
                    };
                },
            }

        }
    }


    None
}

fn should_play_super_ask(
    cards: &Cards,
    hand: &Hand,
    stack: &[Play],
    memories: &Memories,
    own_id: CpuId,
) -> bool {
    should_shed_zingers(
        cards,
        hand,
        stack
    ) || {
        let hand_id = HandId::from(own_id);
        memories
            .memory(own_id)
            .likely_to_fill_basket_soon(hand_id)
            .is_some()
    }
}

fn should_shed_zingers(
    cards: &Cards,
    hand: &Hand,
    stack: &[Play]
) -> bool {
    let mut zingers_in_hand = 0;
    for card in hand.iter() {
        if models::get_zinger(card).is_some() {
            zingers_in_hand += 1;
        }
    }

    let mut zingers_in_stack = 0;
    for play in stack.iter() {
        if play.is_zinger() {
            zingers_in_stack += 1;
        }
    }

    let zingers_remaining =
        models::ZINGER_COUNT
            .saturating_sub(
                zingers_in_hand
                // Only zingers end up in the discard pile
                + cards.discard.len()
                + zingers_in_stack
            );

    zingers_remaining <= 1
}

fn discard_game_warden(
    cards: &mut Cards,
    animations: &mut Animations,
    source: HandId,
) {
    discard_given_card(
        cards,
        animations,
        source,
        zingers::THE_GAME_WARDEN,
        AfterDiscard::Nothing,
    )
}

fn discard_glass_bottom_boat(
    cards: &mut Cards,
    animations: &mut Animations,
    source: HandId,
) {
    discard_given_card(
        cards,
        animations,
        source,
        zingers::GLASS_BOTTOM_BOAT,
        AfterDiscard::Nothing,
    )
}

/// This is currently called both when using divine intervention and when doing the
/// speical discard action that doesn't affect the stack.
fn discard_divine_intervention(
    cards: &mut Cards,
    animations: &mut Animations,
    source: HandId,
) {
    discard_given_card(
        cards,
        animations,
        source,
        zingers::DIVINE_INTERVENTION,
        AfterDiscard::Nothing,
    )
}

fn discard_no_fishing(
    cards: &mut Cards,
    animations: &mut Animations,
    targeting: Targeting,
    predicate: Predicate
) {
    discard_given_card(
        cards,
        animations,
        targeting.source,
        zingers::NO_FISHING,
        AfterDiscard::PushPlay(Play {
            sub_turn_ids: targeting.source.next_to_current(),
            sub_turn_index: 0,
            kind: PlayKind::NoFishing {
                targeting,
                predicate,
            }
        }),
    )
}

fn discard_two_fisted_fisherman(
    cards: &mut Cards,
    animations: &mut Animations,
    source: HandId,
) {
    discard_given_card(
        cards,
        animations,
        source,
        zingers::TWO_FISTED_FISHERMAN,
        AfterDiscard::BackToSelecting(source),
    )
}

fn discard_lure(
    cards: &mut Cards,
    animations: &mut Animations,
    source: HandId,
) {
    discard_given_card(
        cards,
        animations,
        source,
        zingers::THE_LURE,
        AfterDiscard::Nothing,
    )
}

fn discard_net(
    cards: &mut Cards,
    animations: &mut Animations,
    targeting: Targeting,
    predicate: NetPredicate
) {
    discard_given_card(
        cards,
        animations,
        targeting.source,
        zingers::THE_NET,
        AfterDiscard::PushPlay(Play {
            sub_turn_ids: targeting.source.next_to_current(),
            sub_turn_index: 0,
            kind: PlayKind::TheNet {
                targeting,
                predicate,
            }
        }),
    )
}

fn discard_given_card(
    cards: &mut Cards,
    animations: &mut Animations,
    source: HandId,
    card: Card,
    after_discard: AfterDiscard,
) {
    let mut remove_at = None;
    for (i, current_card) in cards.hand(source).enumerated_iter() {
        if current_card == card {
            remove_at = Some(i);
            break
        }
    }

    if let Some(i) = remove_at {
        let hand = cards.hand_mut(source);
        if let Some(card) = hand.remove(i) {
            let at = get_card_position(
                spread(source),
                hand.len(),
                i,
            );

            animations.push(Animation {
                card,
                at,
                target: DISCARD_XY,
                action: AnimationAction::AddToDiscard(after_discard),
                shown: true,
                .. <_>::default()
            });

            return
        }
    }
    debug_assert!(false, "Didn't find card {}!", card);
}

fn play_dead_scuba_diver(
    cards: &mut Cards,
    id: HandId,
    almost_basket: AlmostCompleteBasket,
    scuba_i: CardIndex,
) {
    let mut to_remove = [
        almost_basket[0],
        almost_basket[1],
        almost_basket[2],
        almost_basket[3],
        scuba_i
    ];
    to_remove.sort();

    let baskets = match id {
        HandId::Player => &mut cards.player_baskets,
        HandId::Cpu1 => &mut cards.cpu1_baskets,
        HandId::Cpu2 => &mut cards.cpu2_baskets,
        HandId::Cpu3 => &mut cards.cpu3_baskets,
    };

    let hand = match id {
        HandId::Player => &mut cards.player,
        HandId::Cpu1 => &mut cards.cpu1,
        HandId::Cpu2 => &mut cards.cpu2,
        HandId::Cpu3 => &mut cards.cpu3,
    };

    for i in to_remove.iter().rev() {
        let card = hand.remove(*i).expect("all to_remove indexes should be valid!");
        baskets.push(card);
    }
}

#[derive(Clone, Copy)]
enum AnytimeOutcome {
    Hold,
    Done
}

fn do_play_anytime_menu(
    mut group: &mut ui::Group,
    cards: &mut Cards,
    animations: &mut Animations,
    rng: &mut Xs,
    player_selection: &mut PlayerSelection,
    available: AvailablePlayAnytime,
) -> AnytimeOutcome {
    use AnytimeOutcome::*;

    const GRID_LEN: usize = 3;

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum Section {
        Card,
        Target,
        Submit,
    }

    const GRID: [Section; GRID_LEN] = [
        Section::Card, Section::Target, Section::Submit,
    ];

    // Clamp things to initial good vales, from the defaults.
    {
        player_selection.card.clamp_to(available.flags);

        if available.almost_complete_baskets[(player_selection.rank as u8) as usize]
            .is_none() {
            for (i, entry) in available.almost_complete_baskets.iter().enumerate() {
                if entry.is_some() {
                    player_selection.rank = Rank::ALL[i];
                    break
                }
            }
        }

        let old_el = match group.ctx.hot {
            AnytimeCard => Some(Section::Card),
            CpuIdSelect
            | RankSelect => Some(Section::Target),
            Submit => Some(Section::Submit),
            Zero
            | AskSuit
            | LurePredicate
            | NetPredicate => None,
        };

        let mut el_i = GRID.iter()
            .position(|el| Some(*el) == old_el)
            .unwrap_or_default();

        // Place it on the first active position if it is not on an active position.
        if el_i == 0 && available.flags.is_single() {
            el_i = 1;
        }
        if player_selection.card == AnytimeCard::DeadScubaDiver
        && el_i == 1
        && available.basket_count() <= 1 {
            el_i = GRID_LEN - 1;
        }

        group.ctx.set_next_hot(match GRID[el_i] {
            Section::Card => AnytimeCard,
            Section::Target => match player_selection.card {
                AnytimeCard::GameWarden
                | AnytimeCard::GlassBottomBoat => CpuIdSelect,
                AnytimeCard::DeadScubaDiver => RankSelect,
            },
            Section::Submit => Submit,
        });
    }


    if let Some(card) = player_selection.viewing {
        group.commands.draw_nine_slice(
            gfx::NineSlice::Window,
            CARD_VIEWING_WINDOW
        );

        let base_xy = CARD_VIEWING_WINDOW.xy()
            + WINDOW_CONTENT_OFFSET;

        let text_xy = base_xy;

        group.commands.print_centered(
            CpuId::HAS_TEXT[player_selection.target as u8 as usize],
            Rect::xy_wh(
                text_xy,
                CARD_VIEWING_TEXT_WH,
            ),
            WHITE,
        );

        let viewing_card_xy = base_xy + CARD_VIEWING_TEXT_WH.h;

        group.commands.draw_card(
            card,
            viewing_card_xy,
        );

        let submit_base_xy = base_xy + CARD_WIDTH;

        group.ctx.set_next_hot(Submit);

        return if do_button(
            &mut group,
            ButtonSpec {
                id: Submit,
                rect: fit_to_rest_of_window(
                    submit_base_xy,
                    CARD_VIEWING_WINDOW,
                ),
                text: b"Done",
            }
        ) {
            player_selection.viewing = None;
            Done
        } else {
            Hold
        };
    }

    let base_xy = PLAYER_PLAY_ANYTIME_WINDOW.xy()
        + WINDOW_CONTENT_OFFSET;

    let card_quick_select_xy = base_xy
        - WINDOW_CONTENT_OFFSET.h
        + ((PLAYER_PLAY_ANYTIME_WINDOW.h - CARD_QUICK_SELECT_WH.h)/ 2);

    let card_xy = card_quick_select_xy + CHEVRON_H;


    group.commands.draw_nine_slice(
        gfx::NineSlice::Window,
        PLAYER_PLAY_ANYTIME_WINDOW
    );

    match available.flags {
        PlayAnytimeFlags::GW => {
            player_selection.card = AnytimeCard::GameWarden;
            group.commands.draw_card(
                zingers::THE_GAME_WARDEN,
                card_xy,
            );
        },
        PlayAnytimeFlags::GBB => {
            player_selection.card = AnytimeCard::GlassBottomBoat;
            group.commands.draw_card(
                zingers::GLASS_BOTTOM_BOAT,
                card_xy,
            );
        },
        PlayAnytimeFlags::DSD => {
            player_selection.card = AnytimeCard::DeadScubaDiver;
            group.commands.draw_card(
                zingers::DEAD_SCUBA_DIVER,
                card_xy,
            );
        }
        _multiple => {
            match player_selection.card {
                AnytimeCard::GameWarden => {
                    group.commands.draw_card(
                        zingers::THE_GAME_WARDEN,
                        card_xy,
                    );
                },
                AnytimeCard::GlassBottomBoat => {
                    group.commands.draw_card(
                        zingers::GLASS_BOTTOM_BOAT,
                        card_xy,
                    );
                },
                AnytimeCard::DeadScubaDiver => {
                    group.commands.draw_card(
                        zingers::DEAD_SCUBA_DIVER,
                        card_xy,
                    );
                },
            }

            ui::draw_quick_select(
                group,
                Rect::xy_wh(
                    card_quick_select_xy,
                    CARD_QUICK_SELECT_WH,
                ),
                AnytimeCard
            );
        }
    }

    let mut almost_basket_option: Option<AlmostCompleteBasket> = None;
    let submit_base_xy;
    match player_selection.card {
        AnytimeCard::GameWarden
        | AnytimeCard::GlassBottomBoat => {
            let target_xy = base_xy + CARD_WIDTH
                + ((PLAYER_PLAY_ANYTIME_WINDOW.h - CPU_ID_SELECT_WH.h)/ 2);

            draw_cpu_id_quick_select(
                group,
                player_selection.target,
                target_xy,
            );

            submit_base_xy = base_xy + CARD_WIDTH + CPU_ID_SELECT_WH.w;
        },
        AnytimeCard::DeadScubaDiver => {
            let rank_xy = base_xy + CARD_WIDTH
                + ((PLAYER_PLAY_ANYTIME_WINDOW.h - RANK_SELECT_WH.h)/ 2);

            let mut suit_xy = (rank_xy + (RANK_SELECT_WH.w / 2)) - (CARD_WIDTH / 2) + gfx::CHEVRON_H;
            for &suit in Suit::ALL.iter().rev() {
                group.commands.draw_card(
                    fish_card(player_selection.rank, suit),
                    suit_xy
                );

                suit_xy += RANK_SELECT_SPREAD_WH;
            }

            if available.basket_count() > 1 {
                ui::draw_quick_select(
                    group,
                    Rect::xy_wh(
                        rank_xy,
                        RANK_SELECT_WH,
                    ),
                    RankSelect
                );
            }

            almost_basket_option = available.almost_complete_baskets[
                (player_selection.rank as u8) as usize
            ];

            submit_base_xy = base_xy + CARD_WIDTH + RANK_SELECT_WH.w;
        }
    }

    let target = player_selection.target.into();

    // TODO? show a "hand is empty" message?
    if !cards.hand(target).is_empty()
    && (
        // if the player selected DeadScubaDiver, then the almost_basket_option must
        // be `Some`, but otherwise that is not required.
        player_selection.card != AnytimeCard::DeadScubaDiver
        || almost_basket_option.is_some()
    )
    && do_button(
        &mut group,
        ButtonSpec {
            id: Submit,
            rect: fit_to_rest_of_window(
                submit_base_xy,
                PLAYER_PLAY_ANYTIME_WINDOW,
            ),
            text: b"Submit",
        }
    ) {
        match player_selection.card {
            AnytimeCard::GameWarden => {
                if let Some(()) = perform_game_warden(
                    cards,
                    animations,
                    rng,
                    Targeting {
                        source: HandId::Player,
                        target: player_selection.target.into(),
                    },
                ) {
                    return Done;
                } else {
                    debug_assert!(false, "perform_game_warden failed");
                }
            },
            AnytimeCard::GlassBottomBoat => {
                let target_hand = cards.hand_mut(target);
                let i = xs::range(rng, 0..(target_hand.len() as u32)) as _;
                let card = target_hand.remove(i).expect("hand should have already been checked to see if it was not empty!");

                let at = get_card_position(
                    spread(target),
                    target_hand.len(),
                    i,
                );

                // The card in the animation is the one that will be returned to the
                // Cpu player's hand. This is a separate copy.
                player_selection.viewing = Some(card);

                animations.push(Animation {
                    card,
                    at,
                    target: in_front_of(HandId::Player),
                    action: AnimationAction::AnimateBackToHand(target),
                    .. <_>::default()
                });

                discard_glass_bottom_boat(
                    cards,
                    animations,
                    HandId::Player,
                );
                return Hold;
            },
            AnytimeCard::DeadScubaDiver => {
                let almost_basket: AlmostCompleteBasket
                    = almost_basket_option.expect("almost_basket_option should have already been checked");
                play_dead_scuba_diver(
                    cards,
                    HandId::Player,
                    almost_basket,
                    available.scuba_i
                );
                return Done;
            }
        }
    } else if group.input.pressed_this_frame(Button::B) {
        // TODO? Separate decline button?
        player_selection.declined = true;
    } else if let Some(dir) = group.input.dir_pressed_this_frame() {
        let old_el = match group.ctx.hot {
            AnytimeCard => Some(Section::Card),
            CpuIdSelect
            | RankSelect => Some(Section::Target),
            ui::Id::Submit => Some(Section::Submit),
            Zero
            | AskSuit
            | LurePredicate
            | NetPredicate => None,
        };

        let mut el_i = GRID.iter()
            .position(|el| Some(*el) == old_el)
            .unwrap_or_default();

        match dir {
            Dir::Up => match GRID[el_i] {
                Section::Card => {
                    player_selection.card
                        = player_selection.card.wrapping_inc(available.flags);
                },
                Section::Target => {
                    match player_selection.card {
                        AnytimeCard::GameWarden
                        | AnytimeCard::GlassBottomBoat => {
                            player_selection.target
                                = player_selection.target.wrapping_inc();
                        },
                        AnytimeCard::DeadScubaDiver => {
                            player_selection.rank
                                = player_selection.rank.wrapping_inc(
                                    available.almost_complete_baskets
                                );
                        },
                    }
                },
                Section::Submit => {}
            },
            Dir::Down => match GRID[el_i] {
                Section::Card => {
                    player_selection.card
                        = player_selection.card.wrapping_dec(available.flags);
                },
                Section::Target => {
                    match player_selection.card {
                        AnytimeCard::GameWarden
                        | AnytimeCard::GlassBottomBoat => {
                            player_selection.target
                                = player_selection.target.wrapping_dec();
                        },
                        AnytimeCard::DeadScubaDiver => {
                            player_selection.rank
                                = player_selection.rank.wrapping_dec(
                                    available.almost_complete_baskets
                                );
                        },
                    }
                },
                Section::Submit => {}
            },
            Dir::Left => if el_i == 0 {
                el_i = GRID_LEN - 1;
            } else {
                el_i -= 1;

                if player_selection.card == AnytimeCard::DeadScubaDiver
                && el_i == 1
                && available.basket_count() <= 1 {
                    el_i = 0;
                }

                // Don't need to select the card if only one is available.
                if el_i == 0 && available.flags.is_single() {
                    el_i = GRID_LEN - 1;
                }
            },
            Dir::Right => {
                if el_i >= GRID_LEN - 1 {
                    // Don't need to select the card if only one is available.
                    if available.flags.is_single() {
                        el_i = 1;
                    } else {
                        el_i = 0;
                    }
                } else {
                    el_i += 1;
                }

                if player_selection.card == AnytimeCard::DeadScubaDiver
                && el_i == 1
                && available.basket_count() <= 1 {
                    el_i = GRID_LEN - 1;
                }
            },
        }

        group.ctx.set_next_hot(match GRID[el_i] {
            Section::Card => AnytimeCard,
            Section::Target => match player_selection.card {
                AnytimeCard::GameWarden
                | AnytimeCard::GlassBottomBoat => CpuIdSelect,
                AnytimeCard::DeadScubaDiver => RankSelect,
            },
            Section::Submit => Submit,
        });
    } else {
        // do nothing
    }

    Hold
}

fn should_use_no_fishing_against(
    memory: &memories::Memory,
    hand: &Hand,
    target: HandId,
    predicate: Predicate,
    active_count: ActiveCardCount,
) -> bool {
    if let ActiveCardCount::VeryFew = active_count {
        return true
    }

    match predicate {
        Predicate::RankSuit(rank, suit) => {
            hand.contains(fish_card(rank, suit))
            && memory.is_likely_to_fill_rank_soon(target, rank)
        },
        Predicate::Net(_predicate) => {
            todo!("should_use_no_fishing_against Net(_predicate)");
        },
    }
}

fn can_and_should_play_two_fisted_fisherman(
    memories: &memories::Memories,
    hand: &Hand,
    source: CpuId,
    active_count: ActiveCardCount,
) -> bool {
    if !hand.contains(zingers::TWO_FISTED_FISHERMAN) {
        return false
    }

    if let ActiveCardCount::VeryFew = active_count {
        return true
    }

    let targets = HandId::from(source).besides();

    // Note: It is not fair to read other players' memories.
    let memory = memories.memory(source);

    // TODO? Does this capture the criteria we want to capture?
    for rank in Rank::ALL {
        for suit in Suit::ALL {
            if hand.contains(fish_card(rank, suit)) {
                for target in targets {
                    if memory.is_likely_to_fill_rank_soon(target, rank) {
                        return true
                    }
                }
            }
        }
    }

    return true
}

pub fn update_and_render(
    commands: &mut Commands,
    state: &mut State,
    input: Input,
    speaker: &mut Speaker
) {
    macro_rules! new_group {
        () => {
            &mut ui::Group {
                commands,
                ctx: &mut state.ctx,
                input,
                speaker,
            }
        }
    }

    state.ctx.frame_init();

    state.tick(speaker);

    if !state.cards.deck.is_empty() {
        commands.draw_card_back(DECK_XY);
    }

    if let Some(last) = state.cards.discard.last() {
        commands.draw_card(last, DISCARD_XY);
    }

    for anim in state.animations.iter() {
        if anim.is_active() {
            if anim.shown {
                commands.draw_card(anim.card, anim.at);
            } else {
                commands.draw_card_back(anim.at);
            }
        }
    }

    for id in HandId::CPUS.into_iter() {
        let hand = state.cards.hand(id);
        let len = hand.len();

        if cfg!(debug_assertions) {
            for (i, card) in hand.enumerated_iter() {
                commands.draw_card(
                    card,
                    get_card_position(spread(id), len, i)
                );
            }
            continue
        }

        for i in 0..len {
            commands.draw_card_back(
                get_card_position(spread(id), len, i)
            );
        }
    }

    'player_hand: {
        let id = HandId::Player;
        let hand = state.cards.hand(id);
        let len = hand.len();

        if len == 0 {
            break 'player_hand
        }

        for (i, card) in hand.enumerated_iter() {
            if state.selection.card_index == i { continue }

            commands.draw_card(
                card,
                get_card_position(spread(id), len, i)
            );
        }

        if let Some(player_card) = hand.get(state.selection.card_index)
        {
            let selected_pos = get_card_position(
                spread(id),
                len,
                state.selection.card_index
            );

            commands.draw_card(
                player_card,
                selected_pos
            );

            commands.draw_selectrum(selected_pos);
        }
    }

    if state.animations.all_done() {
        // TODO Look at sub_turn_ids on top of the stack if there are any. See note below.
        match {
            match state.stack.last() {
                Some(Play {sub_turn_ids, sub_turn_index, ..}) => {
                    sub_turn_ids.get(*sub_turn_index as usize)
                },
                None => state.sub_turn_ids.get(state.sub_turn_index as usize),
            }
        } {
            Some(&responder_id) => {
                // Give this participant a chance to respond.
                enum Selection {
                    Response(()),
                    Nothing,
                    Pending,
                }

                let hand = state.cards.hand(responder_id);

                let selection = if hand.is_empty() {
                    Selection::Nothing
                } else {
                    match CpuId::try_from(responder_id) {
                        Err(()) => {
                            if let (Some(available), false) = (
                                AvailablePlayAnytime::in_hand(state.cards.hand(HandId::Player)),
                                state.selection.player_selection.declined
                            ) {
                                match do_play_anytime_menu(
                                    new_group!(),
                                    &mut state.cards,
                                    &mut state.animations,
                                    &mut state.rng,
                                    &mut state.selection.player_selection,
                                    available,
                                ) {
                                    AnytimeOutcome::Done => Selection::Response(()),
                                    AnytimeOutcome::Hold => Selection::Pending,
                                }
                            } else {
                                Selection::Nothing
                            }
                        }
                        Ok(source) => {
                            if let Some(AnytimePlay { selection })
                            = anytime_play(
                                &mut state.rng,
                                &state.stack,
                                &state.cards,
                                &state.memories,
                                source
                            ) {
                                match selection {
                                    AnytimePlaySelection::GameWarden(target) => {
                                        if let Some(()) = perform_game_warden(
                                            &mut state.cards,
                                            &mut state.animations,
                                            &mut state.rng,
                                            Targeting {
                                                source: responder_id,
                                                target,
                                            },
                                        ) {
                                            Selection::Response(())
                                        } else {
                                            debug_assert!(false, "perform_game_warden failed");
                                            Selection::Nothing
                                        }
                                    },
                                    AnytimePlaySelection::GlassBottomBoat(target) => {
                                        let target_hand = state.cards.hand_mut(target);
                                        let i = xs::range(&mut state.rng, 0..(target_hand.len() as u32)) as _;
                                        let card = target_hand.remove(i).expect("hand should have already been checked to see if it was not empty!");

                                        state.memories.memory_mut(source).known(target, card);

                                        let at = get_card_position(
                                            spread(target),
                                            target_hand.len(),
                                            i,
                                        );

                                        state.animations.push(Animation {
                                            card,
                                            at,
                                            target: in_front_of(responder_id),
                                            action: AnimationAction::AnimateBackToHand(target),
                                            .. <_>::default()
                                        });

                                        discard_glass_bottom_boat(
                                            &mut state.cards,
                                            &mut state.animations,
                                            source.into()
                                        );

                                        Selection::Response(())
                                    },
                                    AnytimePlaySelection::DeadScubaDiver(almost_basket, scuba_i) => {
                                        play_dead_scuba_diver(
                                            &mut state.cards,
                                            responder_id,
                                            almost_basket,
                                            scuba_i
                                        );
                                        Selection::Response(())
                                    },
                                    AnytimePlaySelection::DivineIntervention => {
                                        discard_divine_intervention(
                                            &mut state.cards,
                                            &mut state.animations,
                                            source.into()
                                        );

                                        if let Some(Play {
                                            kind: PlayKind::TwoFistedFisherman {
                                                cancelled,
                                                ..
                                            },
                                            ..
                                        }) = state.stack.last_mut() {
                                            *cancelled = true;
                                        } else {
                                            // Cancel the card we targetted by
                                            // removing it from the stack.
                                            state.stack.pop();
                                        }

                                        Selection::Response(())
                                    }
                                }
                            } else {
                                Selection::Nothing
                            }
                        }
                    }
                };

                match selection {
                    Selection::Response(()) => {
                        // TODO trigger an animation that will get us out of this state
                    },
                    Selection::Nothing => {
                        // Passing the chance to counter
                        match state.stack.last_mut() {
                            Some(Play { sub_turn_index, ..}) => {
                                *sub_turn_index += 1;
                            },
                            None => {
                                state.sub_turn_index += 1;
                            },
                        }
                    },
                    Selection::Pending => {
                        assert_eq!(responder_id, HandId::Player);
                        // Keep drawing the menu for the player until they choose
                    }
                }
            }
            None => {
                match state.stack.pop() {
                    None => {
                        // Since the stack is empty, the turn_id player gets to play
                        match CpuId::try_from(state.turn_id) {
                            Err(()) => {
                                let selected = state.selection.card_index;
                                let menu = &mut state.selection.player_menu;
                                let hand = &state.cards.player;
                                let len = hand.len();

                                if let Some(player_card) = hand.get(selected) {
                                    state.has_started = true;
                                    let id = HandId::Player;
                                    match menu {
                                        PlayerMenu::Selecting {
                                            ref mut sub_menu,
                                        } => {
                                            let selected_pos = get_card_position(
                                                spread(id),
                                                len,
                                                selected
                                            );

                                            commands.draw_card(
                                                player_card,
                                                selected_pos
                                            );

                                            commands.draw_selectrum(selected_pos);

                                            match sub_menu {
                                                PlayerSelectingSubMenu::Root => {
                                                    if input.pressed_this_frame(Button::LEFT) {
                                                        state.selection.card_index = if selected > 0 {
                                                            selected - 1
                                                        } else {
                                                            state.cards.player.len().saturating_sub(1)
                                                        };
                                                    } else if input.pressed_this_frame(Button::RIGHT) {
                                                        state.selection.card_index = if selected < state.cards.player.len().saturating_sub(1) {
                                                            selected + 1
                                                        } else {
                                                            0
                                                        };
                                                    } else if input.pressed_this_frame(Button::A) {
                                                        if !state.cards.player.is_empty() {
                                                            let player_card = state.cards.player.get(selected)
                                                                .expect("selected index should always be valid");
                                                            if let Some(zinger) = models::get_zinger(player_card) {
                                                                match zinger {
                                                                    Zinger::TheGameWarden => {
                                                                        *sub_menu = PlayerSelectingSubMenu::Anytime(
                                                                            <_>::default(),
                                                                            AvailablePlayAnytime::game_warden(selected),
                                                                        );
                                                                    },
                                                                    Zinger::GlassBottomBoat => {
                                                                        *sub_menu = PlayerSelectingSubMenu::Anytime(
                                                                            <_>::default(),
                                                                            AvailablePlayAnytime::glass_bottom_boat(selected),
                                                                        );
                                                                    },
                                                                    Zinger::DeadScubaDiver => {
                                                                        if let Some(almost_complete_baskets) = find_almost_complete_baskets(
                                                                            &state.cards.player
                                                                        ) {
                                                                            *sub_menu = PlayerSelectingSubMenu::Anytime(
                                                                                <_>::default(),
                                                                                AvailablePlayAnytime::dead_scuba_diver(
                                                                                    selected,
                                                                                    almost_complete_baskets
                                                                                ),
                                                                            );
                                                                        } else {
                                                                            let message = b"There are no almost-full baskets in your hand.";
                                                                            let mut vec = Vec::with_capacity(message.len());
                                                                            vec.extend(message);
                                                                            *sub_menu = PlayerSelectingSubMenu::Message(vec);
                                                                        }
                                                                    },
                                                                    Zinger::NoFishing => {
                                                                        let message = b"This card can only be played when someone asks you for a fish.";
                                                                        let mut vec = Vec::with_capacity(message.len());
                                                                        vec.extend(message);
                                                                        *sub_menu = PlayerSelectingSubMenu::Message(vec);
                                                                    },
                                                                    Zinger::TwoFistedFisherman => {
                                                                        let message = b"This card can only be played when you have asked someone for a fish and received a negative response.";
                                                                        let mut vec = Vec::with_capacity(message.len());
                                                                        vec.extend(message);
                                                                        *sub_menu = PlayerSelectingSubMenu::Message(vec);
                                                                    },
                                                                    Zinger::TheNet => if state.done_something_this_turn {
                                                                        let message = b"This card can only be played at the start of your turn.";
                                                                        let mut vec = Vec::with_capacity(message.len());
                                                                        vec.extend(message);
                                                                        *sub_menu = PlayerSelectingSubMenu::Message(vec);
                                                                    } else {
                                                                        state.selection.card_index = selected;
                                                                        state.selection.player_menu = PlayerMenu::Net {
                                                                            target: <_>::default(),
                                                                            predicate: <_>::default(),
                                                                        };
                                                                    },
                                                                    Zinger::TheLure => if state.done_something_this_turn {
                                                                        let message = b"This card can only be played at the start of your turn.";
                                                                        let mut vec = Vec::with_capacity(message.len());
                                                                        vec.extend(message);
                                                                        *sub_menu = PlayerSelectingSubMenu::Message(vec);
                                                                    } else {
                                                                        state.selection.card_index = selected;
                                                                        state.selection.player_menu = PlayerMenu::Lure {
                                                                            target: <_>::default(),
                                                                            predicate: <_>::default(),
                                                                        };
                                                                    },
                                                                    Zinger::DivineIntervention => {
                                                                        let played_zinger_count = state.cards.played_zinger_count();

                                                                        if played_zinger_count >= 7 {
                                                                            let message = b"All of the other zingers have been played. You missed your chance to discard this!";
                                                                            let mut vec = Vec::with_capacity(message.len());
                                                                            vec.extend(message);
                                                                            *sub_menu = PlayerSelectingSubMenu::Message(vec);
                                                                        } else {
                                                                            todo!("Zinger::DivineIntervention")
                                                                        }
                                                                    },
                                                                }
                                                            } else {
                                                                state.selection.card_index = selected;
                                                                state.selection.player_menu = PlayerMenu::Asking{
                                                                    used: player_card,
                                                                    question: Default::default(),
                                                                    sub_menu: Default::default(),
                                                                };
                                                                state.ctx.set_next_hot(CpuIdSelect);
                                                            }
                                                        }
                                                    } else {
                                                        // do nothing
                                                    }
                                                },
                                                PlayerSelectingSubMenu::Anytime(
                                                    ref mut player_selection,
                                                    available,
                                                ) => {
                                                    if player_selection.declined {
                                                        *sub_menu = PlayerSelectingSubMenu::Root;
                                                    } else if let AnytimeOutcome::Done
                                                    = do_play_anytime_menu(
                                                        new_group!(),
                                                        &mut state.cards,
                                                        &mut state.animations,
                                                        &mut state.rng,
                                                        player_selection,
                                                        *available
                                                    ) {
                                                        // Does not count as a turn.
                                                        *sub_menu = PlayerSelectingSubMenu::Root;
                                                    }
                                                },
                                                PlayerSelectingSubMenu::Message(ref mut message) => {
                                                    commands.draw_nine_slice(
                                                        gfx::NineSlice::Window,
                                                        MESSAGE_WINDOW
                                                    );

                                                    let base_xy = MESSAGE_WINDOW.xy() + WINDOW_CONTENT_OFFSET;

                                                    let message_base_xy = base_xy;

                                                    let message_base_rect = fit_to_rest_of_window(
                                                        message_base_xy,
                                                        MESSAGE_WINDOW,
                                                    );

                                                    text::bytes_reflow_in_place(
                                                        message,
                                                        MESSAGE_WINDOW_WIDTH_IN_CHARS
                                                    );

                                                    commands.print_centered(
                                                        message,
                                                        message_base_rect,
                                                        WHITE,
                                                    );

                                                    if input.pressed_this_frame(Button::B) {
                                                        *sub_menu = PlayerSelectingSubMenu::Root;
                                                    }
                                                },
                                            }
                                        },
                                        PlayerMenu::Net {
                                            ref mut target,
                                            ref mut predicate,
                                        } => {
                                            commands.draw_nine_slice(
                                                gfx::NineSlice::Window,
                                                PLAYER_NET_WINDOW
                                            );

                                            let base_xy = PLAYER_NET_WINDOW.xy()
                                                + WINDOW_CONTENT_OFFSET;

                                            let label_card_xy = base_xy;

                                            commands.draw_card(
                                                zingers::THE_NET,
                                                label_card_xy
                                            );

                                            let target_base_xy = label_card_xy + CARD_WIDTH;
                                            let target_xy = target_base_xy + H(CARD_WIN_H / 5);

                                            let group = new_group!();

                                            draw_cpu_id_quick_select(
                                                group,
                                                *target,
                                                target_xy
                                            );

                                            let card_xy = target_base_xy + CPU_ID_SELECT_WH.w;

                                            group.commands.draw_net_predicate_card(
                                                *predicate,
                                                card_xy
                                            );

                                            let predicate_select_xy = card_xy + CARD_WIDTH;

                                            let predicate_select_rect = Rect::xy_wh(
                                                predicate_select_xy,
                                                NET_PREDICATE_SELECT_WH,
                                            );

                                            group.commands.print_centered(
                                                NetPredicate::TEXT[predicate.index_of()],
                                                Rect::xy_wh(
                                                    predicate_select_xy + NET_PREDICATE_SELECT_TEXT_OFFSET.w,
                                                    NET_PREDICATE_SELECT_TEXT_WH,
                                                ),
                                                WHITE,
                                            );

                                            ui::draw_quick_select(
                                                group,
                                                predicate_select_rect,
                                                NetPredicate,
                                            );

                                            let submit_xy = predicate_select_xy + NET_PREDICATE_SELECT_WH.w;

                                            if do_button(
                                                group,
                                                ButtonSpec {
                                                    id: Submit,
                                                    rect: fit_to_rest_of_window(
                                                        submit_xy,
                                                        PLAYER_NET_WINDOW,
                                                    ),
                                                    text: b"Submit",
                                                }
                                            ) {
                                                discard_net(
                                                    &mut state.cards,
                                                    &mut state.animations,
                                                    Targeting {
                                                        source: HandId::Player,
                                                        target: (*target).into(),
                                                    },
                                                    *predicate,
                                                );
                                            } else if input.pressed_this_frame(Button::B) {
                                                state.selection.card_index = selected;
                                                state.selection.player_menu = PlayerMenu::default();
                                            } else if let Some(dir) = input.dir_pressed_this_frame() {
                                                const GRID_LEN: usize = 3;

                                                #[derive(Clone, Copy, PartialEq, Eq)]
                                                enum Section {
                                                    Target,
                                                    Predicate,
                                                    Submit,
                                                }

                                                const GRID: [Section; GRID_LEN] = [
                                                    Section::Target, Section::Predicate, Section::Submit,
                                                ];

                                                let old_el = match state.ctx.hot {
                                                    CpuIdSelect => Some(Section::Target),
                                                    NetPredicate => Some(Section::Predicate),
                                                    Submit => Some(Section::Submit),
                                                    _ => None,
                                                };
                                                let mut el_i = GRID.iter()
                                                    .position(|el| Some(*el) == old_el)
                                                    .unwrap_or_default();

                                                match dir {
                                                    Dir::Up => match GRID[el_i] {
                                                        Section::Target => match *target {
                                                            CpuId::One => { *target = CpuId::Two; },
                                                            CpuId::Two => { *target = CpuId::Three; },
                                                            CpuId::Three => { *target = CpuId::One; },
                                                        },
                                                        Section::Predicate => predicate.wrapping_inc(),
                                                        Section::Submit => {}
                                                    },
                                                    Dir::Down => match GRID[el_i] {
                                                        Section::Target => match *target {
                                                            CpuId::One => { *target = CpuId::Three; },
                                                            CpuId::Two => { *target = CpuId::One; },
                                                            CpuId::Three => { *target = CpuId::Two; },
                                                        },
                                                        Section::Predicate => predicate.wrapping_dec(),
                                                        Section::Submit => {}
                                                    },
                                                    Dir::Left => if el_i == 0 {
                                                        el_i = GRID_LEN - 1;
                                                    } else {
                                                        el_i -= 1;
                                                    },
                                                    Dir::Right => if el_i >= GRID_LEN - 1 {
                                                        el_i = 0;
                                                    } else {
                                                        el_i += 1;
                                                    },
                                                }
                                                state.ctx.set_next_hot(match GRID[el_i] {
                                                    Section::Target => CpuIdSelect,
                                                    Section::Predicate => NetPredicate,
                                                    Section::Submit => Submit,
                                                });
                                            } else {
                                                // do nothing
                                            }
                                        },
                                        PlayerMenu::Lure {
                                            ref mut target,
                                            ref mut predicate,
                                        } => {
                                            commands.draw_nine_slice(
                                                gfx::NineSlice::Window,
                                                PLAYER_LURE_WINDOW
                                            );

                                            let base_xy = PLAYER_LURE_WINDOW.xy()
                                                + WINDOW_CONTENT_OFFSET;

                                            let label_card_xy = base_xy;

                                            commands.draw_card(
                                                zingers::THE_LURE,
                                                label_card_xy
                                            );

                                            let target_base_xy = label_card_xy + CARD_WIDTH;
                                            let target_xy = target_base_xy + H(CARD_WIN_H / 5);

                                            let group = new_group!();

                                            draw_cpu_id_quick_select(
                                                group,
                                                *target,
                                                target_xy
                                            );

                                            let card_xy = target_base_xy + CPU_ID_SELECT_WH.w;

                                            group.commands.draw_card(
                                                fish_card(predicate.rank, predicate.suit),
                                                card_xy
                                            );

                                            let predicate_select_xy = card_xy + CARD_WIDTH;

                                            let predicate_select_rect = Rect::xy_wh(
                                                predicate_select_xy,
                                                LURE_PREDICATE_SELECT_WH,
                                            );

                                            group.commands.print_centered(
                                                LurePredicate::TEXT[predicate.index_of()],
                                                Rect::xy_wh(
                                                    predicate_select_xy + LURE_PREDICATE_SELECT_TEXT_OFFSET.w,
                                                    LURE_PREDICATE_SELECT_TEXT_WH,
                                                ),
                                                WHITE,
                                            );

                                            ui::draw_quick_select(
                                                group,
                                                predicate_select_rect,
                                                LurePredicate,
                                            );

                                            let submit_xy = predicate_select_xy + LURE_PREDICATE_SELECT_WH.w;

                                            if do_button(
                                                group,
                                                ButtonSpec {
                                                    id: Submit,
                                                    rect: fit_to_rest_of_window(
                                                        submit_xy,
                                                        PLAYER_NET_WINDOW,
                                                    ),
                                                    text: b"Submit",
                                                }
                                            ) {
                                                macro_rules! p_net_handle_negative_response {
                                                    () => {
                                                        let player_len = state.cards.player.len();

                                                        let drew = state.cards.deck.draw();

                                                        if let Some(card) = drew {
                                                            let at = DECK_XY;

                                                            let target = get_card_insert_position(
                                                                spread(HandId::Player),
                                                                player_len
                                                            );

                                                            state.animations.push(Animation {
                                                                card,
                                                                at,
                                                                target,
                                                                action: AnimationAction::AddToHand(HandId::Player),
                                                                .. <_>::default()
                                                            });
                                                        }

                                                        // This card counts as a turn, so just go on to the next turn.
                                                        to_next_turn!(state);
                                                    }
                                                }

                                                discard_lure(
                                                    &mut state.cards,
                                                    &mut state.animations,
                                                    HandId::Player
                                                );

                                                let target_hand_id = (*target).into();
                                                if state.cards.hand(target_hand_id)
                                                    .contains(zingers::NO_FISHING)
                                                && should_use_no_fishing_against(
                                                    state.memories.memory(*target),
                                                    state.cards.hand(target_hand_id),
                                                    HandId::Player,
                                                    Predicate::RankSuit(predicate.rank, predicate.suit),
                                                    state.cards.active_count(),
                                                ) {
                                                    discard_no_fishing(
                                                        &mut state.cards,
                                                        &mut state.animations,
                                                        target_hand_id.with_target(HandId::Player),
                                                        Predicate::RankSuit(predicate.rank, predicate.suit)
                                                    );
                                                    p_net_handle_negative_response!();
                                                } else {
                                                    let player_len = state.cards.player.len();
                                                    let target_hand = state.cards.hand_mut(target_hand_id);

                                                    state.memories.asked_for(
                                                        HandId::Player,
                                                        Predicate::RankSuit(predicate.rank, predicate.suit)
                                                    );

                                                    let memory = state.memories.memory(*target);
                                                    let mut found = None;
                                                    let LurePredicate { rank, suit } = *predicate;
                                                    {
                                                        for (i, card) in target_hand.enumerated_iter() {
                                                            match (get_rank(card), get_suit(card)) {
                                                                (Some(r), Some(s)) if r == rank && s == suit => {
                                                                    found = Some((rank, suit, i));
                                                                    // TODO? pick best one to give up, when there's no unlikely ones left?
                                                                    // If this is an undesirable to give up card, keep looking.
                                                                    if memory.is_likely_to_fill_rank_soon(
                                                                        target_hand_id,
                                                                        rank
                                                                    ) || memory.is_likely_to_fill_rank_soon(
                                                                        HandId::Player,
                                                                        rank
                                                                    ) {
                                                                        continue
                                                                    }
                                                                    break
                                                                }
                                                                _ => {}
                                                            }
                                                        }
                                                    }

                                                    if let Some((rank, suit, i)) = found {
                                                        state.memories.found(
                                                            HandId::Player,
                                                            rank,
                                                            suit
                                                        );
                                                        let at = get_card_position(
                                                            spread(target_hand_id),
                                                            target_hand.len(),
                                                            i,
                                                        );
                                                        let removed = target_hand.remove(i);
                                                        debug_assert!(removed.is_some());
                                                        if let Some(card) = removed {
                                                            let target = get_card_insert_position(
                                                                spread(HandId::Player),
                                                                player_len
                                                            );

                                                            state.animations.push(Animation {
                                                                card,
                                                                at,
                                                                target,
                                                                action: AnimationAction::AddToHand(HandId::Player),
                                                                shown: true,
                                                                .. <_>::default()
                                                            });
                                                        }

                                                        to_next_turn!(state);
                                                    } else {
                                                        p_net_handle_negative_response!();
                                                    }
                                                }
                                            } else if input.pressed_this_frame(Button::B) {
                                                state.selection.card_index = selected;
                                                state.selection.player_menu = PlayerMenu::default();
                                            } else if let Some(dir) = input.dir_pressed_this_frame() {
                                                const GRID_LEN: usize = 3;

                                                #[derive(Clone, Copy, PartialEq, Eq)]
                                                enum Section {
                                                    Target,
                                                    Predicate,
                                                    Submit,
                                                }

                                                const GRID: [Section; GRID_LEN] = [
                                                    Section::Target, Section::Predicate, Section::Submit,
                                                ];

                                                let old_el = match state.ctx.hot {
                                                    CpuIdSelect => Some(Section::Target),
                                                    LurePredicate => Some(Section::Predicate),
                                                    Submit => Some(Section::Submit),
                                                    _ => None,
                                                };
                                                let mut el_i = GRID.iter()
                                                    .position(|el| Some(*el) == old_el)
                                                    .unwrap_or_default();

                                                match dir {
                                                    Dir::Up => match GRID[el_i] {
                                                        Section::Target => match *target {
                                                            CpuId::One => { *target = CpuId::Two; },
                                                            CpuId::Two => { *target = CpuId::Three; },
                                                            CpuId::Three => { *target = CpuId::One; },
                                                        },
                                                        Section::Predicate => predicate.wrapping_inc(),
                                                        Section::Submit => {}
                                                    },
                                                    Dir::Down => match GRID[el_i] {
                                                        Section::Target => match *target {
                                                            CpuId::One => { *target = CpuId::Three; },
                                                            CpuId::Two => { *target = CpuId::One; },
                                                            CpuId::Three => { *target = CpuId::Two; },
                                                        },
                                                        Section::Predicate => predicate.wrapping_dec(),
                                                        Section::Submit => {}
                                                    },
                                                    Dir::Left => if el_i == 0 {
                                                        el_i = GRID_LEN - 1;
                                                    } else {
                                                        el_i -= 1;
                                                    },
                                                    Dir::Right => if el_i >= GRID_LEN - 1 {
                                                        el_i = 0;
                                                    } else {
                                                        el_i += 1;
                                                    },
                                                }
                                                state.ctx.set_next_hot(match GRID[el_i] {
                                                    Section::Target => CpuIdSelect,
                                                    Section::Predicate => LurePredicate,
                                                    Section::Submit => Submit,
                                                });
                                            } else {
                                                // do nothing
                                            }
                                        }
                                        PlayerMenu::Asking {
                                            used,
                                            ref mut question,
                                            ref mut sub_menu,
                                        } => {
                                            let used = *used;

                                            macro_rules! p_handle_negative_response {
                                                () => {
                                                    let player_len = state.cards.player.len();

                                                    let drew = state.cards.deck.draw();

                                                    *menu = PlayerMenu::Fished{
                                                        used,
                                                        question: core::mem::take(question),
                                                        drew,
                                                    };
                                                    state.done_something_this_turn = true;

                                                    if let Some(card) = drew {
                                                        let at = DECK_XY;

                                                        let target = get_card_insert_position(
                                                            spread(HandId::Player),
                                                            player_len
                                                        );

                                                        state.animations.push(Animation {
                                                            card,
                                                            at,
                                                            target,
                                                            action: AnimationAction::AddToHand(HandId::Player),
                                                            .. <_>::default()
                                                        });
                                                    }
                                                }
                                            }

                                            match sub_menu {
                                                PlayerAskingSubMenu::Root => {
                                                    let rank = models::get_rank(used)
                                                        .expect("Asking used card should always have a rank!");

                                                    commands.draw_nine_slice(gfx::NineSlice::Window, ASKING_WINDOW);

                                                    let base_xy = ASKING_WINDOW.xy() + WINDOW_CONTENT_OFFSET;

                                                    let card_xy = base_xy;

                                                    commands.draw_card(
                                                        used,
                                                        card_xy
                                                    );

                                                    let target_xy = card_xy + CARD_WIDTH + (ASKING_WINDOW.h / 5);

                                                    let group = new_group!();

                                                    draw_cpu_id_quick_select(
                                                        group,
                                                        question.target.try_into().unwrap_or(CpuId::One),
                                                        target_xy
                                                    );

                                                    let suit_base_xy = target_xy + CPU_ID_SELECT_WH.w;

                                                    let suit_quick_select_rect = Rect::xy_wh(
                                                        suit_base_xy,
                                                        ASKING_SUIT_WH,
                                                    );

                                                    // TODO? Display the target card instead of text?
                                                    group.commands.print_centered(
                                                        Suit::TEXT[question.suit as u8 as usize],
                                                        Rect::xy_wh(
                                                            suit_base_xy + ASKING_SUIT_TEXT_OFFSET,
                                                            ASKING_SUIT_TEXT_WH,
                                                        ),
                                                        WHITE,
                                                    );

                                                    ui::draw_quick_select(
                                                        group,
                                                        suit_quick_select_rect,
                                                        AskSuit
                                                    );

                                                    let description_base_rect = unscaled::Rect::xy_wh(
                                                        unscaled::XY {
                                                            x: base_xy.x,
                                                            y: suit_base_xy.y + ASKING_SUIT_WH.h,
                                                        },
                                                        unscaled::WH {
                                                            w: ASKING_WINDOW.w,
                                                            h: (base_xy.y + ASKING_WINDOW.h)
                                                            - (suit_base_xy.y + ASKING_SUIT_WH.h),
                                                        }
                                                    );

                                                    let description = question.fresh_ask_description(rank);

                                                    let description_xy = gfx::center_line_in_rect(
                                                        description.len() as _,
                                                        description_base_rect,
                                                    );
                                                    group.commands.print_line(
                                                        description,
                                                        description_xy,
                                                        WHITE,
                                                    );

                                                    let submit_base_xy = unscaled::XY {
                                                        x: suit_base_xy.x + ASKING_SUIT_WH.w,
                                                        y: base_xy.y
                                                    };

                                                    if do_button(
                                                        group,
                                                        ButtonSpec {
                                                            id: Submit,
                                                            rect: fit_to_rest_of_window(
                                                                submit_base_xy,
                                                                ASKING_WINDOW,
                                                            ),
                                                            text: b"Submit",
                                                        }
                                                    ) {
                                                        if state.cards.hand(question.target)
                                                            .contains(zingers::NO_FISHING)
                                                        && should_use_no_fishing_against(
                                                            state.memories.memory(
                                                                CpuId::try_from(question.target)
                                                                .expect("target should be a Cpu player")
                                                            ),
                                                            state.cards.hand(question.target),
                                                            HandId::Player,
                                                            Predicate::RankSuit(rank, question.suit),
                                                            state.cards.active_count(),
                                                        ) {
                                                            discard_no_fishing(
                                                                &mut state.cards,
                                                                &mut state.animations,
                                                                question.target.with_target(HandId::Player),
                                                                Predicate::RankSuit(rank, question.suit)
                                                            );
                                                        } else {
                                                            let player_len = state.cards.player.len();
                                                            let target_hand = state.cards.hand_mut(question.target);

                                                            state.memories.asked_for(
                                                                HandId::Player,
                                                                Predicate::RankSuit(rank, question.suit)
                                                            );

                                                            let target_card = fish_card(rank, question.suit);

                                                            let mut found = None;
                                                            for i in 0..target_hand.len() {
                                                                let was_found = target_hand.get(i)
                                                                    .map(|card| card == target_card)
                                                                    .unwrap_or_default();
                                                                if was_found {
                                                                    found = Some((
                                                                        target_hand.remove(i).expect("We just looked at it! (player)"),
                                                                        i
                                                                    ));

                                                                    break
                                                                }
                                                            }

                                                            if let Some((card, i)) = found {
                                                                state.memories.found(
                                                                    HandId::Player,
                                                                    rank,
                                                                    question.suit
                                                                );
                                                                let at = get_card_position(
                                                                    spread(question.target),
                                                                    target_hand.len(),
                                                                    i,
                                                                );

                                                                let target = get_card_insert_position(
                                                                    spread(HandId::Player),
                                                                    player_len
                                                                );

                                                                state.animations.push(Animation {
                                                                    card,
                                                                    at,
                                                                    target,
                                                                    action: AnimationAction::AddToHand(HandId::Player),
                                                                    shown: true,
                                                                    .. <_>::default()
                                                                });

                                                                *menu = PlayerMenu::default();
                                                                state.done_something_this_turn = true;
                                                            } else if state.cards.player.contains(zingers::TWO_FISTED_FISHERMAN) {
                                                                *sub_menu = PlayerAskingSubMenu::TwoFistedFisherman;
                                                            } else {
                                                                p_handle_negative_response!();
                                                            }
                                                        }
                                                    } else if input.pressed_this_frame(Button::B) {
                                                        state.selection.card_index = selected;
                                                        state.selection.player_menu = PlayerMenu::default();
                                                    } else if let Some(dir) = input.dir_pressed_this_frame() {
                                                        const GRID_LEN: usize = 3;

                                                        #[derive(Clone, Copy, PartialEq, Eq)]
                                                        enum Section {
                                                            Target,
                                                            Suit,
                                                            Submit,
                                                        }

                                                        const GRID: [Section; GRID_LEN] = [
                                                            Section::Target, Section::Suit, Section::Submit,
                                                        ];

                                                        let old_el = match state.ctx.hot {
                                                            CpuIdSelect => Some(Section::Target),
                                                            AskSuit => Some(Section::Suit),
                                                            Submit => Some(Section::Submit),
                                                            _ => None,
                                                        };
                                                        let mut el_i = GRID.iter()
                                                            .position(|el| Some(*el) == old_el)
                                                            .unwrap_or_default();

                                                        match dir {
                                                            Dir::Up => match GRID[el_i] {
                                                                Section::Target => match question.target {
                                                                    HandId::Cpu1 => { question.target = HandId::Cpu2; },
                                                                    HandId::Cpu2 => { question.target = HandId::Cpu3; },
                                                                    HandId::Cpu3 => { question.target = HandId::Cpu1; },
                                                                    HandId::Player => {
                                                                        // Player cannot ask the player.
                                                                        debug_assert!(false);
                                                                    }
                                                                },
                                                                Section::Suit => {
                                                                    question.suit = question.suit.wrapping_inc()
                                                                },
                                                                Section::Submit => {}
                                                            },
                                                            Dir::Down => match GRID[el_i] {
                                                                Section::Target => match question.target {
                                                                    HandId::Cpu1 => { question.target = HandId::Cpu3; },
                                                                    HandId::Cpu2 => { question.target = HandId::Cpu1; },
                                                                    HandId::Cpu3 => { question.target = HandId::Cpu2; },
                                                                    HandId::Player => {
                                                                        // Player cannot ask the player.
                                                                        debug_assert!(false);
                                                                    }
                                                                },
                                                                Section::Suit => {
                                                                    question.suit = question.suit.wrapping_dec()
                                                                },
                                                                Section::Submit => {}
                                                            },
                                                            Dir::Left => if el_i == 0 {
                                                                el_i = GRID_LEN - 1;
                                                            } else {
                                                                el_i -= 1;
                                                            },
                                                            Dir::Right => if el_i >= GRID_LEN - 1 {
                                                                el_i = 0;
                                                            } else {
                                                                el_i += 1;
                                                            },
                                                        }
                                                        state.ctx.set_next_hot(match GRID[el_i] {
                                                            Section::Target => CpuIdSelect,
                                                            Section::Suit => AskSuit,
                                                            Section::Submit => Submit,
                                                        });
                                                    } else {
                                                        // do nothing
                                                    }
                                                },
                                                PlayerAskingSubMenu::TwoFistedFisherman => {
                                                    commands.draw_nine_slice(
                                                        gfx::NineSlice::Window,
                                                        PLAYER_TWO_FISTED_FISHERMAN_WINDOW
                                                    );

                                                    let base_xy = PLAYER_TWO_FISTED_FISHERMAN_WINDOW.xy()
                                                        + WINDOW_CONTENT_OFFSET;

                                                    let card_xy = base_xy;

                                                    commands.draw_card(
                                                        zingers::TWO_FISTED_FISHERMAN,
                                                        card_xy
                                                    );

                                                    let text_xy = card_xy + CARD_WIDTH;

                                                    commands.print_centered(
                                                        b"Do you want to play the\nTwo-Fisted Fisherman?",
                                                        Rect::xy_wh(
                                                            text_xy,
                                                            PLAYER_TWO_FISTED_FISHERMAN_TEXT_WH,
                                                        ),
                                                        WHITE,
                                                    );

                                                    let submit_xy = text_xy + PLAYER_TWO_FISTED_FISHERMAN_TEXT_WH.w;

                                                    let group = new_group!();

                                                    if do_button(
                                                        group,
                                                        ButtonSpec {
                                                            id: Submit,
                                                            rect: fit_to_rest_of_window(
                                                                submit_xy,
                                                                PLAYER_TWO_FISTED_FISHERMAN_WINDOW,
                                                            ),
                                                            text: b"Submit",
                                                        }
                                                    ) {
                                                        discard_two_fisted_fisherman(
                                                            &mut state.cards,
                                                            &mut state.animations,
                                                            HandId::Player,
                                                        );
                                                        state.selection.card_index = selected;
                                                        state.selection.player_menu = PlayerMenu::default();
                                                        state.done_something_this_turn = true;
                                                    } else if input.pressed_this_frame(Button::B) {
                                                        p_handle_negative_response!();
                                                    } else {
                                                        // do nothing
                                                    }

                                                    group.ctx.set_next_hot(Submit);
                                                },
                                            }
                                        },
                                        PlayerMenu::Fished {
                                            used,
                                            ref mut question,
                                            drew,
                                        } => {
                                            let used = *used;
                                            let rank = models::get_rank(used)
                                                .expect("Fished used card should always have a rank!");

                                            commands.draw_nine_slice(gfx::NineSlice::Window, GO_FISH_WINDOW);

                                            let base_xy = GO_FISH_WINDOW.xy() + WINDOW_CONTENT_OFFSET;

                                            let target_card_xy = base_xy;

                                            let target_card = fish_card(rank, question.suit);

                                            commands.draw_card(
                                                target_card,
                                                target_card_xy
                                            );

                                            let drew_card_xy = target_card_xy + CARD_WIDTH * 2;
                                            let drew_card_xy = drew_card_xy - WINDOW_CONTENT_OFFSET.w * 2;

                                            if let Some(card) = drew {
                                                commands.draw_card(
                                                    *card,
                                                    drew_card_xy
                                                );
                                            } else {
                                                commands.print_centered(
                                                    b"Nothin'",
                                                    unscaled::Rect::xy_wh(
                                                        drew_card_xy,
                                                        WH { w: CARD_WIDTH, h: CARD_HEIGHT }
                                                    ),
                                                    WHITE,
                                                );
                                            }

                                            let description_base_xy =
                                                target_card_xy
                                                + CARD_HEIGHT;

                                            let description_base_rect = fit_to_rest_of_window(
                                                description_base_xy,
                                                GO_FISH_WINDOW,
                                            );

                                            let description = question.fresh_fished_description(
                                                rank,
                                                *drew,
                                                description_base_rect.w,
                                            );

                                            commands.print_centered(
                                                description,
                                                description_base_rect,
                                                WHITE,
                                            );
                                            // TODO? Dorky sound effect?

                                            if input.pressed_this_frame(Button::A)
                                            || input.pressed_this_frame(Button::B) {
                                                let target_card = fish_card(rank, question.suit);

                                                if let Some(true) = drew
                                                    .map(|card| card == target_card) {
                                                    state.memories.fished_for(
                                                        HandId::Player,
                                                        rank,
                                                        question.suit
                                                    );
                                                    state.selection.card_index = state.cards.player.len().saturating_sub(1);
                                                    state.selection.player_menu = PlayerMenu::default();
                                                    state.done_something_this_turn = true;
                                                } else {
                                                    state.stack.push(Play {
                                                        sub_turn_ids: HandId::Player.next_to_current(),
                                                        sub_turn_index: 0,
                                                        kind: PlayKind::FishedUnsuccessfully {
                                                            source: HandId::Player,
                                                        }
                                                    });
                                                    allow_to_respond!(state);
                                                };
                                            }
                                        }
                                    }
                                } else {
                                    if state.has_started {
                                        draw_dead_in_the_water(commands);

                                        if input.pressed_this_frame(Button::A)
                                        || input.pressed_this_frame(Button::B) {
                                            let drew = state.cards.deck.draw();

                                            state.selection.card_index = state.cards.player.len().saturating_sub(1);
                                            state.selection.player_menu = PlayerMenu::default();

                                            if let Some(card) = drew {
                                                let at = DECK_XY;

                                                let target = get_card_insert_position(
                                                    spread(HandId::Player),
                                                    state.cards.player.len(),
                                                );

                                                state.animations.push(Animation {
                                                    card,
                                                    at,
                                                    target,
                                                    action: AnimationAction::AddToHand(HandId::Player),
                                                    .. <_>::default()
                                                });
                                            }
                                        }
                                    } else {
                                        // Wait until the initial animations have completed, putting
                                        // a card in the player's hand, which means we won't come back
                                        // here.
                                    }
                                }
                            },
                            Ok(id) => {
                                let menu = &mut state.cpu_menu;
                                match menu {
                                    CpuMenu::Selecting => {
                                        // Showing this avoids a flicker for the one frame the Cpu
                                        // is selecting when they stop waiting.
                                        // Maybe enforce that the Cpu windows must all be the same size?
                                        commands.draw_nine_slice(
                                            gfx::NineSlice::Window,
                                            CPU_SELECTING_WINDOW
                                        );

                                        let hand_id = HandId::from(id);
                                        let hand = state.cards.hand(hand_id);
                                        if hand.is_empty() {
                                            *menu = CpuMenu::DeadInTheWater;
                                        } else {
                                            //TODO restore
                                            if let Some((rank, suit, target)) = state.memories.memory(id)
                                                .informed_question(hand, hand_id) {
                                                let mut question = Question::default();

                                                question.suit = suit;
                                                question.target = target;

                                                *menu = CpuMenu::Asking(
                                                    rank,
                                                    question,
                                                );
                                                state.done_something_this_turn = true;
                                            }

                                            if let CpuMenu::Selecting = *menu {
                                                let mut zinger_to_play = None;
                                                // TODO? randomize order through the cards here to make Cpu
                                                // player less predictable?

                                                let mut ordering = HandOrdering::default();

                                                for (i, card) in hand.enumerated_iter() {
                                                    if models::get_zinger(card).is_some() {
                                                        // TODO? Are all zingers high priority?
                                                        ordering.move_to_first(i);
                                                    }
                                                }

                                                for card in hand.ordering_iter(ordering) {
                                                    if let Some(rank) = models::get_rank(card) {
                                                        let besides = HandId::besides(hand_id);
                                                        let target_id = besides[
                                                            xs::range(&mut state.rng, 0..besides.len() as u32) as usize
                                                        ];

                                                        // TODO? Decide what suit to ask for intelligently
                                                        // in this case? Does it matter given that asking
                                                        // for a card you have as a distraction is viable?
                                                        // Maybe only ask for a card you have when your
                                                        // have 4 in your hand already?
                                                        let suit = Suit::from_rng(&mut state.rng);

                                                        let mut question = Question::default();
                                                        question.target = target_id;
                                                        question.suit = suit;

                                                        *menu = CpuMenu::Asking(
                                                            rank,
                                                            question,
                                                        );
                                                        state.done_something_this_turn = true;
                                                        break
                                                    } else if let Some(zinger) = models::get_zinger(card) {
                                                        match zinger {
                                                            Zinger::DeadScubaDiver => {
                                                                // TODO? is there a case where we'd rather play
                                                                // it here than wait to respond to our own turn?
                                                            }
                                                            Zinger::TheNet | Zinger::TheLure => {
                                                                if state.done_something_this_turn {
                                                                    // Cannot play it
                                                                } else if should_play_super_ask(
                                                                    &state.cards,
                                                                    state.cards.hand(hand_id),
                                                                    &state.stack,
                                                                    &state.memories,
                                                                    id,
                                                                ) {
                                                                    zinger_to_play = Some(zinger);
                                                                    break
                                                                } else {
                                                                    // Don't discard it
                                                                }
                                                            }
                                                            Zinger::DivineIntervention => {
                                                                if state.done_something_this_turn {
                                                                    // Cannot play it
                                                                } else if should_shed_zingers(
                                                                    &state.cards,
                                                                    state.cards.hand(hand_id),
                                                                    &state.stack,
                                                                ) {
                                                                    zinger_to_play = Some(zinger);
                                                                    break
                                                                } else {
                                                                    // Don't discard it
                                                                }
                                                            }
                                                            Zinger::TwoFistedFisherman => {
                                                                // Can't play that now. Wait until asking for something.
                                                            }
                                                            // TODO Play other Zingers sometimes.
                                                            _ => { todo!("Attempted to play {zinger:?}") }
                                                        }
                                                    } else {
                                                        debug_assert!(false, "Non-fish, non-zinger card!? {card}");
                                                    }
                                                }

                                                match zinger_to_play {
                                                    Some(Zinger::DivineIntervention) => {
                                                        discard_divine_intervention(
                                                            &mut state.cards,
                                                            &mut state.animations,
                                                            hand_id
                                                        );
                                                        // We're doing the special
                                                        // discard action, so don't
                                                        // do anything to the stack.
                                                    }
                                                    //Some(Zinger::TheLure) => {
                                                        //state.stack.push()
                                                    //}
                                                    Some(zinger) => {
                                                        todo!("cpu handle selecting {zinger:?}");
                                                    }
                                                    None => {
                                                        if let CpuMenu::Selecting = *menu {
                                                            *menu = CpuMenu::DeadInTheWater;
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    },
                                    CpuMenu::Asking(rank, ref mut question) => {
                                        macro_rules! handle_ask {
                                            () => {
                                                let rank = *rank;

                                                state.memories.asked_for(
                                                    id.into(),
                                                    Predicate::RankSuit(rank, question.suit)
                                                );

                                                let target_card = fish_card(rank, question.suit);
                                                let my_len = state.cards.hand(id.into()).len();

                                                let target_hand = state.cards.hand_mut(question.target);

                                                let mut found = None;
                                                // TODO? randomize order here to make it harder to learn their
                                                // whole hand with glass bottom boat
                                                for i in 0..target_hand.len() {
                                                    let was_found = target_hand.get(i)
                                                        .map(|card| card == target_card)
                                                        .unwrap_or_default();
                                                    if was_found {
                                                        found = Some((
                                                            target_hand.remove(i)
                                                                .expect("We just looked at it! (cpu)"),
                                                            i
                                                        ));

                                                        break
                                                    }
                                                }

                                                if let Some((card, i)) = found {
                                                    state.memories.found(id.into(), rank, question.suit);

                                                    let at = get_card_position(
                                                        spread(question.target),
                                                        target_hand.len(),
                                                        i,
                                                    );

                                                    let target = get_card_insert_position(
                                                        spread(id.into()),
                                                        my_len
                                                    );

                                                    state.animations.push(Animation {
                                                        card,
                                                        at,
                                                        target,
                                                        action: AnimationAction::AddToHand(id.into()),
                                                        shown: true,
                                                        .. <_>::default()
                                                    });

                                                    state.cpu_menu = CpuMenu::WaitingForSuccesfulAsk;
                                                    state.done_something_this_turn = true;
                                                } else {
                                                    cpu_handle_negative_response!(state, menu, id, rank, question.suit);
                                                }
                                            }
                                        }

                                        match (
                                            question.target,
                                            state.cards.hand(question.target).contains(zingers::NO_FISHING),
                                        ) {
                                            (HandId::Player, true) => {
                                                commands.draw_nine_slice(gfx::NineSlice::Window, NO_FISHING_WINDOW);

                                                let base_xy = NO_FISHING_WINDOW.xy()
                                                    + WINDOW_CONTENT_OFFSET;

                                                let card_xy = base_xy;

                                                commands.draw_card(
                                                    zingers::NO_FISHING,
                                                    card_xy,
                                                );

                                                let description_base_xy = card_xy + CARD_WIDTH;

                                                let description_base_rect = unscaled::Rect::xy_wh(
                                                    description_base_xy,
                                                    unscaled::WH {
                                                        w: NO_FISHING_WINDOW.w - CARD_WIDTH * 2,
                                                        h: NO_FISHING_WINDOW.h - WINDOW_CONTENT_OFFSET.h * 2,
                                                    }
                                                );

                                                let description = question.fresh_cpu_ask_description(
                                                    *rank,
                                                    id.into(),
                                                    description_base_rect.w,
                                                );

                                                commands.print_centered(
                                                    description,
                                                    description_base_rect,
                                                    WHITE,
                                                );

                                                let submit_base_xy = NO_FISHING_WINDOW.xy()
                                                    + WINDOW_CONTENT_OFFSET.h
                                                    + NO_FISHING_WINDOW.w
                                                    - (CARD_WIDTH + WINDOW_CONTENT_OFFSET.w);

                                                let group = new_group!();

                                                if do_button(
                                                    group,
                                                    ButtonSpec {
                                                        id: Submit,
                                                        rect: fit_to_rest_of_window(
                                                            submit_base_xy,
                                                            NO_FISHING_WINDOW,
                                                        ),
                                                        text: b"Play",
                                                    }
                                                ) {
                                                    discard_no_fishing(
                                                        &mut state.cards,
                                                        &mut state.animations,
                                                        question.target.with_target(id.into()),
                                                        Predicate::RankSuit(*rank, question.suit)
                                                    );
                                                } else if input.pressed_this_frame(Button::B) {
                                                    handle_ask!();
                                                }

                                                group.ctx.set_next_hot(Submit);
                                            },
                                            (_, has_no_fishing) => {
                                                // If we reach this branch when the target is
                                                // `HandId::Player`, then we know that `has_no_fishing`
                                                // is false.
                                                if has_no_fishing
                                                && should_use_no_fishing_against(
                                                    state.memories.memory(
                                                        CpuId::try_from(question.target)
                                                        .expect("target should be a Cpu player")
                                                    ),
                                                    state.cards.hand(question.target),
                                                    id.into(),
                                                    Predicate::RankSuit(*rank, question.suit),
                                                    state.cards.active_count(),
                                                ) {
                                                    discard_no_fishing(
                                                        &mut state.cards,
                                                        &mut state.animations,
                                                        question.target.with_target(id.into()),
                                                        Predicate::RankSuit(*rank, question.suit)
                                                    );
                                                } else {
                                                    commands.draw_nine_slice(gfx::NineSlice::Window, CPU_ASKING_WINDOW);

                                                    let base_xy = CPU_ASKING_WINDOW.xy() + WINDOW_CONTENT_OFFSET;

                                                    let description_base_xy = base_xy;

                                                    let description_base_rect = fit_to_rest_of_window(
                                                        description_base_xy,
                                                        CPU_ASKING_WINDOW,
                                                    );

                                                    let description = question.fresh_cpu_ask_description(
                                                        *rank,
                                                        id.into(),
                                                        description_base_rect.w,
                                                    );

                                                    commands.print_centered(
                                                        description,
                                                        description_base_rect,
                                                        WHITE,
                                                    );

                                                    if input.pressed_this_frame(Button::A)
                                                    | input.pressed_this_frame(Button::B) {
                                                        handle_ask!();
                                                    }
                                                }
                                            },
                                        }
                                    },
                                    CpuMenu::DeadInTheWater => {
                                        draw_dead_in_the_water(commands);

                                        // Just wait until player acknowledges turn.
                                        if input.pressed_this_frame(Button::A)
                                        | input.pressed_this_frame(Button::B) {
                                            let drew = state.cards.deck.draw();
                                            let hand_id = id.into();
                                            let len = state.cards.hand(hand_id).len();

                                            state.selection.card_index = state.cards.player.len().saturating_sub(1);
                                            state.selection.player_menu = PlayerMenu::default();
                                            state.cpu_menu = CpuMenu::default();

                                            if let Some(card) = drew {
                                                let at = DECK_XY;

                                                let target = get_card_insert_position(
                                                    spread(hand_id),
                                                    len
                                                );

                                                state.animations.push(Animation {
                                                    card,
                                                    at,
                                                    target,
                                                    action: AnimationAction::AddToHand(hand_id),
                                                    .. <_>::default()
                                                });
                                            }
                                        }
                                    },
                                    // TODO? retain their target card for this message?
                                    CpuMenu::WaitingForSuccesfulAsk => {
                                        commands.draw_nine_slice(gfx::NineSlice::Window, CPU_SUCCESFUL_ASK_WINDOW);

                                        let base_xy = CPU_SUCCESFUL_ASK_WINDOW.xy() + WINDOW_CONTENT_OFFSET;

                                        let description_base_xy = base_xy;

                                        let description_base_rect = fit_to_rest_of_window(
                                            description_base_xy,
                                            CPU_SUCCESFUL_ASK_WINDOW,
                                        );

                                        commands.print_centered(
                                            b"They got what they were asking for!",
                                            description_base_rect,
                                            WHITE,
                                        );
                                    },
                                    // TODO? retain their target card for this message?
                                    CpuMenu::WaitingWhenGotWhatWasFishingFor => {
                                        commands.draw_nine_slice(gfx::NineSlice::Window, CPU_SUCCESFUL_FISH_WINDOW);

                                        let base_xy = CPU_SUCCESFUL_FISH_WINDOW.xy() + WINDOW_CONTENT_OFFSET;

                                        let description_base_xy = base_xy;

                                        let description_base_rect = fit_to_rest_of_window(
                                            description_base_xy,
                                            CPU_SUCCESFUL_FISH_WINDOW,
                                        );

                                        commands.print_centered(
                                            b"They got what they fished for!",
                                            description_base_rect,
                                            WHITE,
                                        );
                                    },
                                    CpuMenu::WaitingWhenPlayedTwoFistedFisherman => {
                                        commands.draw_nine_slice(gfx::NineSlice::Window, CPU_TWO_FISTED_FISHERMAN_WINDOW);

                                        let base_xy = CPU_TWO_FISTED_FISHERMAN_WINDOW.xy()
                                            + WINDOW_CONTENT_OFFSET;

                                        let description_base_xy = base_xy;

                                        let description_base_rect = fit_to_rest_of_window(
                                            description_base_xy,
                                            CPU_TWO_FISTED_FISHERMAN_WINDOW,
                                        );

                                        commands.print_centered(
                                            b"They played the Two-Fisted \nFisherman! So they get to go again!",
                                            description_base_rect,
                                            WHITE,
                                        );
                                    },
                                }
                            }
                        }
                    }
                    // Resolve the card on the top of the stack
                    Some(play) => {
                        let mut go_again = false;
                        match play.kind {
                            PlayKind::FishedUnsuccessfully{ .. } => {
                                assert!(state.stack.is_empty());
                                // Just move to the next turn
                            },
                            PlayKind::NoFishing{ targeting: Targeting{ source: _, target }, predicate } => {
                                match CpuId::try_from(target) {
                                    Err(()) => {
                                        todo!("probably move p_handle_negative_response!(); here")
                                    }
                                    Ok(asker_id) => {
                                        match predicate {
                                            Predicate::RankSuit(rank, suit) => {
                                                let menu = &mut state.cpu_menu;
                                                cpu_handle_negative_response!(state, menu, asker_id, rank, suit);
                                            },
                                            Predicate::Net(_predicate) => {
                                                todo!("implement cpu_net_handle_negative_response!(); here")
                                            }
                                        }
                                    }
                                }
                            }
                            PlayKind::TwoFistedFisherman{ source: _, cancelled } => {
                                if !cancelled {
                                    // If this turns out to be false ever, we need to
                                    // store that the player gets an extra turn somewhere
                                    // more persistently.
                                    assert!(state.stack.is_empty());
                                    go_again = true;
                                }
                            }
                            PlayKind::TheNet{ targeting: Targeting{ source, target }, predicate } => {
                                match CpuId::try_from(target) {
                                    Err(()) => {
                                        todo!("probably move p_handle_negative_response!(); here")
                                    }
                                    Ok(target_id) => {
                                        macro_rules! net_handle_negative_response {
                                            () => {
                                                let len = state.cards.hand(source).len();

                                                let drew = state.cards.deck.draw();

                                                if let Some(card) = drew {
                                                    let at = DECK_XY;

                                                    let target = get_card_insert_position(
                                                        spread(source),
                                                        len
                                                    );

                                                    state.animations.push(Animation {
                                                        card,
                                                        at,
                                                        target,
                                                        action: AnimationAction::AddToHand(source),
                                                        .. <_>::default()
                                                    });
                                                }

                                                // This card counts as a turn, so just go on to the next turn.
                                                to_next_turn!(state);
                                            }
                                        }

                                        if state.cards.hand(target)
                                            .contains(zingers::NO_FISHING)
                                        && should_use_no_fishing_against(
                                            state.memories.memory(target_id),
                                            state.cards.hand(target),
                                            source,
                                            Predicate::Net(predicate),
                                            state.cards.active_count(),
                                        ) {
                                            discard_no_fishing(
                                                &mut state.cards,
                                                &mut state.animations,
                                                target.with_target(source),
                                                Predicate::Net(predicate)
                                            );
                                            net_handle_negative_response!();
                                        } else {
                                            let len = state.cards.hand(source).len();
                                            let target_hand = state.cards.hand_mut(target);

                                            state.memories.asked_for(
                                                source,
                                                Predicate::Net(predicate)
                                            );

                                            let memory = state.memories.memory(target_id);
                                            let mut found = None;
                                            match predicate {
                                                NetPredicate::Rank(rank) => {
                                                    for (i, card) in target_hand.enumerated_iter() {
                                                        match (get_rank(card), get_suit(card)) {
                                                            (Some(r), Some(suit)) if r == rank => {
                                                                found = Some((rank, suit, i));
                                                                // TODO? pick best one to give up, when there's no unlikely ones left?
                                                                // If this is an undesirable to give up card, keep looking.
                                                                if memory.is_likely_to_fill_rank_soon(
                                                                    target,
                                                                    rank
                                                                ) || memory.is_likely_to_fill_rank_soon(
                                                                    source,
                                                                    rank
                                                                ) {
                                                                    continue
                                                                }
                                                                break
                                                            }
                                                            _ => {}
                                                        }
                                                    }
                                                }
                                                NetPredicate::Suit(suit) => {
                                                    for (i, card) in target_hand.enumerated_iter() {
                                                        match (get_rank(card), get_suit(card)) {
                                                            (Some(rank), Some(s)) if s == suit => {
                                                                found = Some((rank, suit, i));
                                                                // TODO? pick best one to give up, when there's no unlikely ones left?
                                                                // If this is an undesirable to give up card, keep looking.
                                                                if memory.is_likely_to_fill_rank_soon(
                                                                    target,
                                                                    rank
                                                                ) || memory.is_likely_to_fill_rank_soon(
                                                                    source,
                                                                    rank
                                                                ) {
                                                                    continue
                                                                }
                                                                break
                                                            }
                                                            _ => {}
                                                        }
                                                    }
                                                }
                                            }

                                            if let Some((rank, suit, i)) = found {
                                                state.memories.found(
                                                    source,
                                                    rank,
                                                    suit
                                                );
                                                let at = get_card_position(
                                                    spread(target),
                                                    target_hand.len(),
                                                    i,
                                                );

                                                let removed = target_hand.remove(i);
                                                debug_assert!(removed.is_some());
                                                if let Some(card) = removed {
                                                    let target = get_card_insert_position(
                                                        spread(source),
                                                        len
                                                    );

                                                    state.animations.push(Animation {
                                                        card,
                                                        at,
                                                        target,
                                                        action: AnimationAction::AddToHand(HandId::Player),
                                                        shown: true,
                                                        .. <_>::default()
                                                    });
                                                }

                                                to_next_turn!(state);
                                            } else {
                                                net_handle_negative_response!();
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        if !go_again && state.stack.is_empty() {
                            // We've resolved the whole stack.
                            to_next_turn!(state);
                        }
                    }
                }
            }
        }
    }
}

fn draw_dead_in_the_water(commands: &mut Commands) {
    commands.draw_nine_slice(
        gfx::NineSlice::Window,
        DEAD_IN_THE_WATER_WINDOW
    );

    let base_xy = DEAD_IN_THE_WATER_WINDOW.xy() + WINDOW_CONTENT_OFFSET;

    let description_base_xy = base_xy;

    let description_base_rect = fit_to_rest_of_window(
        description_base_xy,
        DEAD_IN_THE_WATER_WINDOW,
    );

    commands.print_centered(
        b"\"I am dead in the water.\"",
        description_base_rect,
        WHITE,
    );
}

fn get_card_insert_position(spread: Spread, len: u8) -> XY {
    get_card_position(spread, len + 1, len)
}

fn get_card_position(spread: Spread, len: u8, index: models::CardIndex) -> XY {
    match spread {
        Spread::LTR((min_edge, max_edge), y) => {
            if len == 0 {
                return XY { x: min_edge, y };
            }

            let span = CARD_WIDTH;

            let full_width = max_edge.saturating_point_sub(min_edge);
            let usable_width = full_width.saturating_sub(span);

            let offset = core::cmp::min(usable_width / len.into(), span);

            XY {
                x: min_edge.saturating_add(offset * index.into()),
                y
            }
        },
        Spread::TTB((min_edge, max_edge), x) => {
            if len == 0 {
                return XY { x, y: min_edge };
            }

            let span = CARD_HEIGHT;

            let full_width = max_edge.saturating_point_sub(min_edge);
            let usable_height = full_width.saturating_sub(span);

            let offset = core::cmp::min(usable_height / len.into(), span);

            XY {
                x,
                y: min_edge.saturating_add(offset * index.into())
            }
        },
    }
}

fn perform_game_warden(
    cards: &mut Cards,
    animations: &mut Animations,
    rng: &mut Xs,
    Targeting {
        source,
        target,
    }: Targeting,
) -> Option<()> {
    let hand = cards.hand_mut(target);

    if hand.is_empty() {
        return None;
    }

    let i = xs::range(rng, 0..hand.len() as u32) as CardIndex;

    let at = get_card_position(
        spread(target),
        hand.len(),
        i,
    );

    let () = hand
        .remove(i)
        .map(|card| {
            animations.push(Animation {
                card,
                at,
                target: DECK_XY,
                action: AnimationAction::PerformGameWarden,
                .. <_>::default()
            })
        })?;

    discard_game_warden(
        cards,
        animations,
        source.into(),
    );

    Some(())
}

mod text {
    // NOTE This does not use a general purpose definition of whitespace.
    // This should count a byte as whitespace iff it has all blank
    // pixels in this game's font.
    #[inline]
    fn is_byte_whitespace(byte: u8) -> bool {
        (byte <= b' ' && byte != b'\xED' && byte != b'\xE9')
        || byte > b'~'
    }

    macro_rules! test_log {
        //($($tokens: tt)*) => {dbg!(&$($tokens)*);}
        ($($tokens: tt)*) => {}
    }

    pub fn bytes_reflow_in_place(bytes: &mut Vec<u8>, width_in_chars: usize) {
        if width_in_chars == 0 || bytes.is_empty() {
            test_log!("width_in_chars == 0 || bytes.is_empty()");
            return;
        }

        test_log!("start");
        test_log!((std::str::from_utf8(&bytes), width_in_chars));
        let used_len = bytes.len();
        let extra = bytes.len() / width_in_chars;

        if extra == 0 {
            test_log!("no extra");
            return;
        }
        bytes.reserve(extra);

        //fill with 0's to capacity
        for _ in 0..extra {
            bytes.push(0);
        }

        //shift used parts down to the end
        {
            let mut index = bytes.len() - 1;
            test_log!(index);
            test_log!(used_len);
            test_log!((0..used_len).rev());
            for i in (0..used_len).rev() {
                test_log!(index);
                test_log!(i);
                bytes[index] = bytes[i];
                index -= 1;
            }
        }

        let mut index = 0;
        {
            //full length - used_len == (used_len + extra) - used_len == extra
            let shifted_start = extra;
            let mut next_i = shifted_start;
            test_log!(std::str::from_utf8(&bytes));
            test_log!(shifted_start);
            //scan from the start of the (moved) used portion and copy it back to the front
            //inserting newlines where appropiate.
            let mut x = 0;
            while let Some((w_i, len)) = bytes_next_word(bytes, &mut next_i) {
                test_log!((w_i, len));
                test_log!(std::str::from_utf8(&bytes[w_i..w_i + len]));
                x += len;
                test_log!(x);

                if x == width_in_chars && x == len {
                    for i in w_i..w_i + len {
                        bytes[index] = bytes[i];
                        index += 1;
                    }
                    continue;
                }

                if x >= width_in_chars {
                    bytes[index] = b'\n';
                    index += 1;

                    x = len;
                } else if x > len {
                    bytes[index] = b' ';
                    index += 1;

                    x += 1;
                }

                for i in w_i..w_i + len {
                    bytes[index] = bytes[i];
                    index += 1;
                }
                test_log!(std::str::from_utf8(bytes));
            }
        }
        test_log!("!");
        test_log!(std::str::from_utf8(bytes));
        test_log!(index);
        bytes.truncate(index);
    }

    #[test]
    fn bytes_reflow_in_place_reflows_this_found_example() {
        let initial = b"You asked for the Red Shrimp but you didn't get it.";
        assert_eq!(initial.len(), 51);
        let mut actual = initial.to_vec();

        bytes_reflow_in_place(&mut actual, 30);

        assert_eq!(
            &actual,
            b"You asked for the Red Shrimp\nbut you didn't get it."
        );
    }

    fn bytes_next_word(bytes: &[u8], in_i: &mut usize) -> Option<(usize, usize)> {
        test_log!("next");
        test_log!(std::str::from_utf8(&bytes));
        let end = bytes.len();
        test_log!(end);
        test_log!(in_i);

        for index in *in_i..end {
            if !is_byte_whitespace(bytes[index]) {
                let out_i = index;
                let mut len = 0;

                // The suggestion that includes `.take(end + 1)` changes the behaviour
                // because `end == bytes.len()`, so it loops one less time.
                #[allow(clippy::needless_range_loop)]
                for i in index + 1..=end {
                    *in_i = i;
                    if i == end || is_byte_whitespace(bytes[i]) {
                        len = i - out_i;
                        break;
                    }
                }
                if *in_i == end - 1 {
                    *in_i = end;
                }

                return Some((out_i, len));
            }
        }
        test_log!("None");
        None
    }

    #[test]
    fn bytes_next_word_works_on_this_found_example() {
        let bytes = b"You asked for the Red Shrimp but you didn't get it.";
        let expected = [
            Some(( 0, 3)), // You
            Some(( 4, 5)), // asked
            Some((10, 3)), // for
            Some((14, 3)), // the
        ];
        let mut i = 0;

        for e in expected {
            assert_eq!(
                bytes_next_word(bytes, &mut i),
                e,
            );
        }
    }
}

fn fit_to_rest_of_window(
    base_xy: unscaled::XY,
    window: unscaled::Rect
) -> unscaled::Rect {
    unscaled::Rect::xy_wh(
        base_xy,
        (
            window.xy()
            + (window.wh() - WINDOW_CONTENT_OFFSET)
        ) - base_xy,
    )
}

fn draw_cpu_id_quick_select(
    group: &mut ui::Group,
    current: CpuId,
    xy: XY,
) {
    group.commands.print_centered(
        CpuId::TEXT[current as u8 as usize],
        Rect::xy_wh(
            xy + CPU_ID_SELECT_TEXT_OFFSET,
            CPU_ID_SELECT_TEXT_WH,
        ),
        WHITE,
    );

    ui::draw_quick_select(
        group,
        Rect::xy_wh(
            xy,
            CPU_ID_SELECT_WH,
        ),
        CpuIdSelect
    );
}

const CARD_QUICK_SELECT_WH: unscaled::WH = unscaled::WH {
    w: CARD_WIDTH,
    h: unscaled::H(
        CARD_HEIGHT.get()
        + CHEVRON_H.get() * 2
    ),
};

const CARD_WIN_H: unscaled::Inner = CARD_HEIGHT.get()
    + WINDOW_CONTENT_OFFSET.h.get() * 2;

const ASKING_WINDOW: unscaled::Rect = {
    const OFFSET: unscaled::Inner = 8;

    unscaled::Rect {
        x: X(OFFSET),
        y: Y((command::HEIGHT - CARD_WIN_H) / 2),
        w: W(command::WIDTH - OFFSET * 2),
        h: H(CARD_WIN_H),
    }
};

const PLAYER_TWO_FISTED_FISHERMAN_WINDOW: unscaled::Rect = {
    const OFFSET: unscaled::Inner = 8;

    unscaled::Rect {
        x: X(OFFSET),
        y: Y((command::HEIGHT - CARD_WIN_H) / 2),
        w: W(command::WIDTH - OFFSET * 2),
        h: H(CARD_WIN_H),
    }
};

const PLAYER_TWO_FISTED_FISHERMAN_TEXT_WH: unscaled::WH = unscaled::WH {
    w: W(CARD_WIDTH.get() * 3),
    h: H(PLAYER_TWO_FISTED_FISHERMAN_WINDOW.h.0 - (WINDOW_CONTENT_OFFSET.h.0 * 2)),
};

const PLAYER_NET_WINDOW: unscaled::Rect = {
    const OFFSET: unscaled::Inner = 8;

    const WIN_H: unscaled::Inner = CARD_HEIGHT.get()
    + WINDOW_CONTENT_OFFSET.h.get() * 2;

    unscaled::Rect {
        x: X(OFFSET),
        y: Y((command::HEIGHT - WIN_H) / 2),
        w: W(command::WIDTH - OFFSET * 2),
        h: H(WIN_H),
    }
};

const NET_PREDICATE_SELECT_WH: unscaled::WH = unscaled::WH {
    w: W(CARD_WIDTH.get() * 3 - CPU_ID_SELECT_WH.w.get() * 7 / 8),
    h: H(PLAYER_NET_WINDOW.h.0 - (WINDOW_CONTENT_OFFSET.h.0 * 2)),
};

const NET_PREDICATE_SELECT_TEXT_OFFSET: unscaled::WH = CPU_ID_SELECT_TEXT_OFFSET;

const NET_PREDICATE_SELECT_TEXT_WH: unscaled::WH = unscaled::WH {
    w: W(NET_PREDICATE_SELECT_WH.w.get() - NET_PREDICATE_SELECT_TEXT_OFFSET.w.get() * 2),
    h: NET_PREDICATE_SELECT_WH.h,
};

const PLAYER_LURE_WINDOW: unscaled::Rect = {
    const OFFSET: unscaled::Inner = 8;

    const WIN_H: unscaled::Inner = CARD_HEIGHT.get()
    + WINDOW_CONTENT_OFFSET.h.get() * 2;

    unscaled::Rect {
        x: X(OFFSET),
        y: Y((command::HEIGHT - WIN_H) / 2),
        w: W(command::WIDTH - OFFSET * 2),
        h: H(WIN_H),
    }
};

const LURE_PREDICATE_SELECT_WH: unscaled::WH = unscaled::WH {
    w: W(CARD_WIDTH.get() * 3 - CPU_ID_SELECT_WH.w.get() * 7 / 8),
    h: H(PLAYER_LURE_WINDOW.h.0 - (WINDOW_CONTENT_OFFSET.h.0 * 2)),
};

const LURE_PREDICATE_SELECT_TEXT_OFFSET: unscaled::WH = CPU_ID_SELECT_TEXT_OFFSET;

const LURE_PREDICATE_SELECT_TEXT_WH: unscaled::WH = unscaled::WH {
    w: W(LURE_PREDICATE_SELECT_WH.w.get() - LURE_PREDICATE_SELECT_TEXT_OFFSET.w.get() * 2),
    h: LURE_PREDICATE_SELECT_WH.h,
};

const CPU_ID_SELECT_TEXT_OFFSET: unscaled::WH = unscaled::WH {
    w: W(0),
    h: H(
        gfx::CHEVRON_H.get()
        + (gfx::CHAR_SPACING as unscaled::Inner / 2)
    ),
};

const CPU_ID_SELECT_WH: unscaled::WH = unscaled::WH {
    w: W(ASKING_WINDOW.w.get() / 3),
    h: H(
        gfx::CHEVRON_H.get()
        + gfx::CHAR_SPACING as unscaled::Inner
        + gfx::CHAR_ADVANCE_H.get().get()
        + gfx::CHAR_SPACING as unscaled::Inner
        + gfx::CHEVRON_H.get()
    ),
};

const CPU_ID_SELECT_TEXT_WH: unscaled::WH = unscaled::WH {
    w: CPU_ID_SELECT_WH.w,
    h: CPU_ID_SELECT_TEXT_OFFSET.h,
};

const RANK_SELECT_SPREAD_WH: unscaled::WH = unscaled::WH {
    w: W(0),
    h: H(CARD_HEIGHT.get() / 16),
};

const RANK_SELECT_WH: unscaled::WH = unscaled::WH {
    w: W(ASKING_WINDOW.w.get() / 3),
    h: unscaled::H(
        CHEVRON_H.get()
        + (RANK_SELECT_SPREAD_WH.h.get() * (Suit::COUNT - 1) as unscaled::Inner)
        + CARD_HEIGHT.get()
        + CHEVRON_H.get()
    ),
};


const ASKING_SUIT_TEXT_OFFSET: unscaled::WH = CPU_ID_SELECT_TEXT_OFFSET;

const ASKING_SUIT_WH: unscaled::WH = CPU_ID_SELECT_WH;

const ASKING_SUIT_TEXT_WH: unscaled::WH = CPU_ID_SELECT_TEXT_WH;

const GO_FISH_WINDOW: unscaled::Rect = {
    const WIN_W: unscaled::Inner = CARD_WIDTH.get() * 3;
    const X_OFFSET: unscaled::Inner = (command::WIDTH - WIN_W) / 2;
    const Y_OFFSET: unscaled::Inner = (
        command::HEIGHT
        - (CARD_HEIGHT.get() + gfx::CHAR_ADVANCE_H.get().get() * 4)
    ) / 2;
    unscaled::Rect {
        x: X(X_OFFSET),
        y: Y(Y_OFFSET),
        w: W(command::WIDTH - X_OFFSET * 2),
        h: H(command::HEIGHT - Y_OFFSET * 2),
    }
};

const CPU_SELECTING_WINDOW: unscaled::Rect = {
    const OFFSET: unscaled::Inner = 128 - 16;
    unscaled::Rect {
        x: X(OFFSET),
        y: Y(OFFSET),
        w: W(command::WIDTH - OFFSET * 2),
        h: H(command::HEIGHT - OFFSET * 2),
    }
};

const CPU_ASKING_WINDOW: unscaled::Rect = {
    const OFFSET: unscaled::Inner = 128 - 16;
    unscaled::Rect {
        x: X(OFFSET),
        y: Y(OFFSET),
        w: W(command::WIDTH - OFFSET * 2),
        h: H(command::HEIGHT - OFFSET * 2),
    }
};

const CPU_SUCCESFUL_ASK_WINDOW: unscaled::Rect = {
    const OFFSET: unscaled::Inner = 128 - 16;
    unscaled::Rect {
        x: X(OFFSET),
        y: Y(OFFSET),
        w: W(command::WIDTH - OFFSET * 2),
        h: H(command::HEIGHT - OFFSET * 2),
    }
};

const CPU_SUCCESFUL_FISH_WINDOW: unscaled::Rect = {
    const OFFSET: unscaled::Inner = 128 - 16;
    unscaled::Rect {
        x: X(OFFSET),
        y: Y(OFFSET),
        w: W(command::WIDTH - OFFSET * 2),
        h: H(command::HEIGHT - OFFSET * 2),
    }
};

const CPU_TWO_FISTED_FISHERMAN_WINDOW: unscaled::Rect = {
    const OFFSET: unscaled::Inner = 128 - 16;
    unscaled::Rect {
        x: X(OFFSET),
        y: Y(OFFSET),
        w: W(command::WIDTH - OFFSET * 2),
        h: H(command::HEIGHT - OFFSET * 2),
    }
};

const DEAD_IN_THE_WATER_WINDOW: unscaled::Rect = {
    const OFFSET: unscaled::Inner = 128 - 16;
    unscaled::Rect {
        x: X(OFFSET),
        y: Y(OFFSET),
        w: W(command::WIDTH - OFFSET * 2),
        h: H(command::HEIGHT - OFFSET * 2),
    }
};

const PLAYER_PLAY_ANYTIME_WINDOW: unscaled::Rect = {
    const OFFSET: unscaled::Inner = 64;
    unscaled::Rect {
        x: X(OFFSET),
        y: Y(OFFSET),
        w: W(command::WIDTH - OFFSET * 2),
        h: H(command::HEIGHT - OFFSET * 2),
    }
};

const CARD_VIEWING_WINDOW: unscaled::Rect = {
    const OFFSET: unscaled::Inner = 64 + 16 + 8;
    unscaled::Rect {
        x: X(OFFSET * 2),
        y: Y(OFFSET),
        w: W(command::WIDTH - OFFSET * 4),
        h: H(command::HEIGHT - OFFSET * 2),
    }
};

const CARD_VIEWING_TEXT_WH: unscaled::WH = unscaled::WH {
    w: CARD_WIDTH,
    h: H(CARD_VIEWING_WINDOW.h.0 - (WINDOW_CONTENT_OFFSET.h.0 * 2 + CARD_HEIGHT.0)),
};

const MESSAGE_WINDOW: unscaled::Rect = {
    const OFFSET: unscaled::Inner = 64;
    unscaled::Rect {
        x: X(OFFSET),
        y: Y(OFFSET),
        w: W(command::WIDTH - OFFSET * 2),
        h: H(command::HEIGHT - OFFSET * 2),
    }
};

const MESSAGE_WINDOW_WIDTH_IN_CHARS: usize = (
    (MESSAGE_WINDOW.w.get() - (WINDOW_CONTENT_OFFSET.w.get() * 2))
    / gfx::CHAR_ADVANCE_W.get().get()
) as usize;

const NO_FISHING_WINDOW: unscaled::Rect = {
    const WIN_W: unscaled::Inner = ((CARD_WIDTH.get() * 3) / 2) * 3
    + WINDOW_CONTENT_OFFSET.w.get() * 2;

    const WIN_H: unscaled::Inner = CARD_HEIGHT.get()
    + WINDOW_CONTENT_OFFSET.h.get() * 2;

    unscaled::Rect {
        x: X((command::WIDTH - WIN_W) / 2),
        y: Y((command::HEIGHT - WIN_H) / 2),
        w: W(WIN_W),
        h: H(WIN_H),
    }
};