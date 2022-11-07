use models::{Hand};
use xs::{Xs, Seed};

#[derive(Clone, Default)]
pub struct State {
    pub rng: Xs,
    pub deck: Hand,
}

impl State {
    pub fn new(seed: Seed) -> State {
        let mut rng = xs::from_seed(seed);

        State {
            rng,
            deck: Hand::fresh_deck(&mut rng),
            .. <_>::default()
        }
    }

    pub fn tick(&mut self) {

    }
}
