use models::{Card, Hand, DECK_SIZE};
use platform_types::{
    command,
    unscaled::{self, X, Y, XY, W, H, x_const_add_w, w_const_sub, y_const_add_h, h_const_sub},
    CARD_WIDTH,
    CARD_HEIGHT
};
use xs::{Xs, Seed};

pub const DECK_XY: XY = XY {
    x: X((command::WIDTH - CARD_WIDTH.get()) / 2),
    y: Y((command::HEIGHT - CARD_HEIGHT.get()) / 2),
};

pub const PLAYER_BASE_XY: XY = XY {
    x: X(CARD_WIDTH.get() * 5 / 4),
    y: Y(command::HEIGHT - CARD_HEIGHT.get()),
};

pub const CPU1_BASE_XY: XY = XY {
    x: X(0),
    y: Y(CARD_HEIGHT.get() / 2),
};

pub const CPU2_BASE_XY: XY = XY {
    x: X(CARD_WIDTH.get() * 5 / 4),
    y: Y(0),
};

pub const CPU3_BASE_XY: XY = XY {
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

#[derive(Clone, Copy)]
pub enum HandId {
    Player,
    Cpu1,
    Cpu2,
    Cpu3,
}

// TODO macro for this, I guess?
impl HandId {
    pub const ALL: [HandId; 4] = [
        HandId::Player, 
        HandId::Cpu1, 
        HandId::Cpu2, 
        HandId::Cpu3,
    ];
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
}

impl State {
    pub fn new(seed: Seed) -> State {
        use HandId::*;

        let mut rng = xs::from_seed(seed);

        let mut state = State {
            rng,
            deck: Hand::fresh_deck(&mut rng),
            .. <_>::default()
        };

        for card_i in 0..INITIAL_HAND_SIZE {
            for id in HandId::ALL {
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
                                .saturating_mul(2),
                        };

                        break;
                    }
                }
            }
        }

        state
    }

    pub fn tick(&mut self) {
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
