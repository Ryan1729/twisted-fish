use common::{xs::{Xs, Seed}, *};

pub struct State {
    pub current_player: PlayerID,
    
    pub winners: Vec<PlayerID>,
    // control state
    pub hand_index: u8,
}

macro_rules! dealt_hand {
    ($deck:expr, $spread:expr) => {{
        let mut hand = Hand::new($spread);

        hand.draw_from($deck);
        hand.draw_from($deck);
        hand.draw_from($deck);
        hand.draw_from($deck);
        hand.draw_from($deck);

        hand
    }};
}

#[derive(Clone, Default)]
pub struct GameState {
    pub rng: Xs,
    pub cpu_hands: [Hand; 3],
    pub hand: Hand,
    pub deck: Hand,
    pub discard: Hand,
    pub top_wild_declared_as: Option<Suit>,
    pub hand_index: u8,
}

impl GameState {
    pub fn new(seed: Seed) -> GameState {
        let mut rng = xs::from_seed(seed);

        let mut deck = Hand::new_shuffled_deck(&mut rng);

        let discard = Hand::new(Spread::stack(DISCARD_X, DISCARD_Y));

        let hand = dealt_hand!(
            &mut deck,
            Spread::LTR(TOP_AND_BOTTOM_HAND_EDGES, PLAYER_HAND_HEIGHT)
        );

        let cpu_hands = [
            dealt_hand!(
                &mut deck,
                Spread::TTB(LEFT_AND_RIGHT_HAND_EDGES, LEFT_CPU_HAND_X)
            ),
            dealt_hand!(
                &mut deck,
                Spread::LTR(TOP_AND_BOTTOM_HAND_EDGES, MIDDLE_CPU_HAND_HEIGHT,)
            ),
            dealt_hand!(
                &mut deck,
                Spread::TTB(LEFT_AND_RIGHT_HAND_EDGES, RIGHT_CPU_HAND_X)
            ),
        ];

        GameState {
            rng,
            cpu_hands,
            hand,
            deck,
            discard,
            .. <_>::default()
        }
    }
}
