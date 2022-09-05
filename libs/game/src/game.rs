use common::*;
use game_state::{GameState, Splat};
use platform_types::{Button, Input, Speaker, State, SFX};
pub use platform_types::StateParams;

pub struct AppState {
    pub game_state: GameState,
    pub framebuffer: Framebuffer,
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

        Self {
            game_state: GameState::new(seed),
            framebuffer: Framebuffer::default(),
            input: Input::default(),
            speaker: Speaker::default(),
        }
    }
}

impl State for AppState {
    fn frame(&mut self) -> (&[platform_types::Command], &[SFX]) {
        self.framebuffer.commands.clear();
        self.speaker.clear();
        update_and_render(
            &mut self.framebuffer,
            &mut self.game_state,
            self.input,
            &mut self.speaker,
        );

        self.input.previous_gamepad = self.input.gamepad;

        (&self.framebuffer.commands, self.speaker.slice())
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
fn render_in_game(framebuffer: &mut Framebuffer, state: &GameState) {
    for &Splat { kind, x, y } in &state.splats {
        framebuffer.draw_card(kind, x, y);
    }
}

#[inline]
fn update_and_render(
    framebuffer: &mut Framebuffer,
    state: &mut GameState,
    input: Input,
    speaker: &mut Speaker,
) {
    framebuffer.clearTo(1 /* green */);

    update(state, input, speaker);
    render_in_game(framebuffer, state);
}
