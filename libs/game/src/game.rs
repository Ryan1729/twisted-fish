use common::*;
use game_state::{GameState, Splat};
use platform_types::{Button, Input, Speaker, State, SFX};
pub use platform_types::StateParams;

pub struct AppState {
    pub game_state: GameState,
    pub commands: Commands,
    pub input: Input,
    pub speaker: Speaker,
}

impl AppState {
    pub fn new((seed, logger, error_logger): StateParams) -> Self {
        unsafe {
            GLOBAL_LOGGER = logger;
            GLOBAL_ERROR_LOGGER = error_logger;
        }

        // We always want to log the seed, if there is a logger available, so use the function,
        // not the macro.
        log(&format!("{:?}", seed));

        let mut game_state = GameState::new(seed);
        game_state.add_splat();

        Self {
            game_state,
            commands: Commands::default(),
            input: Input::default(),
            speaker: Speaker::default(),
        }
    }
}

impl State for AppState {
    fn frame(&mut self) -> (&[platform_types::Command], &[SFX]) {
        self.commands.clear();
        self.speaker.clear();
        update_and_render(
            &mut self.commands,
            &mut self.game_state,
            self.input,
            &mut self.speaker,
        );

        self.input.previous_gamepad = self.input.gamepad;

        (self.commands.slice(), self.speaker.slice())
    }

    fn press(&mut self, button: Button) {
        if self.input.previous_gamepad.contains(button) {
            //This is meant to pass along the key repeat, if any.
            //Not sure if rewriting history is the best way to do this.
            self.input.previous_gamepad.remove(button);
        }

        self.input.gamepad.insert(button);
    }

    fn release(&mut self, button: Button) {
        self.input.gamepad.remove(button);
    }
}

fn update(state: &mut GameState, input: Input, speaker: &mut Speaker) {
    if input.gamepad != <_>::default() {
        state.add_splat();
        speaker.request_sfx(SFX::CardPlace);
    }
}

#[inline]
fn render_in_game(commands: &mut Commands, state: &GameState) {
    for &Splat { kind, x, y } in &state.splats {
        commands.draw_card(kind, x, y);
    }
}

#[inline]
fn update_and_render(
    commands: &mut Commands,
    state: &mut GameState,
    input: Input,
    speaker: &mut Speaker,
) {
    commands.clear_to(1 /* green */);

    update(state, input, speaker);
    render_in_game(commands, state);
}
