use models::{Card, Hand, DECK_SIZE};
use platform_types::{
    command,
    unscaled::{self, X, Y, XY, W, H},
    CARD_WIDTH,
    CARD_HEIGHT
};
use xs::{Xs, Seed};

pub const DECK_XY: XY = XY {
    x: X((command::WIDTH - CARD_WIDTH.get()) / 2),
    y: Y((command::HEIGHT - CARD_HEIGHT.get()) / 2),
};

pub const PLAYER_BASE_XY: XY = XY {
    x: X(CARD_WIDTH.get()),
    y: Y(command::HEIGHT - CARD_HEIGHT.get()),
};

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

#[derive(Clone, Copy, Default)]
pub struct Animation {
    pub card: Card,
    pub at: XY,
    pub target: XY,
    pub action: AnimationAction,
}

impl Animation {
    fn is_done(&self) -> bool {
        self.at == self.target
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
}

#[derive(Clone, Default)]
pub struct State {
    pub rng: Xs,
    pub deck: Hand,
    pub player: Hand,
    pub animations: Animations,
}

impl State {
    pub fn new(seed: Seed) -> State {
        let mut rng = xs::from_seed(seed);

        let mut state = State {
            rng,
            deck: Hand::fresh_deck(&mut rng),
            .. <_>::default()
        };

        let hands = [HandId::Player];

        for _ in 0..INITIAL_HAND_SIZE {
            for id in hands {
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
                        };

                        animations[i] = Animation {
                            card,
                            at: DECK_XY,
                            target,
                            action: AnimationAction::AddToHand(id),
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
                        };

                        hand.push(anim.card);
                    }
                }
            }
        }
    }
}
