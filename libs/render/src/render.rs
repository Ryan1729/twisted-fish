use platform_types::{
    Command,
    Kind,
    GFX_WIDTH,
    FONT_WIDTH,
    ARGB,
    unscaled::{self, Rect},
};

use assets::{GFX, FONT, FONT_TRANSPARENT};

pub mod clip {
    use core::ops::Range;
    pub type X = u16;
    pub type Y = u16;
    pub type W = u16;
    pub type H = u16;

    #[derive(Clone, Debug)]
    pub struct Rect {
        pub x: Range<X>,
        pub y: Range<Y>,
    }

    impl Rect {
        pub fn width(&self) -> W {
            self.x.end - self.x.start
        }

        pub fn height(&self) -> H {
            self.y.end - self.y.start
        }

        pub fn contains(&self, x: X, y: Y) -> bool {
            self.x.start <= x
            && x < self.x.end
            && self.y.start <= y
            && y < self.y.end
        }
    }

    pub fn to(clipped: &mut Rect, clipper: &Rect) {
        use core::cmp::{max, min};

        clipped.x = max(clipped.x.start, clipper.x.start)
            ..min(clipped.x.end, clipper.x.end);

        clipped.y = max(clipped.y.start, clipper.y.start)
            ..min(clipped.y.end, clipper.y.end);
    }
}

const CELLS_X: u8 = 4;
const CELLS_Y: u8 = 4;
const CELLS_LENGTH: usize = CELLS_X as usize * CELLS_Y as usize;

/// Implements a 32 bit FNV-1a hash
mod hash {
    use super::*;

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct Cell(u32);

    impl Default for Cell {
        fn default() -> Cell {
            Cell(0x811c9dc5)
        }
    }

    pub fn byte(hash: &mut Cell, byte: u8) {
        hash.0 ^= byte as u32;
        hash.0 = hash.0.wrapping_mul(0x01000193);
    }

    pub fn bytes(hash: &mut Cell, bytes: &[u8]) {
        for &b in bytes {
            byte(hash, b);
        }
    }

    pub fn u16(hash: &mut Cell, u16: u16) {
        // We prioritize speed over portablilty of hashes between architechtures,
        // which we expect wouldn't come up anyway. Hence `to_ne_bytes`.
        bytes(hash, &u16.to_ne_bytes());
    }

    pub fn command(hash: &mut Cell, command: &Command) {
        use Kind::*;

        // Pattern match so we get a compile error if the fields change.
        let &Command {
            rect: Rect {
                x,
                y,
                w,
                h,
            },
            kind,
        } = command;

        u16(hash, x.get());
        u16(hash, y.get());
        u16(hash, w.get());
        u16(hash, h.get());

        match kind {
            Gfx((x, y)) => {
                byte(hash, 0);
                u16(hash, x.0);
                u16(hash, y.0);
            },
            Font((x, y), i) => {
                byte(hash, 1);
                u16(hash, x.0);
                u16(hash, y.0);
                byte(hash, i);
            },
            Colour(i) => { bytes(hash, &[2, i]); },
        };
    }

    pub fn hash(hash: &mut Cell, hashed: Cell) {
        // We prioritize speed over portablilty of hashes between architechtures,
        // which we expect wouldn't come up anyway. Hence `to_ne_bytes`.
        bytes(hash, &hashed.0.to_ne_bytes());
    }
}

type Cells = [hash::Cell; CELLS_LENGTH];

#[derive(Copy, Clone, Default)]
enum CurrentCells {
    #[default]
    A,
    B
}

pub struct FrameBuffer {
    pub buffer: Vec<u32>,
    pub width: clip::W,
    pub height: clip::H,
    pub cells: HashCells,
}

impl FrameBuffer {
    pub fn from_size((width, height): (clip::W, clip::H)) -> Self {
        Self {
            buffer: Vec::with_capacity(
                usize::from(width) * usize::from(height)
            ),
            width,
            height,
            cells: HashCells::default(),
        }
    }
}

#[derive(Default)]
pub struct HashCells {
    current_cells: CurrentCells,
    cells_a: Cells,
    cells_b: Cells,
}

impl HashCells {
    fn current_mut(&mut self) -> &mut Cells {
        match self.current_cells {
            CurrentCells::A => &mut self.cells_a,
            CurrentCells::B => &mut self.cells_b,
        }
    }

    fn current_and_prev(&self) -> (&Cells, &Cells) {
        match self.current_cells {
            CurrentCells::A => (&self.cells_a, &self.cells_b),
            CurrentCells::B => (&self.cells_b, &self.cells_a),
        }
    }

    fn swap(&mut self) {
        self.current_cells = match self.current_cells {
            CurrentCells::A => CurrentCells::B,
            CurrentCells::B => CurrentCells::A,
        };
    }

    fn reset_then_hash_commands(
        &mut self,
        commands: &[Command],
        (w, h): (u16, u16),
        cells_size: u16,
        multiplier: u16,
    ) {
        let cells = self.current_mut();
        *cells = [<_>::default(); CELLS_LENGTH];

        for y in 0..CELLS_Y {
            for x in 0..CELLS_Y {
                let i = usize::from(y)
                        * usize::from(CELLS_X)
                        + usize::from(x);
                hash::u16(&mut cells[i], w);
                hash::u16(&mut cells[i], h);
            }
        }

        for command in commands {
            let mut hash = <_>::default();
            hash::u16(&mut hash, multiplier);
            hash::u16(&mut hash, cells_size);
            hash::command(&mut hash, command);

            // update hash of overlapping cells
            let r = &command.rect;
            let r_x = clip::X::from(r.x) * multiplier;
            let r_y = clip::Y::from(r.y) * multiplier;
            let r_w = clip::W::from(r.w) * multiplier;
            let r_h = clip::H::from(r.h) * multiplier;

            for y in r_y / cells_size..=(r_y + r_h) / cells_size {
                for x in r_x / cells_size..=(r_x + r_w) / cells_size {
                    let i = usize::from(y)
                            * usize::from(CELLS_X)
                            + usize::from(x);
                    // We want to allow drawing things that are partially offscreen.
                    if i < cells.len() {
                        hash::hash(
                            &mut cells[i],
                            hash
                        );
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod reset_then_hash_commands_around_a_swap_produces_identical_current_and_prev_cells {
    use super::*;

    #[test]
    fn on_the_empty_slice() {
        let mut h_c = HashCells::default();

        h_c.reset_then_hash_commands(&[], (CELLS_X.into(), CELLS_Y.into()), 1, 1);
        h_c.swap();
        h_c.reset_then_hash_commands(&[], (CELLS_X.into(), CELLS_Y.into()), 1, 1);

        let (current, prev) = h_c.current_and_prev();

        assert_eq!(current, prev);
    }

    #[test]
    fn on_this_one_element_slice() {
        let mut h_c = HashCells::default();

        let commands = &[Command {
            rect: Rect {
                x: unscaled::X(0),
                y: unscaled::Y(0),
                w: unscaled::W(CELLS_X),
                h: unscaled::H(CELLS_Y),
            },
            kind: Kind::Colour(0),
        }];

        h_c.reset_then_hash_commands(commands, (CELLS_X.into(), CELLS_Y.into()), 1, 1);
        h_c.swap();
        h_c.reset_then_hash_commands(commands, (CELLS_X.into(), CELLS_Y.into()), 1, 1);

        let (current, prev) = h_c.current_and_prev();

        assert_eq!(current, prev);
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum NeedsRedraw {
    No,
    Yes
}

pub fn render(
    frame_buffer: &mut FrameBuffer,
    commands: &[Command],
) -> NeedsRedraw {
    let mut output = NeedsRedraw::No;

    // The dimensions the commands are written in terms of.
    let src_w = unscaled::WIDTH.into();
    let src_h = unscaled::HEIGHT.into();

    if frame_buffer.width < src_w
    || frame_buffer.height < src_h {
        frame_buffer.width = src_w;
        frame_buffer.height = src_h;
    }

    let width_multiplier = frame_buffer.width / src_w;
    let height_multiplier = frame_buffer.height / src_h;
    let multiplier = core::cmp::min(width_multiplier, height_multiplier);
    if multiplier == 0 {
        debug_assert!(multiplier != 0);
        return output;
    }

    let vertical_bars_width: clip::W = frame_buffer.width - (multiplier * src_w);

    let left_bar_width: clip::W = (vertical_bars_width + 1) / 2;

    let right_bar_width: clip::W = vertical_bars_width / 2;

    let horizontal_bars_height: clip::H = frame_buffer.height - (multiplier * src_h);

    let top_bar_height: clip::H = (horizontal_bars_height + 1) / 2;

    let bottom_bar_height: clip::H = horizontal_bars_height / 2;

    let d_w = frame_buffer.width;

    let outer_clip_rect = clip::Rect {
        x: left_bar_width..(
            frame_buffer.width - right_bar_width
        ),
        y: top_bar_height..(
            frame_buffer.height - bottom_bar_height
        ),
    };

    let cells_size = core::cmp::max(
        (outer_clip_rect.width() + 1) / clip::W::from(CELLS_X),
        (outer_clip_rect.height() + 1) / clip::H::from(CELLS_Y),
    );

    // Cached software rendering based on:
    // https://rxi.github.io/cached_software_rendering.html
    //
    // TODO Attempt to merge adjacent regions for cells that are
    // adjacent and render merged regions only once each.
    //   Compute boolean mask of cells that need redrawing (`[bool; CELLS_LENGTH]`)
    //   Scan through mask until we find a true, then try to expand a rectangle by
    //     going right and down from that point alternately.
    //   Render once with expanded cell_clip_rect
    //   Mark the rendered cells as false, keep scanning.

    frame_buffer.cells.reset_then_hash_commands(
        commands,
        (frame_buffer.width, frame_buffer.height),
        cells_size,
        multiplier
    );

    let expected_length = usize::from(frame_buffer.width)
    * usize::from(frame_buffer.height);

    if frame_buffer.buffer.len() != expected_length {
        frame_buffer.buffer.clear();
        // Hopefully this compiles to something not inefficent
        frame_buffer.buffer.reserve(expected_length);
        for _ in 0..expected_length {
            frame_buffer.buffer.push(0);
        }
    }

    let (cells, cells_prev) = frame_buffer.cells.current_and_prev();
    for cell_y in 0..CELLS_Y {
        for cell_x in 0..CELLS_X {
            let i = usize::from(cell_y)
            * usize::from(CELLS_X)
            + usize::from(cell_x);

            if cells[i] == cells_prev[i] {
                continue
            }
            output = NeedsRedraw::Yes;

            let cell_x = clip::X::from(cell_x);
            let cell_y = clip::Y::from(cell_y);
            let cell_clip_rect = clip::Rect {
                x: cell_x * cells_size + left_bar_width..(cell_x + 1) * cells_size + left_bar_width,
                y: cell_y * cells_size + top_bar_height..(cell_y + 1) * cells_size + top_bar_height,
            };

            for &Command {
                kind,
                rect: Rect {
                    x: d_x,
                    y: d_y,
                    w,
                    h,
                },
            } in commands {
                let d_x = clip::X::from(d_x);
                let d_y = clip::Y::from(d_y);
                let w = clip::W::from(w);
                let h = clip::H::from(h);

                let d_x_max = d_x + w;
                let d_y_max = d_y + h;

                let x_range = (d_x * multiplier + left_bar_width)..(
                    d_x_max * multiplier + left_bar_width
                );

                let mut clip_rect = clip::Rect {
                    x: x_range.clone(),
                    y: (d_y * multiplier + top_bar_height)..(
                        d_y_max * multiplier + top_bar_height
                    ),
                };

                clip::to(&mut clip_rect, &outer_clip_rect);

                macro_rules! advance {
                    ($src_i: ident, $x_remaining: ident) => {
                        $x_remaining -= 1;
                        if $x_remaining == 0 {
                            $src_i += 1;
                            $x_remaining = multiplier;
                        }
                    }
                }

                match kind {
                    Kind::Gfx((sprite_x, sprite_y)) => {
                        let sprite_x = usize::from(sprite_x);
                        let sprite_y = usize::from(sprite_y);

                        let src_w = GFX_WIDTH as usize;

                        let mut src_i = sprite_y * src_w + sprite_x;
                        let mut y_remaining = multiplier;
                        for y in clip_rect.y {
                            let mut x_remaining = multiplier;
                            for x in clip_rect.x.clone() {
                                let gfx_colour: ARGB = GFX[src_i];

                                if cell_clip_rect.contains(x, y)
                                {
                                    let d_i = usize::from(y)
                                    * usize::from(d_w)
                                    + usize::from(x);
                                    if d_i < frame_buffer.buffer.len() {
                                        fn f32_to_u8(x: f32) -> u8 {
                                            // This saturates instead of being UB
                                            // as of rust 1.45.0
                                            x as u8
                                        }
                                        // Interprets 1.0 as full bright.
                                        fn linear_to_gamma(x: f32) -> u8 {
                                            f32_to_u8(255. * x.powf(1./2.2))
                                        }

                                        fn gamma_to_linear(x: u8) -> f32 {
                                            ((x as f32)/255.).powf(2.2)
                                        }

                                        let under = frame_buffer.buffer[d_i];

                                        // `_g` for gfx.
                                        let a_g = ((gfx_colour >> 24) & 255) as u8;
                                        let r_g = ((gfx_colour >> 16) & 255) as u8;
                                        let g_g = ((gfx_colour >>  8) & 255) as u8;
                                        let b_g = ((gfx_colour >>  0) & 255) as u8;

                                        // `_u` for under.
                                        let a_u = ((under >> 24) & 255) as u8;
                                        let r_u = ((under >> 16) & 255) as u8;
                                        let g_u = ((under >>  8) & 255) as u8;
                                        let b_u = ((under >>  0) & 255) as u8;

                                        let a_g = gamma_to_linear(a_g);
                                        let r_g = gamma_to_linear(r_g);
                                        let g_g = gamma_to_linear(g_g);
                                        let b_g = gamma_to_linear(b_g);

                                        let a_u = gamma_to_linear(a_u);
                                        let r_u = gamma_to_linear(r_u);
                                        let g_u = gamma_to_linear(g_u);
                                        let b_u = gamma_to_linear(b_u);

                                        // `_o` for output.
                                        let a_o = a_g + a_u * (1. - a_g);
                                        let r_o = (r_g * a_g + r_u * (1. - a_g)) / a_o;
                                        let g_o = (g_g * a_g + g_u * (1. - a_g)) / a_o;
                                        let b_o = (b_g * a_g + b_u * (1. - a_g)) / a_o;

                                        let a_o = linear_to_gamma(a_o);
                                        let r_o = linear_to_gamma(r_o);
                                        let g_o = linear_to_gamma(g_o);
                                        let b_o = linear_to_gamma(b_o);

                                        let output =
                                              (ARGB::from(a_o) << 24)
                                            | (ARGB::from(r_o) << 16)
                                            | (ARGB::from(g_o) <<  8)
                                            | (ARGB::from(b_o) <<  0);

                                        frame_buffer.buffer[d_i] = output;
                                    }
                                }

                                advance!(src_i, x_remaining);
                            }

                            // If we would have went off the edge, advance `src_i`
                            // as if we actually drew past the edge.
                            for _ in clip_rect.x.end..x_range.end {
                                advance!(src_i, x_remaining);
                            }

                            // Go back to the beginning of the row.
                            src_i -= usize::from(w);

                            y_remaining -= 1;
                            if y_remaining == 0 {
                                y_remaining = multiplier;
                                src_i += src_w;
                            }
                        }
                    },
                    Kind::Font((sprite_x, sprite_y), colour) => {
                        let sprite_x = usize::from(sprite_x);
                        let sprite_y = usize::from(sprite_y);

                        let src_w = FONT_WIDTH as usize;

                        let mut src_i = sprite_y * src_w + sprite_x;
                        let mut y_remaining = multiplier;
                        for y in clip_rect.y {
                            let mut x_remaining = multiplier;
                            for x in clip_rect.x.clone() {
                                let font_pixel_colour = FONT[src_i];

                                if font_pixel_colour != FONT_TRANSPARENT
                                && cell_clip_rect.contains(x, y) {
                                    let d_i = usize::from(y)
                                    * usize::from(d_w)
                                    + usize::from(x);
                                    if d_i < frame_buffer.buffer.len() {
                                        frame_buffer.buffer[d_i] = PALETTE[colour as usize & 15];
                                    }
                                }

                                advance!(src_i, x_remaining);
                            }

                            // If we would have went off the edge, advance `src_i`
                            // as if we actually drew past the edge.
                            for _ in clip_rect.x.end..x_range.end {
                                advance!(src_i, x_remaining);
                            }

                            // Go back to the beginning of the row.
                            src_i -= usize::from(w);

                            y_remaining -= 1;
                            if y_remaining == 0 {
                                y_remaining = multiplier;
                                src_i += src_w;
                            }
                        }
                    },
                    Kind::Colour(colour) => {
                        for y in clip_rect.y {
                            for x in clip_rect.x.clone() {
                                if cell_clip_rect.contains(x, y) {
                                    let index = usize::from(y)
                                    * usize::from(d_w)
                                    + usize::from(x);
                                    if index < frame_buffer.buffer.len() {
                                        frame_buffer.buffer[index] = PALETTE[colour as usize & 15];
                                    }
                                }
                            }
                        }
                    }
                };
            }
        }
    }

    frame_buffer.cells.swap();

    output
}

// reportedly colourblind friendly colours
// https://twitter.com/ea_accessible/status/968595073184092160

mod colours {
    use super::ARGB;

    pub const BLUE: ARGB = 0xFF3352E1;
    pub const GREEN: ARGB = 0xFF30B06E;
    pub const RED: ARGB = 0xFFDE4949;
    pub const YELLOW: ARGB = 0xFFFFB937;
    pub const PURPLE: ARGB = 0xFF533354;
    #[allow(unused)]
    pub const GREY: ARGB = 0xFF5A7D8B;
    #[allow(unused)]
    pub const GRAY: ARGB = GREY;
    pub const WHITE: ARGB = 0xFFEEEEEE;
    pub const BLACK: ARGB = 0xFF222222;
}

use colours::*;

#[rustfmt::skip]
const PALETTE: [ARGB; 8] = [
    BLUE,
    GREEN,
    RED,
    YELLOW,
    PURPLE,
    GREY,
    WHITE,
    BLACK,
];
