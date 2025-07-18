use gfx::{Commands};
use platform_types::{Button, Input, Speaker};
use models::{Rank, Suit, Zinger, fish_card, zinger_card};
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

    force_into_start_of_hand(&mut u_and_r.state, zinger_card(Zinger::DivineIntervention), FullHandId::Player);

    // playing divine intervention
    u_and_r.call(Button::A);

    // If this didn't panic, the test passes.
    // TODO? Asserts of some kind?
}

#[test]
fn player_respond_to_no_fishing() {
    let mut u_and_r = UAndR::new(SOME_SEED);

    force_into_start_of_hand(&mut u_and_r.state, fish_card(Rank::Dogfish, Suit::Red), FullHandId::Cpu1);
    force_into_start_of_hand(&mut u_and_r.state, fish_card(Rank::Dogfish, Suit::Yellow), FullHandId::Player);
    force_into_start_of_hand(&mut u_and_r.state, zinger_card(Zinger::NoFishing), FullHandId::Cpu1);

    // Start asking for the card so that CPU 1 will use No Fishing
    u_and_r.call(Button::A);
    // Move to the submit button, assuming it is set to Cpu1 and Red by default
    u_and_r.call(Button::RIGHT);
    u_and_r.call_until_animations_are_done();
dbg!();
    u_and_r.call(Button::RIGHT);
    u_and_r.call_until_animations_are_done();
    // Press The submit button
dbg!();
    u_and_r.call(Button::A);
    u_and_r.call_until_animations_are_done();

    u_and_r.call_no_button();
dbg!();


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

        self.call_no_button();
    }

    fn call_no_button(&mut self) {
        update_and_render(
            &mut self.commands,
            &mut self.state,
            self.input,
            &mut self.speaker,
        );

        self.input.previous_gamepad = self.input.gamepad;
    }

    fn call_until_animations_are_done(&mut self) {
        while !self.state.animations.all_done() {
            self.call_no_button();
        }
    }
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