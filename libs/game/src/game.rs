use models::{Card, CardIndex, Hand, DECK_SIZE};
use gfx::{Commands, WINDOW_CONTENT_OFFSET};
use platform_types::{
    command,
    unscaled::{self, X, Y, XY, W, H, x_const_add_w, w_const_sub},
    Button,
    Input,
    Speaker,
    SFX,
    CARD_WIDTH,
    CARD_HEIGHT
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

pub fn spread(id: HandId) -> Spread {
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
}

pub type Frames = u8;

#[derive(Clone, Copy, Default)]
pub struct Animation {
    pub delay: Frames,
    pub card: Card,
    pub at: XY,
    pub target: XY,
    pub action: AnimationAction,
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

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum HandId {
    Player,
    Cpu1,
    Cpu2,
    Cpu3,
}

// TODO macro for this, I guess?
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


#[derive(Copy, Clone)]
pub enum Menu {
    Selecting(CardIndex),
    Asking(CardIndex),
}

impl Default for Menu {
    fn default() -> Menu {
        Menu::Selecting(CardIndex::default())
    }
}

impl Menu {
    // We assume there will probably be cases where there isn't a selected card.
    // We'll see.
    pub fn selected(self) -> CardIndex {
        match self {
            Self::Selecting(selected)
            | Self::Asking(selected) => selected,
        }
    }
}

#[derive(Clone, Default)]
pub struct State {
    pub rng: Xs,
    pub deck: Hand,
    pub player: Hand,
    pub cpu1: Hand,
    pub cpu2: Hand,
    pub cpu3: Hand,
    pub animations: Animations,
    pub menu: Menu,
}

impl State {
    pub fn new(seed: Seed) -> State {
        let mut rng = xs::from_seed(seed);

        let mut state = State {
            rng,
            deck: Hand::fresh_deck(&mut rng),
            .. <_>::default()
        };

        for card_i in 0..INITIAL_HAND_SIZE {
            for (id_i, id) in HandId::ALL.into_iter().enumerate() {
                let card = match state.deck.draw() {
                    Some(card) => card,
                    None => continue,
                };
                let animations = &mut state.animations.0;
                for i in 0..animations.len() {
                    if animations[i].is_done() {
                        let target = match id {
                            // TODO different based on how many cards are in the
                            // hand already? Maybe account for any hand sorting?
                            HandId::Player => PLAYER_BASE_XY,
                            HandId::Cpu1 => CPU1_BASE_XY,
                            HandId::Cpu2 => CPU2_BASE_XY,
                            HandId::Cpu3 => CPU3_BASE_XY,
                        };

                        animations[i] = Animation {
                            card,
                            at: DECK_XY,
                            target,
                            action: AnimationAction::AddToHand(id),
                            delay: card_i
                                .saturating_mul(HandId::ALL.len() as u8)
                                .saturating_add(id_i as u8),
                        };

                        break;
                    }
                }
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

            let x_rate = 1;
            let y_rate = 1;
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
                            HandId::Player => &mut self.player,
                            HandId::Cpu1 => &mut self.cpu1,
                            HandId::Cpu2 => &mut self.cpu2,
                            HandId::Cpu3 => &mut self.cpu3,
                        };

                        hand.push(anim.card);

                        if let HandId::Player = id {
                            match self.menu {
                                Menu::Selecting(_) => {
                                    self.menu = Menu::Selecting(hand.len() - 1);
                                },
                                Menu::Asking(_) => {},
                            }
                        }

                        speaker.request_sfx(SFX::CardPlace);
                    }
                }
            }
        }
    }

    pub fn hand(&self, id: HandId) -> &Hand {
        match id {
            HandId::Player => &self.player,
            HandId::Cpu1 => &self.cpu1,
            HandId::Cpu2 => &self.cpu2,
            HandId::Cpu3 => &self.cpu3,
        }
    }
}

pub fn update_and_render(
    commands: &mut Commands,
    state: &mut State,
    input: Input,
    speaker: &mut Speaker
) {
    match state.menu {
        Menu::Selecting(selected) => {
            if input.pressed_this_frame(Button::LEFT) {
                state.menu = Menu::Selecting(
                    if selected > 0 {
                        selected - 1
                    } else {
                        0
                    }
                );
            } else if input.pressed_this_frame(Button::RIGHT) {
                state.menu = Menu::Selecting(
                    if selected < state.player.len() - 1 {
                        selected + 1
                    } else {
                        0
                    }
                );
            } else if input.pressed_this_frame(Button::A) {
                let player_card = state.player.get(selected)
                    .expect("selected index should always be valid");
                if let Some(_zinger) = models::get_zinger(player_card) {
                    // TODO probably add specific menus for each zinger
                } else {
                    state.menu = Menu::Asking(selected);
                }
            } else {
                // do nothing
            }
        },
        Menu::Asking(selected) => {
            if input.pressed_this_frame(Button::B) {
                state.menu = Menu::Selecting(selected);
            } else {
                // do nothing
            }
        }
    }

    state.tick(speaker);

    if !state.deck.is_empty() {
        commands.draw_card_back(DECK_XY);
    }

    for anim in state.animations.iter() {
        if anim.is_active() {
            commands.draw_card_back(anim.at);
        }
    }

    // Rev to put player cards on top.
    for id in HandId::CPUS.into_iter() {
        let hand = state.hand(id);
        let len = hand.len();

        for i in 0..len {
            commands.draw_card_back(
                get_card_position(spread(id), len, i)
            );
        }
    }

    'player_hand: {
        let id = HandId::Player;
        let hand = state.hand(id);
        let len = hand.len();

        if len == 0 {
            break 'player_hand
        }

        let selected = state.menu.selected();
        for (i, card) in hand.enumerated_iter() {
            if selected == i { continue }

            commands.draw_card(
                card,
                get_card_position(spread(id), len, i)
            );
        }

        let player_card = hand.get(selected)
            .expect("selected index should always be valid");

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

        match state.menu {
            Menu::Selecting(_) => {},
            Menu::Asking(_) => {
                commands.draw_nine_slice(ASKING_WINDOW);

                commands.draw_card(
                    player_card,
                    ASKING_WINDOW.xy() + WINDOW_CONTENT_OFFSET
                );
            },
        }
    }
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

const ASKING_WINDOW: unscaled::Rect = {
    const OFFSET: unscaled::Inner = 8;
    unscaled::Rect {
        x: X(OFFSET),
        y: Y(OFFSET),
        w: W(command::WIDTH - OFFSET * 2),
        h: H(command::HEIGHT - OFFSET * 2),
    }
};