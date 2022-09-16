use game::Splat;
use gfx::{Commands, CHAR_ADVANCE_H, CHAR_SPACING_H, WIDTH_IN_CHARS};
use platform_types::{Button, Input, Speaker, SFX, unscaled};
pub use platform_types::StateParams;

#[derive(Clone, Copy, Default)]
enum HelpVis {
    #[default]
    Shown,
    Hidden
}

pub struct State {
    game_state: game::State,
    commands: Commands,
    input: Input,
    speaker: Speaker,
    help_vis: HelpVis
}

impl State {
    pub fn new((seed, logger, error_logger): StateParams) -> Self {
        unsafe {
            features::GLOBAL_LOGGER = logger;
            features::GLOBAL_ERROR_LOGGER = error_logger;
        }

        // We always want to log the seed, if there is a logger available, so use the function,
        // not the macro.
        features::log(&format!("{:?}", seed));

        let mut game_state = game::State::new(seed);
        game_state.add_splat();

        Self {
            game_state,
            commands: Commands::default(),
            input: Input::default(),
            speaker: Speaker::default(),
            help_vis: HelpVis::default(),
        }
    }
}

impl platform_types::State for State {
    fn frame(&mut self) -> (&[platform_types::Command], &[SFX]) {
        self.commands.clear();
        self.speaker.clear();

        if self.input.pressed_this_frame(Button::HELP) {
            self.help_vis = match self.help_vis {
                HelpVis::Shown => HelpVis::Hidden,
                HelpVis::Hidden => HelpVis::Shown,
            };

            self.speaker.request_sfx(SFX::ButtonPress);
        }

        update(
            &mut self.game_state,
            self.input,
            &mut self.speaker,
        );
        render(
            &mut self.commands,
            &self.game_state,
            self.help_vis
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

fn update(state: &mut game::State, input: Input, speaker: &mut Speaker) {
    if input.gamepad != <_>::default() {
        state.add_splat();
        speaker.request_sfx(SFX::CardPlace);
    }
}

fn render(
    commands: &mut Commands,
    state: &game::State,
    help_vis: HelpVis,
) {
    commands.clear_to(1 /* green */);

    match help_vis {
        HelpVis::Shown => {
            const HELP: &[u8] = b"press shift to show/hide this message
ABCDEFGHIJKLMNOPQRSTUVWXYZ
abcdefghijklmnopqrstuvwxyz
1234567890-=
!@#$%^&*()_+
";

            // TODO Is it worth it to avoid reflowing every frame?
            let help_text = text::reflow(HELP, WIDTH_IN_CHARS.into());

            let mut y = 0;

            for line in text::lines(&help_text)
                //.skip(self.top_index)
                //.take(HEIGHT_IN_CHARS)
            {
                commands.print_line(
                    line,
                    unscaled::X(0),
                    unscaled::Y(0)
                    + y * CHAR_ADVANCE_H
                    + CHAR_SPACING_H,
                    0 /* blue */
                );

                y += 1;
            }
        },
        HelpVis::Hidden => render_game(commands, state),
    }
}

fn render_game(
    commands: &mut Commands,
    state: &game::State,
) {
    for &Splat { kind, x, y } in &state.splats {
        commands.draw_card(kind, x, y);
    }
}

mod text {
    pub fn reflow(bytes: &[u8], width: usize) -> Vec<u8> {
        if width == 0 || bytes.is_empty() {
            return Vec::new();
        }

        let mut output = Vec::with_capacity(bytes.len() + bytes.len() / width);
    
        let mut x = 0;
        for word in split_whitespace(bytes) {
            x += word.len();

            if x == width && x == word.len() {
                output.extend(word.iter());
                continue;
            }
    
            if x >= width {
                output.push(b'\n');
    
                x = word.len();
            } else if x > word.len() {
                output.push(b' ');
    
                x += 1;
            }
            output.extend(word.iter());
        }
    
        output
    }

    pub fn split_whitespace(bytes: &[u8]) -> impl Iterator<Item = &[u8]> {
        bytes
            .split(|b| b.is_ascii_whitespace())
            .filter(|word| !word.is_empty())
    }

    pub fn lines(bytes: &[u8]) -> impl Iterator<Item = &[u8]> {
        bytes.split(|&b| b == b'\n')
    }
}