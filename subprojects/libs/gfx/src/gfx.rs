macro_rules! compile_time_assert {
    ($assertion: expr) => (
        #[allow(unknown_lints, clippy::eq_op)]
        // Based on the const_assert macro from static_assertions;
        const _: [(); 0 - !{$assertion} as usize] = [];
    )
}

use models::{Card, Rank, Suit, get_rank, get_suit};

use platform_types::{ARGB, Command, sprite, unscaled, command::{self, Rect}, CHAR_W, CHAR_H, CHAR_WIDTH, CHAR_HEIGHT, FONT_WIDTH, CARD_WIDTH, CARD_HEIGHT, bytes_lines};

const GFX_RANK_COUNT: u8 = 13;

const FONT_OFFSET: sprite::H = unscaled::h_const_mul(
    card::IMAGE_H.get(),
    GFX_RANK_COUNT as _
);

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
        sprite_xy: sprite::XY,
        rect: command::Rect,
    ) {
        self.push_command_if_useful(
            Command {
                rect,
                sprite_xy,
                colour_override: 0,
            }
        );
    }

    pub fn print_char(
        &mut self,
        mut character: u8,
        unscaled::XY { x, y } : unscaled::XY,
        colour_override: ARGB
    ) {
        let mut y = y;
        let mut font_offset = FONT_OFFSET;

        let mut h = CHAR_H.get();
        // If it's one of the accented characters ...
        if character > 0xE0 {
            // ... we want to shift back into the extras characters we stuffed into 
            // the gfx where non-printables go, ...
            character -= 0xD0;

            // ... and we need an extra pixel for the accents
            h += unscaled::H(1);
            font_offset -= unscaled::H(1);
            y = y.saturating_sub(unscaled::H(1));
        }

        let sprite_xy = {
            const SPRITES_PER_ROW: u8 = FONT_WIDTH / CHAR_WIDTH;

            sprite::XY {
                x: sprite::X(Into::into(
                    (character % SPRITES_PER_ROW) * CHAR_WIDTH,
                )),
                y: sprite::Y(Into::into(
                    (character / SPRITES_PER_ROW) * CHAR_HEIGHT,
                )) + font_offset,
            }
        };

        self.push_command_if_useful(
            Command {
                rect: Rect::from_unscaled(
                    unscaled::Rect {
                        x,
                        y,
                        w: CHAR_W.get(),
                        h,
                    }
                ),
                sprite_xy,
                colour_override,
            }
        );
    }

    pub fn print_line(
        &mut self,
        bytes: &[u8],
        mut xy : unscaled::XY,
        colour: ARGB,
    ) {
        for &c in bytes.iter() {
            self.print_char(c, xy, colour);
            xy.x += CHAR_ADVANCE_W.get();
        }
    }

    pub fn print(
        &mut self,
        bytes: &[u8],
        mut xy : unscaled::XY,
        colour: ARGB,
    ) {
        for line in bytes_lines(bytes) {
            self.print_line(line, xy, colour);
            xy.y += CHAR_ADVANCE_H.get();
        }
    }

    /// As of this writing, this assumes that the bytes have already been arranged
    /// such that they will fit in the passed rect.
    pub fn print_centered(
        &mut self,
        bytes: &[u8],
        rect : unscaled::Rect,
        colour: ARGB,
    ) {
        let longest_line = platform_types::longest_line_of(bytes);

        let unscaled::Rect { x, y, w, h } = rect;
    
        let mut xy = unscaled::XY {
            x: x + (w / 2),
            y: y + (h / 2),
        };
    
        // TODO reduce duplication with `center_line_in_rect`?
        xy -= (CHAR_ADVANCE_W * longest_line.len() as _).get() / 2;
        let count = bytes_lines(bytes).count();
        if count == 1 {
            xy -= CHAR_H.get() / 2;
        } else {
            for _ in 0..count {
                xy -= CHAR_ADVANCE_H.get() / 2;
            }
        }

        self.print(
            bytes,
            xy,
            colour,
        );
    }

    const LABEL_LENGTH: u8 = 10;

    fn label_lines_for(
        suit_opt: Option<Suit>,
        rank_opt: Option<Rank>,
    ) -> (
        &'static [u8; Self::LABEL_LENGTH as _],
        &'static [u8; Self::LABEL_LENGTH as _]
    ) {
        let lazily_left_here: Option<()> = None;
        match (suit_opt, rank_opt, lazily_left_here) {
            (Some(Suit::Red), Some(Rank::Barnacle), None) => (
                b"Red       ",
                b"Barnacle  ",
            ),
            (Some(Suit::Green), Some(Rank::Barnacle), None) => (
                b"Green     ",
                b"Barnacle  ",
            ),
            (Some(Suit::Blue), Some(Rank::Barnacle), None) => (
                b"Blue      ",
                b"Barnacle  ",
            ),
            (Some(Suit::Yellow), Some(Rank::Barnacle), None) => (
                b"Yellow    ",
                b"Barnacle  ",
            ),
            (Some(Suit::Purple), Some(Rank::Barnacle), None) => (
                b"Purple    ",
                b"Barnacle  ",
            ),
            (Some(Suit::Red), Some(Rank::Crab), None) => (
                b"Red       ",
                b"Crab      ",
            ),
            (Some(Suit::Green), Some(Rank::Crab), None) => (
                b"Green     ",
                b"Crab      ",
            ),
            (Some(Suit::Blue), Some(Rank::Crab), None) => (
                b"Blue      ",
                b"Crab      ",
            ),
            (Some(Suit::Yellow), Some(Rank::Crab), None) => (
                b"Yellow    ",
                b"Crab      ",
            ),
            (Some(Suit::Purple), Some(Rank::Crab), None) => (
                b"Purple    ",
                b"Crab      ",
            ),
            (Some(Suit::Red), Some(Rank::Eel), None) => (
                b"Red       ",
                b"Eel       ",
            ),
            (Some(Suit::Green), Some(Rank::Eel), None) => (
                b"Green     ",
                b"Eel       ",
            ),
            (Some(Suit::Blue), Some(Rank::Eel), None) => (
                b"Blue      ",
                b"Eel       ",
            ),
            (Some(Suit::Yellow), Some(Rank::Eel), None) => (
                b"Yellow    ",
                b"Eel       ",
            ),
            (Some(Suit::Purple), Some(Rank::Eel), None) => (
                b"Purple    ",
                b"Eel       ",
            ),
            (Some(Suit::Red), Some(Rank::Jellyfish), None) => (
                b"Red       ",
                b"Jellyfish ",
            ),
            (Some(Suit::Green), Some(Rank::Jellyfish), None) => (
                b"Green     ",
                b"Jellyfish ",
            ),
            (Some(Suit::Blue), Some(Rank::Jellyfish), None) => (
                b"Blue      ",
                b"Jellyfish ",
            ),
            (Some(Suit::Yellow), Some(Rank::Jellyfish), None) => (
                b"Yellow    ",
                b"Jellyfish ",
            ),
            (Some(Suit::Purple), Some(Rank::Jellyfish), None) => (
                b"Purple    ",
                b"Jellyfish ",
            ),
            (Some(Suit::Red), Some(Rank::Shrimp), None) => (
                b"Red       ",
                b"Shrimp    ",
            ),
            (Some(Suit::Green), Some(Rank::Shrimp), None) => (
                b"Green     ",
                b"Shrimp    ",
            ),
            (Some(Suit::Blue), Some(Rank::Shrimp), None) => (
                b"Blue      ",
                b"Shrimp    ",
            ),
            (Some(Suit::Yellow), Some(Rank::Shrimp), None) => (
                b"Yellow    ",
                b"Shrimp    ",
            ),
            (Some(Suit::Purple), Some(Rank::Shrimp), None) => (
                b"Purple    ",
                b"Shrimp    ",
            ),
            (Some(Suit::Red), None, None) => (
                b"Red       ",
                b"          ",
            ),
            (Some(Suit::Green), None, None) => (
                b"Green     ",
                b"          ",
            ),
            (Some(Suit::Blue), None, None) => (
                b"Blue      ",
                b"          ",
            ),
            (Some(Suit::Yellow), None, None) => (
                b"Yellow    ",
                b"          ",
            ),
            (Some(Suit::Purple), None, None) => (
                b"Purple    ",
                b"          ",
            ),
            (None, Some(Rank::Barnacle), None) => (
                b"          ",
                b"Barnacle  ",
            ),
            (None, Some(Rank::Crab), None) => (
                b"          ",
                b"Crab      ",
            ),
            (None, Some(Rank::Eel), None) => (
                b"          ",
                b"Eel       ",
            ),
            (None, Some(Rank::Jellyfish), None) => (
                b"          ",
                b"Jellyfish ",
            ),
            (None, Some(Rank::Shrimp), None) => (
                b"          ",
                b"Shrimp    ",
            ),
            _ => {
                debug_assert!(false, "No lines for: {suit_opt:?} {rank_opt:?}");
                (
                    b"line 1 ???",
                    b"line 2 ???",
                )
            }
        }
    }

    pub fn draw_card(
        &mut self,
        card: Card,
        xy: unscaled::XY,
    ) {
        self.draw_shadow_for_card_at(xy);

        let suit_opt = get_suit(card);
        let rank_opt = get_rank(card);

        self.sspr(
            sprite::XY {
                x: card::BACKING_SPRITE_X,
                y: card::BACKING_SPRITE_BASE_Y
                + card::HEIGHT.get()
                * sprite::Inner::from(card / models::RANK_COUNT)
            },
            Rect::from_unscaled(unscaled::Rect {
                x: xy.x,
                y: xy.y,
                w: card::WIDTH.get(),
                h: card::HEIGHT.get(),
            })
        );

        let image_x = match suit_opt {
            Some(suit) => card::IMAGE_BASE_X
                + unscaled::Inner::from(suit as u8)
                * card::IMAGE_W.get(),
            None => card::ZINGER_IMAGE_X,
        };

        let image_y = match rank_opt {
            Some(rank) => {
                let munged_rank: u8 = match rank {
                    models::Rank::Crab => 1,
                    models::Rank::Eel => 3,
                    models::Rank::Barnacle => 0,
                    models::Rank::Jellyfish => 6,
                    models::Rank::Shrimp => 7,
                };

                card::IMAGE_BASE_Y
                + unscaled::Inner::from(munged_rank)
                * card::IMAGE_H.get()
            },
            None => {
                debug_assert!(false, "No suit or zinger for card: {card}");
                card::IMAGE_BASE_Y
            },
        };

        self.sspr(
            sprite::XY {
                x: image_x,
                y: image_y,
            },
            Rect::from_unscaled(unscaled::Rect {
                x: xy.x + card::IMAGE_W_OFFSET.get(),
                y: xy.y + card::IMAGE_H_OFFSET.get(),
                w: card::IMAGE_W.get(),
                h: card::IMAGE_H.get(),
            })
        );

        let (line1, line2) = Self::label_lines_for(
            suit_opt,
            rank_opt,
        );

        self.print_line(
            line1,
            xy + card::LINE_W_OFFSET.get() + card::LINE_H_1_OFFSET.get(),
            card::TEXT_COLOUR,
        );
        self.print_line(
            line2,
            xy + card::LINE_W_OFFSET.get() + card::LINE_H_2_OFFSET.get(),
            card::TEXT_COLOUR,
        );
    }

    pub fn draw_card_back(
        &mut self,
        xy: unscaled::XY,
    ) {
        self.draw_shadow_for_card_at(xy);

        let image_x = card::BACKING_SPRITE_X;
        let image_y = card::BACKING_SPRITE_BASE_Y
            + unscaled::Inner::from(models::SUIT_COUNT + 2)
            * card::HEIGHT.get();

        self.sspr(
            sprite::XY {
                x: image_x, 
                y: image_y,
            },
            Rect::from_unscaled(unscaled::Rect {
                x: xy.x,
                y: xy.y,
                w: card::WIDTH.get(),
                h: card::HEIGHT.get(),
            })
        );
    }

    fn draw_shadow_for_card_at(
        &mut self,
        xy: unscaled::XY,
    ) {
        let image_x = card::BACKING_SPRITE_X;
        let image_y = card::BACKING_SPRITE_BASE_Y
            + unscaled::Inner::from(models::SUIT_COUNT + 1)
            * card::HEIGHT.get();

        self.sspr(
            sprite::XY {
                x: image_x, 
                y: image_y,
            },
            Rect::from_unscaled(unscaled::Rect {
                x: xy.x + unscaled::W(2),
                y: xy.y + unscaled::H(2),
                w: card::WIDTH.get(),
                h: card::HEIGHT.get(),
            })
        );
    }

    pub fn draw_selectrum(
        &mut self,
        xy: unscaled::XY,
    ) {
        let image_x = card::BACKING_SPRITE_X;
        let image_y = card::BACKING_SPRITE_BASE_Y
            + unscaled::Inner::from(models::SUIT_COUNT + 3)
            * card::HEIGHT.get();

        self.sspr(
            sprite::XY {
                x: image_x, 
                y: image_y,
            },
            Rect::from_unscaled(unscaled::Rect {
                x: xy.x,
                y: xy.y,
                w: card::WIDTH.get(),
                h: card::HEIGHT.get(),
            })
        );
    }
}

#[derive(Clone, Copy)]
pub enum NineSlice {
    Window,
    Button,
    ButtonHot,
    ButtonPressed,
}

impl NineSlice {
    pub const CELL_W: unscaled::W = unscaled::W(8);
    pub const CELL_H: unscaled::H = unscaled::H(8);

    pub const GRID_W: unscaled::W = unscaled::W(24);
    pub const GRID_H: unscaled::H = unscaled::H(24);

    const BASE: sprite::XY = sprite::XY {
        x: sprite::X(FONT_WIDTH as _),
        y: sprite::y_const_add_h(sprite::Y(0), FONT_OFFSET),
    };

    fn top_left(self) -> sprite::XY {
        NineSlice::BASE 
        + NineSlice::GRID_W
        * match self {
            NineSlice::Window => 0,
            NineSlice::Button => 1,
            NineSlice::ButtonHot => 2,
            NineSlice::ButtonPressed => 3,
        }
    }
}

impl Commands {
    pub fn draw_nine_slice(
        &mut self,
        nine_slice: NineSlice,
        unscaled::Rect { x, y, w, h }: unscaled::Rect,
    ) {
        const WIDTH: unscaled::W = NineSlice::CELL_W;
        const HEIGHT: unscaled::H = NineSlice::CELL_H;

        macro_rules! r {
            ($x: ident, $y: ident $(,)?) => {
                Rect::from_unscaled(unscaled::Rect {
                    x: $x,
                    y: $y,
                    w: WIDTH,
                    h: HEIGHT,
                })
            };
        }

        let top_left: sprite::XY = nine_slice.top_left();

        let top: sprite::XY = top_left + WIDTH;
        let top_right: sprite::XY = top + WIDTH;

        let middle_left: sprite::XY = top_left + HEIGHT;
        let middle: sprite::XY = top + HEIGHT;
        let middle_right: sprite::XY = top_right + HEIGHT;

        let bottom_left: sprite::XY = middle_left + HEIGHT;
        let bottom: sprite::XY = middle + HEIGHT;
        let bottom_right: sprite::XY = middle_right + HEIGHT;

        let after_left_corner = x.saturating_add(WIDTH);
        let before_right_corner = x.saturating_add(w).saturating_sub(WIDTH);

        let below_top_corner = y.saturating_add(HEIGHT);
        let above_bottom_corner = y.saturating_add(h).saturating_sub(HEIGHT);

        macro_rules! step_by {
            (
                for $element: ident in $start: ident .. $end: ident 
                step_by $by: ident 
                $body: block
            ) => ({
                let mut $element = $start;
                while $element < $end {
                    $body

                    $element += $by;
                }
            })
        }

        step_by!(
            for fill_y in below_top_corner..above_bottom_corner
            step_by HEIGHT {
                step_by!(
                    for fill_x in after_left_corner..before_right_corner
                    step_by WIDTH {
                        self.sspr(
                            middle,
                            r!(fill_x, fill_y),
                        );
                    }
                )
            }
        );

        step_by!(
            for fill_x in after_left_corner..before_right_corner
            step_by WIDTH {
                self.sspr(
                    top,
                    r!(fill_x, y),
                );
    
                self.sspr(
                    bottom,
                    r!(fill_x, above_bottom_corner),
                );
            }
        );

        step_by!(
            for fill_y in below_top_corner..above_bottom_corner
            step_by HEIGHT {
                self.sspr(
                    middle_left,
                    r!(x, fill_y),
                );
    
                self.sspr(
                    middle_right,
                    r!(before_right_corner, fill_y),
                );
            }
        );

        self.sspr(
            top_left,
            r!(x, y),
        );

        self.sspr(
            top_right,
            r!(before_right_corner, y),
        );

        self.sspr(
            bottom_left,
            r!(x, above_bottom_corner),
        );

        self.sspr(
            bottom_right,
            r!(before_right_corner, above_bottom_corner),
        );
    }

    fn push_command_if_useful(&mut self, command: Command) {
        if command.rect.x_min != command.rect.x_max
        && command.rect.y_min != command.rect.y_max {
            self.commands.push(command);
        }
    }
}

#[derive(Clone, Copy)]
pub enum Highlighted {
    No,
    Yes,
}

#[derive(Clone, Copy)]
pub enum ChevronDir {
    Up,
    Down,
}

pub const CHEVRON_BASE_X: sprite::X = NineSlice::BASE.x;
pub const CHEVRON_Y: sprite::Y = sprite::y_const_add_h(
    NineSlice::BASE.y,
    NineSlice::GRID_H
);
pub const CHEVRON_W: unscaled::W = unscaled::W(24);
pub const CHEVRON_H: unscaled::H = unscaled::H(12);
pub const CHEVRON_WH: unscaled::WH = unscaled::WH {
    w: CHEVRON_W,
    h: CHEVRON_H,
};

impl Commands {
    pub fn draw_chevron(
        &mut self,
        xy: unscaled::XY,
        dir: ChevronDir,
        highlighted: Highlighted,
    ) {
        use ChevronDir::*;
        use Highlighted::*;

        let x = CHEVRON_BASE_X
            + CHEVRON_W * match (dir, highlighted) {
                (Up, No) => 0,
                (Down, No) => 1,
                (Up, Yes) => 2,
                (Down, Yes) => 3,
            };

        self.sspr(
            sprite::XY {
                x,
                y: CHEVRON_Y
            },
            Rect::from_unscaled(unscaled::Rect::xy_wh(
                xy,
                CHEVRON_WH,
            ))
        );
    }
}

pub mod card {
    use super::*;

    use command::{W, H, Inner, w_const_mul, h_const_add};
    use sprite::{x_const_add_w};

    pub const WIDTH: W = W::clipped(CARD_WIDTH);
    pub const HEIGHT: H = H::clipped(CARD_HEIGHT);

    pub const IMAGE_W: W = W::clipped_inner(72);
    pub const IMAGE_H: H = H::clipped_inner(72);

    pub const IMAGE_BASE_X: sprite::X = sprite::X(0);
    pub const IMAGE_BASE_Y: sprite::Y = sprite::Y(0);

    pub const ZINGER_IMAGE_X: sprite::X =
        x_const_add_w(
            IMAGE_BASE_X,
            w_const_mul(IMAGE_W, models::SUIT_COUNT as Inner).get()
        );

    pub const BACKING_SPRITE_X: sprite::X =
        x_const_add_w(
            ZINGER_IMAGE_X,
            IMAGE_W.get()
        );
    pub const BACKING_SPRITE_BASE_Y: sprite::Y = sprite::Y(0);

    // TODO: refer to palette instead of hardcoding this.
    pub const TEXT_COLOUR: platform_types::ARGB = 0xFF222222;

    pub const IMAGE_W_OFFSET: W = W::clipped_inner(1);
    pub const IMAGE_H_OFFSET: H = H::clipped_inner(4);

    pub const LINE_W_OFFSET: W = CHAR_SPACING_W;
    pub const LINE_H_1_OFFSET: H = h_const_add(IMAGE_H_OFFSET, IMAGE_H);
    pub const LINE_H_2_OFFSET: H = h_const_add(LINE_H_1_OFFSET, CHAR_ADVANCE_H);
}

pub const WINDOW_CONTENT_OFFSET: unscaled::WH = unscaled::WH {
    w: unscaled::W(3),
    h: unscaled::H(3),
};

pub const CHAR_SPACING: u8 = 2;
pub const CHAR_SPACING_W: command::W = command::W::clipped_inner(CHAR_SPACING as _);
pub const CHAR_SPACING_H: command::H = command::H::clipped_inner(CHAR_SPACING as _);

const CHAR_ADVANCE_WIDTH: unscaled::Inner =
    CHAR_WIDTH as unscaled::Inner
    + CHAR_SPACING as unscaled::Inner;

pub const CHAR_ADVANCE_W: command::W = command::W::clipped_inner(CHAR_ADVANCE_WIDTH);

const CHAR_ADVANCE_HEIGHT: command::Inner =
    CHAR_HEIGHT as command::Inner
    + CHAR_SPACING as command::Inner;

pub const CHAR_ADVANCE_H: command::H = command::H::clipped_inner(CHAR_ADVANCE_HEIGHT);

// TODO `CharCount` type?
pub const WIDTH_IN_CHARS: command::Inner =
    command::WIDTH
    / CHAR_ADVANCE_WIDTH;

pub type TextLength = unscaled::Inner;

pub fn center_line_in_rect(
    text_length: TextLength,
    rect: unscaled::Rect
) -> unscaled::XY {
    let unscaled::Rect { x, y, w, h } = rect;

    let mut xy = unscaled::XY {
        x: x + (w / 2),
        y: y + (h / 2),
    };

    xy -= (CHAR_ADVANCE_W * text_length).get() / 2;
    xy -= CHAR_H.get() / 2;

    xy
}