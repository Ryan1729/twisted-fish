use models::{Card, gen_card};
use platform_types::unscaled;
use xs::{Xs, Seed};

#[derive(Clone, Default)]
pub struct Splat {
    pub kind: Card,
    pub x: unscaled::X,
    pub y: unscaled::Y,
}

#[derive(Clone, Default)]
pub struct State {
    pub rng: Xs,
    pub splats: Vec<Splat>,
}

impl State {
    pub fn new(seed: Seed) -> State {
        let rng = xs::from_seed(seed);

        State {
            rng,
            .. <_>::default()
        }
    }

    pub fn add_splat(&mut self) {
        let rng = &mut self.rng;

        let kind: Card = gen_card(rng);
        let x = unscaled::X::gen(rng);
        let y = unscaled::Y::gen(rng);

        self.splats.push(Splat {
            kind,
            x,
            y,
        });
    }
}
