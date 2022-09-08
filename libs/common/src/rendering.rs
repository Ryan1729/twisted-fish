use platform_types::{Command, Kind, PaletteIndex, Rect, FONT_WIDTH};
use crate::*;

#[derive(Default)]
pub struct Commands {
    commands: Vec<Command>,
}

impl Commands {
    pub fn slice(&self) -> &[Command] {
        &self.commands
    }

    pub fn clear(&mut self) {
        self.commands.clear();
    }

    pub fn sspr(
        &mut self,
        sprite_x: u8,
        sprite_y: u8,
        w: u8,
        h: u8,
        display_x: u8,
        display_y: u8,
    ) {
        self.commands.push(
            Command {
                kind: Kind::Gfx((sprite_x, sprite_y)),
                rect: Rect {
                    x: display_x,
                    y: display_y,
                    w, 
                    h,
                },
            }
        );
    }

    fn print_char_raw(
        &mut self,
        sprite_xy: (u8, u8),
        (w, h): (u8, u8),
        (display_x, display_y): (u8, u8),
        colour: PaletteIndex,
    ) {
        self.commands.push(
            Command {
                kind: Kind::Font(sprite_xy, colour),
                rect: Rect {
                    x: display_x,
                    y: display_y,
                    w, 
                    h,
                },
            }
        );
    }

    pub fn clear_to(&mut self, colour: PaletteIndex) {
        self.commands.push(
            Command {
                kind: Kind::Colour(colour),
                rect: Rect {
                    x: 0,
                    y: 0,
                    w: screen::WIDTH,
                    h: screen::HEIGHT,
                },
            }
        );
    }

    pub fn print_char(&mut self, character: u8, x: u8, y: u8, colour: u8) {
        let (sprite_x, sprite_y) = get_char_xy(character);
        self.print_char_raw(
            (sprite_x, sprite_y),
            (FONT_SIZE, FONT_SIZE),
            (x, y),
            colour
        );
    }

    pub fn draw_card(&mut self, card: Card, x: u8, y: u8) {
        self.sspr(
            card::FRONT_SPRITE_X,
            card::FRONT_SPRITE_Y,
            card::WIDTH,
            card::HEIGHT,
            x,
            y,
        );

        let (colour, suit_char) = get_suit_colour_and_char(get_suit(card));

        let rank_char = get_rank_char(card);

        self.print_char(
            rank_char,
            x + card::LEFT_RANK_X,
            y + card::LEFT_RANK_Y,
            colour,
        );
        self.print_char(
            suit_char,
            x + card::LEFT_SUIT_X,
            y + card::LEFT_SUIT_Y,
            colour,
        );

        self.print_char(
            rank_char | FONT_FLIP,
            x + card::RIGHT_RANK_X,
            y + card::RIGHT_RANK_Y,
            colour,
        );
        self.print_char(
            suit_char | FONT_FLIP,
            x + card::RIGHT_SUIT_X,
            y + card::RIGHT_SUIT_Y,
            colour,
        );
    }
}

pub fn get_char_xy(sprite_number: u8) -> (u8, u8) {
    const SPRITES_PER_ROW: u8 = FONT_WIDTH as u8 / FONT_SIZE;

    (
        (sprite_number % SPRITES_PER_ROW) * FONT_SIZE,
        (sprite_number / SPRITES_PER_ROW) * FONT_SIZE,
    )
}

