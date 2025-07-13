use gfx::{Commands};
use platform_types::{Button, Input, Speaker};
use models::{Card, HandId, Zinger, zinger_card};
use game::*;

const SOME_SEED: xs::Seed = [42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42];

#[test]
fn basic_mashing() {
    let mut u_and_r = UAndR::new(SOME_SEED);

    const INPUT_COUNT: usize = 16;

    for _ in 0..INPUT_COUNT {
        let button = gen_button(&mut u_and_r.state.rng);
        u_and_r.call(button);
    }

    // If this didn't panic, the test passes.
}

#[test]
fn player_play_divine_intervention() {
    let mut u_and_r = UAndR::new(SOME_SEED);

    // Loop around to the first card in hand
    u_and_r.call(Button::RIGHT);

    force_into_start_of_hand(&mut u_and_r.state, zinger_card(Zinger::DivineIntervention), HandId::Player);

    // playing divine intervention
    u_and_r.call(Button::A);

    // If this didn't panic, the test passes.
    // TODO? Asserts of some kind?
}

/// A testhelper to call game::update_and_render
struct UAndR {
    commands: Commands,
    state: State,
    speaker: Speaker,
    input: Input,
}

impl UAndR {
    fn new(seed: xs::Seed) -> Self {
        let mut output = Self {
            commands: Commands::default(),
            state: State::new(seed),
            speaker: Speaker::default(),
            input: Input::default(),
        };

        while !output.state.animations.all_done() {
            output.call(Button::UP);
        }

        output
    }

    fn call(&mut self, button: Button) {
        self.input.gamepad.insert(button);

        update_and_render(
            &mut self.commands,
            &mut self.state,
            self.input,
            &mut self.speaker,
        );

        self.input.previous_gamepad = self.input.gamepad;
    }
}

fn force_into_start_of_hand(
    state: &mut State, 
    target_card: Card, 
    hand_id: HandId
) {
    let hands = [
        &mut state.cards.deck,
        &mut state.cards.player,
        &mut state.cards.cpu1,
        &mut state.cards.cpu2,
        &mut state.cards.cpu3,
        &mut state.cards.player_baskets,
        &mut state.cards.cpu1_baskets,
        &mut state.cards.cpu2_baskets,
        &mut state.cards.cpu3_baskets,
        &mut state.cards.discard,
    ];

    let mut extracted_card = <_>::default();

    for hand in hands {
        let index_opt = hand.iter().position(|c| c == target_card);

        if let Some(index) = index_opt {
            extracted_card = hand.remove(index.try_into().expect("index should fit in a CardIndex"));
            break
        }
    }

    let target_hand = match hand_id {
        HandId::Player => &mut state.cards.player,
        HandId::Cpu1 => &mut state.cards.cpu1,
        HandId::Cpu2 => &mut state.cards.cpu2,
        HandId::Cpu3 => &mut state.cards.cpu3,
    };

    target_hand.swap_insert_top(extracted_card.expect("card should have been found"));
}

const GAME_BUTTONS: [Button; 7] = [
    Button::A,
    Button::B,
    Button::START,
    Button::UP,
    Button::DOWN,
    Button::LEFT,
    Button::RIGHT,
];

fn gen_button(rng: &mut xs::Xs) -> Button {
    GAME_BUTTONS[xs::range(rng, 0..GAME_BUTTONS.len() as u32) as usize]
}