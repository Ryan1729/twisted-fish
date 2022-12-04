use models::{Card, CardIndex, Hand, Suit, Rank, DECK_SIZE, get_rank};
use gfx::{Commands, WINDOW_CONTENT_OFFSET};
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

const DECK_XY: XY = XY {
    x: X((command::WIDTH - CARD_WIDTH.get()) / 2),
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

const INITIAL_HAND_SIZE: u8 = 8;

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
pub enum AnimationAction {
    #[default]
    DoNothing,
    AddToHand(HandId)
}

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum HandId {
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
        b"Player",
        b"Cpu 1",
        b"Cpu 2",
        b"Cpu 3",
    ];

    fn besides(self) -> [HandId; (Self::COUNT - 1) as usize] {
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
}

#[repr(u8)]
#[derive(Clone, Copy, Default, PartialEq, Eq)]
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

#[derive(Default)]
pub enum Facing {
    #[default]
    Down,
    Up,
}

impl HandId {
    pub fn facing(self) -> Facing {
        match self {
            HandId::Player => Facing::Up,
            HandId::Cpu1
            | HandId::Cpu2
            | HandId::Cpu3 => Facing::Down,
        }
    }
}

mod question {
    use super::*;

    #[derive(Clone)]
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
                models::ranks::TEXT[usize::from(rank as u8)]
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
                models::ranks::TEXT[usize::from(rank as u8)]
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
                models::ranks::TEXT[usize::from(rank as u8)]
            );

            self.description.push(b' ');

            if let Some(card) = drew {
                let target_card = models::fish_card(rank, self.suit);
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

#[derive(Clone)]
pub enum Menu {
    PlayerTurn { selected: CardIndex, menu: PlayerMenu },
    CpuTurn{ id: CpuId, menu: CpuMenu }
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
            menu: PlayerMenu::Selecting,
        }
    }
}

#[derive(Clone)]
pub enum PlayerMenu {
    Selecting,
    Asking {
        used: Card,
        question: Question,
    },
    Fished{
        used: Card,
        question: Question,
        drew: Option<Card>
    },
}

impl Default for PlayerMenu {
    fn default() -> PlayerMenu {
        PlayerMenu::Selecting
    }
}

#[derive(Clone, Default)]
pub enum CpuMenu {
    #[default]
    Selecting,
    Asking(Rank, Question),
    DeadInTheWater,
    WaitingForSuccesfulAsk,
    WaitingWhenGotWhatWasFishingFor,
}

// TODO? Tighter representation that still allows representing Dead Scuba Diver
// but doesn't allow non-matched cards?
type Basket = [Card; Suit::COUNT as usize];

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
}

#[derive(Clone, Default)]
pub struct State {
    pub rng: Xs,
    pub cards: Cards,
    pub animations: Animations,
    pub menu: Menu,
    pub ctx: ui::Context,
}

impl State {
    pub fn new(seed: Seed) -> State {
        let mut rng = xs::from_seed(seed);

        let mut state = State {
            rng,
            cards: Cards {
                deck: Hand::fresh_deck(&mut rng),
                .. <_>::default()
            },
            // TODO Randomize starting turn
            .. <_>::default()
        };

        for card_i in 0..INITIAL_HAND_SIZE {
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
                            // TODO? animate gathering together and heading to a
                            // separate pile? Or maybe poofing in an expolsion of
                            // particles?
                            for card in basket {
                                baskets.push(card);
                            }
                        }

                        speaker.request_sfx(SFX::CardPlace);

                        match CpuId::try_from(id) {
                            Err(_) => match self.menu {
                                Menu::PlayerTurn {
                                    ref mut selected,
                                    menu: _
                                } => {
                                    *selected = hand.len() - 1;
                                },
                                _ => {},
                            },
                            Ok(cpu_id) => match self.menu {
                                Menu::CpuTurn {
                                    id,
                                    ref mut menu,
                                } if cpu_id == id
                                && matches!(
                                    *menu,
                                    CpuMenu::WaitingForSuccesfulAsk
                                    | CpuMenu::WaitingWhenGotWhatWasFishingFor
                                ) => {
                                    *menu = CpuMenu::Selecting;
                                },
                                _ => {},
                            }
                        }
                    }
                }
            }
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
        Cpu1,
        Cpu2,
        Cpu3,
        AskSuit(Suit),
        AskSubmit,
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
        ids: &[Id],
    ) {
        let mut highlighted = gfx::Highlighted::No;
        for &id in ids {
            if group.ctx.active == id
            || group.ctx.hot == id {
                group.ctx.set_next_hot(id);
                highlighted = gfx::Highlighted::Yes;
            }
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

pub fn update_and_render(
    commands: &mut Commands,
    state: &mut State,
    input: Input,
    speaker: &mut Speaker
) {
    state.ctx.frame_init();

    state.tick(speaker);

    if !state.cards.deck.is_empty() {
        commands.draw_card_back(DECK_XY);
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

        let selected = match state.menu {
            Menu::PlayerTurn { selected, menu: PlayerMenu::Selecting } => Some(selected),
            _ => None,
        };
        for (i, card) in hand.enumerated_iter() {
            if selected == Some(i) { continue }

            commands.draw_card(
                card,
                get_card_position(spread(id), len, i)
            );
        }
    }

    match state.menu {
        Menu::PlayerTurn {
            selected,
            ref mut menu,
        } => {
            let hand = &state.cards.player;
            let len = hand.len();
            // TODO Handle player being "dead in the water" (None case).
            if let Some(player_card) = hand.get(selected) {
                let id = HandId::Player;
                match menu {
                    PlayerMenu::Selecting => {
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

                        if input.pressed_this_frame(Button::LEFT) {
                            state.menu = Menu::player(
                                if selected > 0 {
                                    selected - 1
                                } else {
                                    state.cards.player.len().saturating_sub(1)
                                }
                            );
                        } else if input.pressed_this_frame(Button::RIGHT) {
                            state.menu = Menu::player(
                                if selected < state.cards.player.len().saturating_sub(1) {
                                    selected + 1
                                } else {
                                    0
                                }
                            );
                        } else if input.pressed_this_frame(Button::A) {
                            if !state.cards.player.is_empty() {
                                let player_card = state.cards.player.get(selected)
                                    .expect("selected index should always be valid");
                                if let Some(_zinger) = models::get_zinger(player_card) {
                                    // TODO probably add specific menus for each zinger
                                } else {
                                    state.menu = Menu::PlayerTurn {
                                        selected,
                                        menu: PlayerMenu::Asking{
                                            used: player_card,
                                            question: Default::default(),
                                        },
                                    };
                                    state.ctx.set_next_hot(Cpu1);
                                }
                            }
                        } else {
                            // do nothing
                        }
                    },
                    PlayerMenu::Asking {
                        used,
                        ref mut question,
                    } => {
                        let used = *used;
                        // TODO? handle the Net here, or elsewhere?
                        let rank = models::get_rank(used)
                            .expect("Asking used card should always have a rank!");

                        commands.draw_nine_slice(gfx::NineSlice::Window, ASKING_WINDOW);

                        let base_xy = ASKING_WINDOW.xy() + WINDOW_CONTENT_OFFSET;

                        let card_xy = base_xy;

                        commands.draw_card(
                            used,
                            card_xy
                        );

                        let target_xy = card_xy + CARD_WIDTH;

                        commands.print_centered(
                            HandId::TEXT[question.target as u8 as usize],
                            Rect::xy_wh(
                                target_xy + ASKING_TARGET_WH.h,
                                ASKING_TARGET_WH,
                            ),
                            WHITE,
                        );

                        let mut group = ui::Group {
                            commands,
                            ctx: &mut state.ctx,
                            input,
                            speaker,
                        };

                        let target_quick_select_rect = Rect::xy_wh(
                            target_xy,
                            ASKING_TARGET_WH + (ASKING_TARGET_WH.h * 3),
                        );

                        ui::draw_quick_select(
                            &mut group,
                            target_quick_select_rect,
                            &[Cpu1, Cpu2, Cpu3]
                        );

                        let suit_base_xy = target_xy + ASKING_TARGET_WH.w;

                        let suit_quick_select_rect = Rect::xy_wh(
                            suit_base_xy,
                            ASKING_TARGET_WH + (ASKING_SUIT_WH.h * 4),
                        );

                        group.commands.print_centered(
                            Suit::TEXT[question.suit as u8 as usize],
                            Rect::xy_wh(
                                suit_base_xy + (ASKING_SUIT_WH.h * 2),
                                ASKING_SUIT_WH,
                            ),
                            WHITE,
                        );

                        ui::draw_quick_select(
                            &mut group,
                            suit_quick_select_rect,
                            &[
                                AskSuit(Suit::ALL[0]),
                                AskSuit(Suit::ALL[1]),
                                AskSuit(Suit::ALL[2]),
                                AskSuit(Suit::ALL[3]),
                                AskSuit(Suit::ALL[4])
                            ]
                        );

                        let description_base_rect = unscaled::Rect::xy_wh(
                            base_xy + ASKING_SUIT_WH.h * unscaled::Inner::from(Suit::COUNT),
                            unscaled::WH {
                                w: ASKING_WINDOW.w,
                                h: ASKING_SUIT_WH.h,
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

                        let submit_base_xy = suit_base_xy + ASKING_TARGET_WH.w;

                        if do_button(
                            &mut group,
                            ButtonSpec {
                                id: AskSubmit,
                                rect: fit_to_rest_of_window(
                                    submit_base_xy,
                                    ASKING_WINDOW,
                                ),
                                text: b"Submit",
                            }
                        ) {
                            let player_len = state.cards.player.len();
                            let target_hand = state.cards.hand_mut(question.target);
                            let target_card = models::fish_card(rank, question.suit);

                            let mut found = None;
                            // TODO? randomize order here to make it harder to learn their
                            // whole hand with glass bottom boat?
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

                                *menu = PlayerMenu::Selecting;
                            } else {
                                let drew = state.cards.deck.draw();

                                *menu = PlayerMenu::Fished{
                                    used,
                                    question: core::mem::take(question),
                                    drew,
                                };

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
                        } else if input.pressed_this_frame(Button::B) {
                            state.menu = Menu::PlayerTurn {
                                selected,
                                menu: PlayerMenu::Selecting,
                            };
                        } else if let Some(dir) = input.dir_pressed_this_frame() {
                            const GRID_W: usize = 3;
                            const GRID_H: usize = 5;
                            const GRID_LEN: usize = GRID_W * GRID_H;
                            const GRID: [ui::Id; GRID_LEN] = [
                                Cpu1, AskSuit(Suit::ALL[0]), AskSubmit,
                                Cpu2, AskSuit(Suit::ALL[1]), AskSubmit,
                                Cpu3, AskSuit(Suit::ALL[2]), AskSubmit,
                                Zero, AskSuit(Suit::ALL[3]), AskSubmit,
                                Zero, AskSuit(Suit::ALL[4]), AskSubmit,
                            ];

                            let old_id = state.ctx.hot;
                            let id_i = GRID.iter()
                                .position(|el| *el == old_id)
                                .unwrap_or_default();

                            let old_x = id_i % GRID_W;
                            let old_y = id_i / GRID_W;

                            let mut x = old_x;
                            let mut y = old_y;

                            macro_rules! new_id { () => (GRID[y * GRID_W + x]) }

                            while new_id!() == old_id || new_id!() == Zero {
                                match dir {
                                    Dir::Up => if y == 0 {
                                        y = GRID_H - 1;
                                    } else {
                                        y -= 1;
                                    },
                                    Dir::Down => if y >= GRID_H - 1 {
                                        y = 0;
                                    } else {
                                        y += 1;
                                    },
                                    Dir::Left => if x == 0 {
                                        x = GRID_W - 1;
                                    } else {
                                        x -= 1;
                                    },
                                    Dir::Right => if x >= GRID_W - 1 {
                                        x = 0;
                                    } else {
                                        x += 1;
                                    },
                                }

                                // Stop if we looped back to where started.
                                if x == old_x && y == old_y { break }
                            }

                            let id = new_id!();
                            state.ctx.set_next_hot(id);

                            match id {
                                Cpu1 => { question.target = HandId::Cpu1; },
                                Cpu2 => { question.target = HandId::Cpu2; },
                                Cpu3 => { question.target = HandId::Cpu3; },
                                AskSuit(suit) => { question.suit = suit; }
                                _ => {}
                            }
                        } else {
                            // do nothing
                        }
                    },
                    PlayerMenu::Fished{
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

                        let target_card = models::fish_card(rank, question.suit);

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
                        | input.pressed_this_frame(Button::B) {
                            let rank = models::get_rank(player_card)
                                .expect("Fished selected index should always have a rank!");

                            let target_card = models::fish_card(rank, question.suit);

                            state.menu = if let Some(true) = drew
                                .map(|card| card == target_card) {
                                Menu::player(
                                    state.cards.player.len().saturating_sub(1)
                                )
                            } else {
                                Menu::CpuTurn {
                                    id: <_>::default(),
                                    menu: <_>::default(),
                                }
                            };
                        }
                    }
                }
            }
        },
        Menu::CpuTurn {
            id,
            ref mut menu,
        } => match menu {
            CpuMenu::Selecting => {
                // Showing this avoids a flicker for the one frame the Cpu
                // is selecting when they stop waiting.
                // Maybe enforce that the Cpu windows must all be the same size?
                commands.draw_nine_slice(gfx::NineSlice::Window, CPU_SELECTING_WINDOW);

                let id = HandId::from(id);
                let hand = state.cards.hand(id);
                if hand.is_empty() {
                    *menu = CpuMenu::DeadInTheWater;
                } else {
                    for card in hand.iter() {
                        if let Some(rank) = models::get_rank(card) {
                            // TODO Decide whether to play this particular card or
                            // not, based on some memory of what cards are where.

                            let besides = HandId::besides(id);
                            let target_id = besides[
                                xs::range(&mut state.rng, 0..besides.len() as u32) as usize
                            ];

                            // TODO Decide what suit to ask for intelligently.
                            let suit = Suit::from_rng(&mut state.rng);

                            let mut question = Question::default();
                            question.target = target_id;
                            question.suit = suit;

                            *menu = CpuMenu::Asking(
                                rank,
                                question,
                            );
                            break
                        } else {
                            // TODO Play Zingers sometimes.
                        }
                    }

                    if let CpuMenu::Selecting = *menu {
                        *menu = CpuMenu::DeadInTheWater;
                    }
                }
            },
            CpuMenu::Asking(rank, ref mut question) => {
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

                let target_card = models::fish_card(*rank, question.suit);

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

                    state.menu = Menu::CpuTurn{
                        id,
                        menu: CpuMenu::WaitingForSuccesfulAsk,
                    };
                } else {
                    let card_option = state.cards.deck.draw();

                    if let Some(card) = card_option {
                        let at = DECK_XY;

                        let target = get_card_insert_position(
                            spread(id.into()),
                            my_len
                        );

                        state.animations.push(Animation {
                            card,
                            at,
                            target,
                            action: AnimationAction::AddToHand(id.into()),
                            .. <_>::default()
                        });

                        if card == target_card {
                            state.menu = Menu::CpuTurn{
                                id,
                                menu: CpuMenu::WaitingWhenGotWhatWasFishingFor,
                            };
                        } else {
                            state.menu = next_turn_menu(id, &state.cards.player);
                        }
                    } else {
                        state.menu = next_turn_menu(id, &state.cards.player);
                    }
                }
            },
            CpuMenu::DeadInTheWater => {
                // Just wait until player acknowledges turn.
                if input.pressed_this_frame(Button::A)
                | input.pressed_this_frame(Button::B) {
                    state.menu = next_turn_menu(id, &state.cards.player);
                }
            },
            // TODO? retain their target card for this message
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
            // TODO? retain their target card for this message
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
        }
    }
}

fn next_turn_menu(mut id: CpuId, player_hand: &Hand) -> Menu {
    match id.next() {
        Some(next_id) => Menu::CpuTurn{
            id: next_id,
            menu: CpuMenu::default(),
        },
        None => Menu::player(
            player_hand.len().saturating_sub(1)
        )
    }
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
            b"You asked for the Red Shrimp \nbut you didn't get it."
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

const ASKING_WINDOW: unscaled::Rect = {
    const OFFSET: unscaled::Inner = 8;
    unscaled::Rect {
        x: X(OFFSET),
        y: Y(OFFSET),
        w: W(command::WIDTH - OFFSET * 2),
        h: H(command::HEIGHT - OFFSET * 2),
    }
};

const ASKING_TARGET_WH: unscaled::WH = unscaled::WH {
    w: W(ASKING_WINDOW.w.get() / 3),
    h: H(ASKING_WINDOW.h.get() / 8),
};

const ASKING_SUIT_WH: unscaled::WH = ASKING_TARGET_WH;

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