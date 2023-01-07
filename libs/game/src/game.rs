use memories::Memories;
use models::{Basket, Card, CardIndex, CpuId, Hand, HandId, Suit, Rank, Zinger, DECK_SIZE, get_rank, zingers};
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
    AddToHand(HandId),
    PerformGameWarden,
    AddToDiscard,
    AnimateBackToHand(HandId),
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

    fn between_turns(next_id: HandId) -> Self {
        Menu::BetweenTurns {
            next_id,
            player_selection: PlayerSelection::default(),
        }
    }
}

#[derive(Clone)]
pub enum PlayerMenu {
    Selecting { sub_menu: PlayerSubMenu },
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
        PlayerMenu::Selecting { sub_menu: <_>::default() }
    }
}

#[derive(Clone, Default)]
pub enum PlayerSubMenu {
    #[default]
    Root,
    Anytime(PlayerSelection, AvailablePlayAnytime),
    Message(Vec<u8>),
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
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ActiveCardCount {
    Several,
    VeryFew
}

#[derive(Clone, Default)]
pub struct State {
    pub rng: Xs,
    pub cards: Cards,
    pub animations: Animations,
    pub menu: Menu,
    pub ctx: ui::Context,
    pub memories: Memories,
    pub has_started: bool,
}

impl State {
    pub fn new(seed: Seed) -> State {
        const INITIAL_HAND_SIZE: u8 = 8;//16;
        // For debugging: {
        // Gives player multiple zingers. (8)
        //let seed = [150, 148, 11, 45, 255, 227, 216, 65, 225, 81, 35, 202, 235, 145, 4, 62];
        // Gives Cpu1 the game warden (8)
        //let seed = [168, 63, 217, 43, 183, 228, 216, 65, 56, 191, 2, 192, 83, 145, 4, 62];
        // Gives player glass bottom boat. (8)
        //let seed = [233, 217, 2, 79, 186, 228, 216, 65, 146, 77, 106, 40, 81, 145, 4, 62];
        // Gives player the game warden and glass bottom boat. (16)
        //let seed = [162, 35, 66, 102, 63, 230, 216, 65, 211, 81, 226, 193, 15, 144, 4, 62];
        // Gives Cpu2 the dead scuba diver and no fishing. (8)
        //let seed = [146, 115, 135, 54, 37, 236, 216, 65, 70, 182, 129, 14, 50, 139, 4, 62];
        // Gives player the net and no fishing. (8)
        let seed = [130, 162, 218, 177, 150, 236, 216, 65, 146, 44, 249, 132, 212, 138, 4, 62];
        // }

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
                    },
                    AnimationAction::PerformGameWarden => {
                        // TODO Animate all cards in deck moving to random targets
                        // away from, then back to DECK_XY.

                        self.cards.deck.push(anim.card);
                        self.cards.deck.shuffle(&mut self.rng);
                    }
                    AnimationAction::AddToDiscard => {
                        self.cards.discard.push(anim.card);

                        speaker.request_sfx(SFX::CardPlace);
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
        Cpu1,
        Cpu2,
        Cpu3,
        AskSuit(Suit),
        AskSubmit,
        AnytimeCard,
        AnytimeSubmit,
        RankSelect,
        NoFishingSubmit,
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

#[derive(Clone, Copy)]
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

#[derive(Clone, Copy)]
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
    hand.push(models::fish_card(Rank::DOGFISH, Suit::Yellow));
    hand.push(models::fish_card(Rank::DOGFISH, Suit::Purple));
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

    hand.push(models::fish_card(Rank::DOGFISH, Suit::Yellow));
    hand.push(models::fish_card(Rank::DOGFISH, Suit::Purple));

    assert_eq!(
        find_almost_complete_baskets(&hand),
        None,
    );
}

#[test]
fn find_almost_complete_baskets_returns_none_on_this_previously_misbehaving_example() {
    let mut hand = Hand::default();
    hand.push(models::fish_card(Rank::DOGFISH, Suit::Yellow));

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
    GameWarden(HandId),
    GlassBottomBoat(HandId),
    DeadScubaDiver(AlmostCompleteBasket, CardIndex)
}

#[derive(Copy, Clone)]
struct AnytimePlay {
    // Arguably we don't actually want to be able to represent a player targeting
    // themselves. But maybe we won't make those errors in practice.
    source: CpuId,
    selection: AnytimePlaySelection,
}

fn anytime_play(
    rng: &mut Xs,
    cards: &Cards,
    memories: &Memories,
    next_id: HandId,
) -> Option<AnytimePlay> {
    let mut cpu_ids = CpuId::ALL;

    xs::shuffle(rng, &mut cpu_ids);

    for cpu_id in cpu_ids {
        let hand_id = cpu_id.into();
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
                                source: cpu_id,
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
                                    source: cpu_id,
                                    selection: AnytimePlaySelection::$card(
                                        target
                                    ),
                                });
                            }
                        }
                    }

                    if next_id != hand_id {
                        if let Some(_) = memories
                            .memory(cpu_id)
                            .likely_to_fill_basket_soon(next_id) {
                            return Some(AnytimePlay {
                                source: cpu_id,
                                selection: AnytimePlaySelection::$card(
                                    next_id
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
                let hand = cards.hand(cpu_id.into());
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
                            source: cpu_id,
                            selection: AnytimePlaySelection::DeadScubaDiver(
                                best_basket.expect("There should be a basket available!"),
                                card_i
                            ),
                        });
                    }
                }
            }
        }
    }

    None
}

fn discard_glass_bottom_boat(
    cards: &mut Cards,
    animations: &mut Animations,
    source: HandId,
) {
    discard_given_card(cards, animations, source, zingers::GLASS_BOTTOM_BOAT)
}

fn discard_no_fishing(
    cards: &mut Cards,
    animations: &mut Animations,
    source: HandId,
) {
    discard_given_card(cards, animations, source, zingers::NO_FISHING)
}

fn discard_given_card(
    cards: &mut Cards,
    animations: &mut Animations,
    source: HandId,
    card: Card,
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
                action: AnimationAction::AddToDiscard,
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
            Cpu1
            | Cpu2
            | Cpu3
            | RankSelect => Some(Section::Target),
            AnytimeSubmit => Some(Section::Submit),
            Zero
            | AskSuit(_)
            | AskSubmit 
            | NoFishingSubmit => None,
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
                | AnytimeCard::GlassBottomBoat => Cpu1,
                AnytimeCard::DeadScubaDiver => RankSelect,
            },
            Section::Submit => AnytimeSubmit,
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

        return if do_button(
            &mut group,
            ButtonSpec {
                id: AnytimeSubmit,
                rect: fit_to_rest_of_window(
                    submit_base_xy,
                    CARD_VIEWING_WINDOW,
                ),
                text: b"Done",
            }
        ) {
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
                &[AnytimeCard]
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
                    models::fish_card(player_selection.rank, suit),
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
                    &[RankSelect]
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
            id: AnytimeSubmit,
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
            Cpu1
            | Cpu2
            | Cpu3
            | RankSelect => Some(Section::Target),
            AnytimeSubmit => Some(Section::Submit),
            Zero
            | AskSuit(_)
            | AskSubmit
            | NoFishingSubmit => None,
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
                | AnytimeCard::GlassBottomBoat => Cpu1,
                AnytimeCard::DeadScubaDiver => RankSelect,
            },
            Section::Submit => AnytimeSubmit,
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
    rank: Rank,
    suit: Suit,
    active_count: ActiveCardCount,
) -> bool {
    if let ActiveCardCount::VeryFew = active_count {
        return true
    }

    hand.contains(models::fish_card(rank, suit))
    && memory.is_likely_to_fill_rank_soon(target, rank)
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

        let selected = match state.menu {
            Menu::PlayerTurn {
                selected,
                menu: PlayerMenu::Selecting { .. }
            } => Some(selected),
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
        Menu::BetweenTurns {
            next_id,
            ref mut player_selection,
        } => {
            macro_rules! start_next_turn {
                () => {
                    state.menu = match next_id {
                        HandId::Player => Menu::player(CardIndex::default()),
                        HandId::Cpu1 => Menu::CpuTurn {
                            id: CpuId::One,
                            menu: <_>::default(),
                        },
                        HandId::Cpu2 => Menu::CpuTurn {
                            id: CpuId::Two,
                            menu: <_>::default(),
                        },
                        HandId::Cpu3 => Menu::CpuTurn {
                            id: CpuId::Three,
                            menu: <_>::default(),
                        },
                    };
                }
            }

            if let (Some(available), false) = (
                AvailablePlayAnytime::in_hand(state.cards.hand(HandId::Player)),
                player_selection.declined
            ) {
                if let AnytimeOutcome::Done = do_play_anytime_menu(
                    new_group!(),
                    &mut state.cards,
                    &mut state.animations,
                    &mut state.rng,
                    player_selection,
                    available,
                ) {
                    start_next_turn!();
                }
            } else if let Some(AnytimePlay { source, selection })
            = anytime_play(
                &mut state.rng,
                &state.cards,
                &state.memories,
                next_id
            ) {
                match selection {
                    AnytimePlaySelection::GameWarden(target) => {
                        if let Some(()) = perform_game_warden(
                            &mut state.cards,
                            &mut state.animations,
                            &mut state.rng,
                            Targeting {
                                source: source.into(),
                                target,
                            },
                        ) {
                            start_next_turn!();
                        } else {
                            debug_assert!(false, "perform_game_warden failed");
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
                            target: in_front_of(source.into()),
                            action: AnimationAction::AnimateBackToHand(target),
                            .. <_>::default()
                        });

                        discard_glass_bottom_boat(
                            &mut state.cards,
                            &mut state.animations,
                            source.into()
                        );
                    },
                    AnytimePlaySelection::DeadScubaDiver(almost_basket, scuba_i) => {
                        play_dead_scuba_diver(
                            &mut state.cards,
                            source.into(),
                            almost_basket,
                            scuba_i
                        );
                    }
                }
            } else {
                start_next_turn!();
            }
        },
        Menu::PlayerTurn {
            selected,
            ref mut menu,
        } => {
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
                            PlayerSubMenu::Root => {
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
                                        if let Some(zinger) = models::get_zinger(player_card) {
                                            match zinger {
                                                Zinger::TheGameWarden => {
                                                    *sub_menu = PlayerSubMenu::Anytime(
                                                        <_>::default(),
                                                        AvailablePlayAnytime::game_warden(selected),
                                                    );
                                                },
                                                Zinger::GlassBottomBoat => {
                                                    *sub_menu = PlayerSubMenu::Anytime(
                                                        <_>::default(),
                                                        AvailablePlayAnytime::glass_bottom_boat(selected),
                                                    );
                                                },
                                                Zinger::DeadScubaDiver => {
                                                    if let Some(almost_complete_baskets) = find_almost_complete_baskets(
                                                        &state.cards.player
                                                    ) {
                                                        *sub_menu = PlayerSubMenu::Anytime(
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
                                                        *sub_menu = PlayerSubMenu::Message(vec);
                                                    }
                                                },
                                                _ => {
                                                    // TODO add specific menus for each zinger
                                                },
                                            }
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
                            PlayerSubMenu::Anytime(
                                ref mut player_selection,
                                available,
                            ) => {
                                if player_selection.declined {
                                    *sub_menu = PlayerSubMenu::Root;
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
                                    *sub_menu = PlayerSubMenu::Root;
                                }
                            },
                            PlayerSubMenu::Message(ref mut message) => {
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
                                    *sub_menu = PlayerSubMenu::Root;
                                }
                            },
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
                            &[
                                AskSuit(Suit::ALL[0]),
                                AskSuit(Suit::ALL[1]),
                                AskSuit(Suit::ALL[2]),
                                AskSuit(Suit::ALL[3]),
                                AskSuit(Suit::ALL[4])
                            ]
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
                                id: AskSubmit,
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
                                rank,
                                question.suit,
                                state.cards.active_count(),
                            ) {
                                // TODO have Cpu player play "No Fishing".
                                todo!();
                            } else {
                                let player_len = state.cards.player.len();
                                let target_hand = state.cards.hand_mut(question.target);

                                state.memories.asked_for(
                                    HandId::Player,
                                    rank,
                                    question.suit
                                );

                                let target_card = models::fish_card(rank, question.suit);

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
                            }
                        } else if input.pressed_this_frame(Button::B) {
                            state.menu = Menu::player(selected);
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
                                Cpu1 | Cpu2| Cpu3 => Some(Section::Target),
                                AskSuit(_) => Some(Section::Suit),
                                AskSubmit => Some(Section::Submit),
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
                                // TODO? remove now unueeded ui::Id varaints.
                                Section::Target => Cpu1,
                                Section::Suit => AskSuit(Suit::ALL[0]),
                                Section::Submit => AskSubmit,
                            });
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
                        || input.pressed_this_frame(Button::B) {
                            let target_card = models::fish_card(rank, question.suit);

                            state.menu = if let Some(true) = drew
                                .map(|card| card == target_card) {
                                state.memories.fished_for(
                                    HandId::Player,
                                    rank,
                                    question.suit
                                );
                                Menu::player(
                                    state.cards.player.len().saturating_sub(1)
                                )
                            } else {
                                Menu::between_turns(
                                    HandId::Cpu1
                                )
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

                        state.menu = Menu::between_turns(
                            HandId::Cpu1
                        );

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
        Menu::CpuTurn {
            id,
            ref mut menu,
        } => match menu {
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
                    if let Some((rank, suit, target)) = state.memories.memory(id)
                        .informed_question(hand, hand_id) {
                        let mut question = Question::default();

                        question.suit = suit;
                        question.target = target;

                        *menu = CpuMenu::Asking(
                            rank,
                            question,
                        );
                    }

                    // TODO Play Zingers sometimes.

                    if let CpuMenu::Selecting = *menu {
                        // TODO? randomize order through the cards here to make Cpu
                        // player less predictable?
                        for card in hand.iter() {
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
                                break
                            }
                        }

                        if let CpuMenu::Selecting = *menu {
                            *menu = CpuMenu::DeadInTheWater;
                        }
                    }
                }
            },
            CpuMenu::Asking(rank, ref mut question) => {
                macro_rules! handle_negative_response {
                    () => {
                        let rank = *rank;
                        let target_card = models::fish_card(rank, question.suit);
                        let my_len = state.cards.hand(id.into()).len();

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
                                state.memories.fished_for(id.into(), rank, question.suit);

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
                }
                macro_rules! handle_ask {
                    () => {
                        let rank = *rank;

                        state.memories.asked_for(id.into(), rank, question.suit);

                        let target_card = models::fish_card(rank, question.suit);
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

                            state.menu = Menu::CpuTurn{
                                id,
                                menu: CpuMenu::WaitingForSuccesfulAsk,
                            };
                        } else {
                            handle_negative_response!();
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
                                id: NoFishingSubmit,
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
                                question.target
                            );
                            handle_negative_response!();
                        } else if input.pressed_this_frame(Button::B) {
                            handle_ask!();
                        }

                        group.ctx.set_next_hot(NoFishingSubmit);
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
                            *rank,
                            question.suit,
                            state.cards.active_count(),
                        ) {
                            discard_no_fishing(
                                &mut state.cards,
                                &mut state.animations,
                                question.target
                            );
                            handle_negative_response!();
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

                    state.menu = next_turn_menu(id, &state.cards.player);

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

fn next_turn_menu(mut id: CpuId, player_hand: &Hand) -> Menu {
    match id.next() {
        Some(next_id) => Menu::between_turns(
            next_id.into()
        ),
        None => Menu::between_turns(
            HandId::Player
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

struct Targeting {
    source: HandId,
    target: HandId,
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

    discard_given_card(
        cards,
        animations,
        source.into(),
        zingers::THE_GAME_WARDEN,
    );

    None
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
        &[Cpu1, Cpu2, Cpu3]
    );
}

const CARD_QUICK_SELECT_WH: unscaled::WH = unscaled::WH {
    w: CARD_WIDTH,
    h: unscaled::H(
        CARD_HEIGHT.get()
        + CHEVRON_H.get() * 2
    ),
};

const ASKING_WINDOW: unscaled::Rect = {
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
    const OFFSET: unscaled::Inner = 8;

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