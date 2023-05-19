use memories::Memories;
use models::{Basket, Card, CardIndex, CpuId, Hand, HandId, Rank, Suit, DECK_SIZE, get_rank};
use gfx::{Commands, CHEVRON_H, WINDOW_CONTENT_OFFSET};
use platform_types::{
    command,
    unscaled::{self, X, Y, XY, W, H, Rect, x_const_add_w, w_const_sub},
    Button,
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
pub enum AfterDiscard {
    #[default]
    Nothing,
    BackToSelecting(HandId),
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


#[derive(Copy, Clone, Default, Debug)]
pub struct PlayerSelection {
    target: CpuId,
    card: AnytimeCard,
    declined: bool,
    viewing: Option<Card>,
    rank: Rank,
}

#[derive(Clone, Debug)]
pub enum Menu {
    PlayerTurn { selected: CardIndex },
    CpuTurn{ id: CpuId },
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
        }
    }
}

#[derive(Clone, Debug, Default)]
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
    pub fn new(_seed: Seed) -> State {
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
                macro_rules! back_to_selecting {
                    ($id: ident) => ({
                        let hand = match $id {
                            HandId::Player => &mut self.cards.player,
                            HandId::Cpu1 => &mut self.cards.cpu1,
                            HandId::Cpu2 => &mut self.cards.cpu2,
                            HandId::Cpu3 => &mut self.cards.cpu3,
                        };

                        match CpuId::try_from($id) {
                            Err(_) => match self.menu {
                                Menu::PlayerTurn {
                                    ref mut selected,
                                } => {
                                    *selected = hand.len() - 1;
                                },
                                _ => {},
                            },
                            Ok(cpu_id) => match self.menu {
                                Menu::CpuTurn {
                                    id,
                                } if cpu_id == id => {
                                    dbg!("TODO is this case useful?");
                                },
                                _ => {},
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

use ui::{Id::*};

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

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
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

fn can_and_should_play_two_fisted_fisherman(
    memories: &memories::Memories,
    hand: &Hand,
    source: CpuId,
    active_count: ActiveCardCount,
) -> bool {
    if let ActiveCardCount::VeryFew = active_count {
        return true
    }

    let targets = HandId::from(source).besides();

    // Note: It is not fair to read other players' memories.
    let memory = memories.memory(source);

    // TODO? Does this capture the criteria we want to capture?
    for rank in Rank::ALL {
        for suit in Suit::ALL {
            if hand.contains(models::fish_card(rank, suit)) {
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

    // DRAW

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

        if let Some((selected, player_card)) = selected
            .and_then(|selected| hand.get(selected).map(|card| (selected, card)))
        {
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
        }
    }

    // UPDATE

    match state.menu {
        Menu::PlayerTurn {
            selected,
        } => {
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
                    let _player_card = state.cards.player.get(selected)
                        .expect("selected index should always be valid");
                    state.menu = Menu::player(selected);
                    dbg!("TODO: Note attempt to win here");
                }
            } else {
                // do nothing
            }
        },
        _ => { dbg!(&state.menu); },
    };
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

fn next_turn_menu(mut id: CpuId, _player_hand: &Hand) -> Menu {
    match id.next() {
        Some(next_id) => Menu::CpuTurn{
            id: next_id.into(),
        },
        None => Menu::player(
            CardIndex::default()
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

const PLAYER_TWO_FISTED_FISHERMAN_WINDOW: unscaled::Rect = {
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