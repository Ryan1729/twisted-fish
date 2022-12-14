use gfx::{Commands, CHAR_ADVANCE_H, CHAR_SPACING_H, CHAR_SPACING};
use platform_types::{Button, Input, Speaker, SFX, unscaled, command};
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
    help_vis: HelpVis,
    top_index_with_offset: usize,
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

        Self {
            game_state: game::State::new(seed),
            commands: Commands::default(),
            input: Input::default(),
            speaker: Speaker::default(),
            help_vis: if true { HelpVis::Hidden } else {HelpVis::default()},
            top_index_with_offset: 0,
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

        match self.help_vis {
            HelpVis::Shown => {
                if self.input.gamepad.contains(Button::DOWN) {
                    self.top_index_with_offset += 1;
                } else if self.input.gamepad.contains(Button::UP) {
                    self.top_index_with_offset =
                        self.top_index_with_offset.saturating_sub(1);
                }

                render_shown(
                    &mut self.commands,
                    self.top_index_with_offset,
                );
            },
            HelpVis::Hidden => {
                game::update_and_render(
                    &mut self.commands,
                    &mut self.game_state,
                    self.input,
                    &mut self.speaker,
                );
            },
        }

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

fn render_shown(
    commands: &mut Commands,
    top_index_with_offset: usize,
) {
    const HELP: &[u8] = b"Press shift to show/hide this message.
----------------
Help
----------------
TODO

----------------
Image Credits
----------------
\"Shark Jaws\" by Lorc, (modified)
https://game-icons.net/1x1/lorc/shark-jaws.html

\"Jumping Dog\" by Delapouite, (modified)
https://game-icons.net/1x1/delapouite/jumping-dog.html

\"Fish Escape\" by Delapouite, (modified)
https://game-icons.net/1x1/delapouite/fish-escape.html

\"Police Officer Head\" by Delapouite, (modified)
https://game-icons.net/1x1/delapouite/police-officer-head.html

\"Canned Fish\" by Delapouite, (modified)
https://game-icons.net/1x1/delapouite/canned-fish.html

\"Scuba Mask\" by Delapouite, (modified)
https://game-icons.net/1x1/delapouite/scuba-mask.html

\"Wooden Sign\" by Lorc, (modified)
https://game-icons.net/1x1/lorc/wooden-sign.html

\"Fishing Hook\" by Lorc, (modified)
https://game-icons.net/1x1/lorc/fishing-hook.html

\"Fishing Net\" by Lorc, (modified)
https://game-icons.net/1x1/lorc/fishing-net.html

\"Mini Submarine\" by Delapouite, (modified)
https://game-icons.net/1x1/delapouite/mini-submarine.html

\"Fishing Lure\" by Delapouite, (modified)
https://game-icons.net/1x1/delapouite/fishing-lure.html

\"Harpoon Trident\" by Lorc
https://game-icons.net/1x1/lorc/harpoon-trident.html

\"Sperm Whale\" by Delapouite
https://game-icons.net/1x1/delapouite/sperm-whale.html

\"Porcupinefish\" by Delapouite
https://game-icons.net/1x1/delapouite/porcupinefish.html

\"Sea Star\" by Delapouite
https://game-icons.net/1x1/delapouite/sea-star.html

\"Clownfish\" by Delapouite
https://game-icons.net/1x1/delapouite/clownfish.html

\"Golem Head\" by Delapouite
https://game-icons.net/1x1/delapouite/golem-head.html

\"Sad Crab\" by Lorc
https://game-icons.net/1x1/lorc/sad-crab.html

\"Eel\" by Delapouite
https://game-icons.net/1x1/delapouite/eel.html

\"Flying Trout\" by Cathelineau
https://game-icons.net/1x1/cathelineau/flying-trout.html

\"Jellyfish\" by Lorc
https://game-icons.net/1x1/lorc/jellyfish.html

\"Shrimp\" by Delapouite
https://game-icons.net/1x1/delapouite/shrimp.html

\"Shark Fin\" by Delapouite
https://game-icons.net/1x1/delapouite/shark-fin.html

\"Double Fish\" by Delapouite
https://game-icons.net/1x1/delapouite/double-fish.html

\"Fishing Pole\" by Delapouite
https://game-icons.net/1x1/delapouite/fishing-pole.html

All of the above icons used under CC BY 3.0.

----------------
Font Credits
----------------

\"Monogram\" by Vin\xEDcius Men\xE9zio (@vmenezio).
https://datagoblin.itch.io/monogram

----------------
Everything Else
----------------
Ryan Wiedemann (Ryan1729 on github)
";

    for (y, line) in text::lines(HELP)
        .skip((top_index_with_offset as u16 / CHAR_ADVANCE_H.get().get()) as usize)
        .take(command::h_to_usize(command::HEIGHT * CHAR_ADVANCE_H))
        .enumerate()
    {
        let y = y as unscaled::Inner;

        let offset = top_index_with_offset as u16 % CHAR_ADVANCE_H.get().get();

        commands.print_line(
            line,
            unscaled::XY {
                x: unscaled::X(CHAR_SPACING as _),
                y: unscaled::Y(0),
            }
            // TODO investigate scrolling shimmering which seems to be
            // related to this part. Do we need to make the scrolling
            // speed up, then slow down or something? or is the offset
            // calculation just wrong?  Maybe it won't look right unless
            // we add more in-between frames?
            + unscaled::H(
                ((y + 1) * CHAR_ADVANCE_H.get().get())
                - offset
                - 1
            )
            + CHAR_SPACING_H.get(),
            0 // No override
        );
    }
}

mod text {
    #[allow(unused)]
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