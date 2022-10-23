use models::{Card, gen_card};
use platform_types::{command, unscaled};
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

        let mut state = State {
            rng,
            .. <_>::default()
        };

        for _ in 0..models::DECK_SIZE {
            state.add_splat();
        }

        state
    }

    pub fn add_splat(&mut self) {
        let rng = &mut self.rng;

        let kind: Card = gen_card(rng);
        let x = command::X::gen(rng).get();
        let y = command::Y::gen(rng).get();

        self.splats.push(Splat {
            kind,
            x,
            y,
        });
    }

    pub fn tick(&mut self) {
        let rng = &mut self.rng;

        let i = xs::range(rng, 0..self.splats.len() as _) as usize;

        let splat = &mut self.splats[i];
        match xs::range(rng, 0..2 as _) {
            0 => {
                splat.x = splat.x.saturating_sub(unscaled::W(1));
            },
            _ => {
                splat.x = splat.x.saturating_add(unscaled::W(1));
            },
        };
        match xs::range(rng, 0..2 as _) {
            0 => {
                splat.y = splat.y.saturating_sub(unscaled::H(1));
            },
            _ => {
                splat.y = splat.y.saturating_add(unscaled::H(1));
            },
        };
    }
}
