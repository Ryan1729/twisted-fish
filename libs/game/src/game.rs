use common::{xs::Xs, *};
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

#[allow(dead_code)]
enum Face {
    Up,
    Down,
}

fn draw_hand_ltr(
    framebuffer: &mut Framebuffer,
    hand: &Hand,
    offset: u8,
    (mut x, y): (u8, u8),
    face: Face,
) {
    match face {
        Face::Up => {
            for &card in hand.iter() {
                framebuffer.draw_card(card, x, y);

                x += offset;
            }
        }
        Face::Down => {
            for &_card in hand.iter() {
                framebuffer.draw_card_back(x, y);

                x += offset;
            }
        }
    }
}

fn draw_hand_ttb(
    framebuffer: &mut Framebuffer,
    hand: &Hand,
    offset: u8,
    (x, mut y): (u8, u8),
    face: Face,
) {
    match face {
        Face::Up => {
            for &card in hand.iter() {
                framebuffer.draw_card(card, x, y);

                y += offset;
            }
        }
        Face::Down => {
            for &_card in hand.iter() {
                framebuffer.draw_card_back(x, y);

                y += offset;
            }
        }
    }
}

fn draw_hand(framebuffer: &mut Framebuffer, hand: &Hand, face: Face) {
    let offset = get_card_offset(hand.spread, hand.len());

    match hand.spread {
        Spread::LTR((x, _), y) => {
            draw_hand_ltr(framebuffer, hand, offset, (x, y), face);
        }
        Spread::TTB((y, _), x) => {
            draw_hand_ttb(framebuffer, hand, offset, (x, y), face);
        }
    }
}

fn draw_hand_with_cursor_ltr(
    framebuffer: &mut Framebuffer,
    hand: &Hand,
    offset: u8,
    (mut x, y): (u8, u8),
    index: usize,
) {
    let mut selected_card_and_offset = None;
    for (i, &card) in hand.iter().enumerate() {
        if i == index {
            selected_card_and_offset = Some((card, x));
            x += offset;

            continue;
        }
        framebuffer.draw_card(card, x, y);

        x += offset;
    }

    if let Some((card, cursor_offset)) = selected_card_and_offset {
        framebuffer.draw_highlighted_card(card, cursor_offset, y);
    }
}

fn draw_hand_with_cursor_ttb(
    framebuffer: &mut Framebuffer,
    hand: &Hand,
    offset: u8,
    (x, mut y): (u8, u8),
    index: usize,
) {
    let mut selected_card_and_offset = None;
    for (i, &card) in hand.iter().enumerate() {
        if i == index {
            selected_card_and_offset = Some((card, y));
            y += offset;

            continue;
        }
        framebuffer.draw_card(card, x, y);

        y += offset;
    }

    if let Some((card, cursor_offset)) = selected_card_and_offset {
        framebuffer.draw_highlighted_card(card, x, cursor_offset);
    }
}

fn draw_hand_with_cursor(framebuffer: &mut Framebuffer, hand: &Hand, index: usize) {
    let offset = get_card_offset(hand.spread, hand.len());

    match hand.spread {
        Spread::LTR((x, _), y) => {
            draw_hand_with_cursor_ltr(framebuffer, hand, offset, (x, y), index);
        }
        Spread::TTB((y, _), x) => {
            draw_hand_with_cursor_ttb(framebuffer, hand, offset, (x, y), index);
        }
    }
}

fn update(state: &mut GameState, input: Input, speaker: &mut Speaker) {
    if input.gamepad != <_>::default() {
        state.add_splat();
    }
}

fn print_number_below_card(framebuffer: &mut Framebuffer, number: u8, x: u8, y: u8) {
    framebuffer.print_single_line_number(
        number as usize,
        x + (card::WIDTH / 2) - FONT_ADVANCE,
        y + card::HEIGHT + FONT_SIZE / 2,
        BLACK_INDEX,
    );
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
    render_in_game(framebuffer, &state);
}
