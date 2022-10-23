use platform_types::{
    Command,
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
}

const CELLS_W: u8 = 4;
const CELLS_H: u8 = 4;
const CELLS_LENGTH: usize = CELLS_W as usize * CELLS_H as usize;

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
        // Pattern match so we get a compile error if the fields change.
        let &Command {
            rect: Rect {
                x,
                y,
                w,
                h,
            },
            sprite_xy,
            colour_override,
        } = command;

        u16(hash, x.get());
        u16(hash, y.get());
        u16(hash, w.get());
        u16(hash, h.get());

        u16(hash, sprite_xy.0.0);
        u16(hash, sprite_xy.1.0);

        bytes(hash, &colour_override.to_ne_bytes());
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

type Z = usize;

pub struct FrameBuffer {
    pub buffer: Vec<ARGB>,
    pub unscaled_buffer: Box<[ARGB; unscaled::LENGTH]>,
    pub unscaled_z_buffer: Box<[Z; unscaled::LENGTH]>,
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
            unscaled_buffer: Box::new([0; unscaled::LENGTH]),
            unscaled_z_buffer: Box::new([0; unscaled::LENGTH]),
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
    ) {
        let cells = self.current_mut();
        *cells = [<_>::default(); CELLS_LENGTH];

        for y in 0..CELLS_H {
            for x in 0..CELLS_H {
                let i = usize::from(y)
                        * usize::from(CELLS_W)
                        + usize::from(x);
                hash::u16(&mut cells[i], w);
                hash::u16(&mut cells[i], h);
            }
        }

        for command in commands {
            let mut hash = <_>::default();
            hash::u16(&mut hash, cells_size);
            hash::command(&mut hash, command);

            // update hash of overlapping cells
            let r = &command.rect;
            let r_x = clip::X::from(r.x);
            let r_y = clip::Y::from(r.y);
            let r_w = clip::W::from(r.w);
            let r_h = clip::H::from(r.h);

            for y in r_y / cells_size..=(r_y + r_h) / cells_size {
                for x in r_x / cells_size..=(r_x + r_w) / cells_size {
                    let i = usize::from(y)
                            * usize::from(CELLS_W)
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

        h_c.reset_then_hash_commands(&[], (CELLS_W.into(), CELLS_H.into()), 1);
        h_c.swap();
        h_c.reset_then_hash_commands(&[], (CELLS_W.into(), CELLS_H.into()), 1);

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
                w: unscaled::W(CELLS_W.into()),
                h: unscaled::H(CELLS_H.into()),
            },
            kind: Kind::Colour(0),
        }];

        h_c.reset_then_hash_commands(commands, (CELLS_W.into(), CELLS_H.into()), 1);
        h_c.swap();
        h_c.reset_then_hash_commands(commands, (CELLS_W.into(), CELLS_H.into()), 1);

        let (current, prev) = h_c.current_and_prev();

        assert_eq!(current, prev);
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum NeedsRedraw {
    No,
    Yes
}

#[cfg(all(
    target_arch = "x86_64",
    target_feature = "sse",
    target_feature = "sse2",
))]
mod wide {
    use super::*;
    pub const WIDTH: unscaled::Inner = 4;

    /// SAFETY: The pointers produced by the code generated by these macros must
    /// be valid to read 128 bytes from.
    /// We omit unsafe in the macro's code in order to encourage the user to notice
    /// the invariants they are expected to uphold.
    #[macro_export]
    macro_rules! _load {
        (
            $addr: expr
        ) => ({
            use core::arch::x86_64::__m128i as V;

            let addr: *const _ = $addr;
            core::arch::x86_64::_mm_loadu_si128(
                addr as *const V
            )
        });
        (
            $addr: expr,
            $index: expr $(,)?
        ) => ({
            use core::arch::x86_64::__m128i as V;

            let addr: *const _ = $addr;
            core::arch::x86_64::_mm_loadu_si128(
                addr.add($index) as *const V
            )
        });
    }
    pub use _load as load;

    /// SAFETY: The pointers produced by the code generated by these macros must
    /// be valid to write 128 bytes to.
    /// We omit unsafe in the macro's code in order to encourage the user to notice
    /// the invariants they are expected to uphold.
    #[macro_export]
    macro_rules! _store {
        (
            $vector: expr,
            $addr: expr $(,)?
        ) => ({
            use core::arch::x86_64::__m128i as V;

            let vector: V = $vector;
            let addr: *mut _ = $addr;
            core::arch::x86_64::_mm_storeu_si128(
                addr as *mut V,
                vector,
            )
        });
        (
            $vector: expr,
            $addr: expr,
            $index: expr $(,)?
        ) => ({
            use core::arch::x86_64::__m128i as V;
            let vector: V = $vector;

            let addr: *mut _ = $addr;
            core::arch::x86_64::_mm_storeu_si128(
                addr.add($index) as *mut V,
                vector,
            )
        });
    }
    pub use _store as store;
}

// TODO support wasm32
#[cfg(not(all(
    target_arch = "x86_64",
    target_feature = "sse",
    target_feature = "sse2",
)))]
mod wide {
    use super::*;
    pub const WIDTH: unscaled::Inner = 4;
}

pub fn render(
    frame_buffer: &mut FrameBuffer,
    commands: &[Command],
) -> NeedsRedraw {
    let mut output = NeedsRedraw::No;

    // The dimensions the commands are written in terms of.
    let src_w = unscaled::WIDTH;
    let src_h = unscaled::HEIGHT;

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

    let outer_clip_rect = clip::Rect {
        x: left_bar_width..(
            frame_buffer.width - right_bar_width
        ),
        y: top_bar_height..(
            frame_buffer.height - bottom_bar_height
        ),
    };

    let cells_size = core::cmp::max(
        (outer_clip_rect.width() + 1) / clip::W::from(CELLS_W),
        (outer_clip_rect.height() + 1) / clip::H::from(CELLS_H),
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

    let unscaled_cells_size = core::cmp::max(
        unscaled::WIDTH / clip::W::from(CELLS_W),
        unscaled::HEIGHT / clip::H::from(CELLS_H),
    );

    frame_buffer.cells.reset_then_hash_commands(
        commands,
        (frame_buffer.width, frame_buffer.height),
        unscaled_cells_size
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
    for cell_i in 0..CELLS_LENGTH {
        if cells[cell_i] == cells_prev[cell_i] {
            continue
        }
        output = NeedsRedraw::Yes;
    }

    if let NeedsRedraw::No = output {
        frame_buffer.cells.swap();
        return output;
    }

    for cell_y in 0..CELLS_H {
        for cell_x in 0..CELLS_W {
            let cell_i = usize::from(cell_y)
            * usize::from(CELLS_W)
            + usize::from(cell_x);

            if cells[cell_i] == cells_prev[cell_i] {
                continue
            }

            let cell_x = clip::X::from(cell_x);
            let cell_y = clip::Y::from(cell_y);
            let cell_clip_rect = clip::Rect {
                x: cell_x * unscaled_cells_size..(cell_x + 1) * unscaled_cells_size,
                y: cell_y * unscaled_cells_size..(cell_y + 1) * unscaled_cells_size,
            };

            // Hopefully this compiles to something not inefficent
            for y in cell_clip_rect.y.clone() {
                for x in cell_clip_rect.x.clone() {
                    let d_i = usize::from(y)
                        * usize::from(unscaled::WIDTH)
                        + usize::from(x);
                    if d_i < frame_buffer.unscaled_buffer.len() {
                        frame_buffer.unscaled_buffer[d_i] = colours::BLACK;
                        frame_buffer.unscaled_z_buffer[d_i] = 0;
                    }
                }
            }

            macro_rules! calc_clip_rect {
                ($rect: ident) => ({
                    let Rect {
                        x,
                        y,
                        w,
                        h,
                    } = $rect;

                    let x = clip::X::from(x);
                    let y = clip::Y::from(y);
                    let w = clip::W::from(w);
                    let h = clip::H::from(h);

                    let x_max = x + w;
                    let y_max = y + h;

                    let mut clip_rect = clip::Rect {
                        x: x..x_max,
                        y: y..y_max,
                    };

                    clip_rect
                })
            }

            for (
                command_i,
                &Command {
                    sprite_xy: (sprite_x, sprite_y),
                    colour_override,
                    rect,
                }
            ) in commands.iter().enumerate() {
                let z = command_i + 1;

                let clip_rect = calc_clip_rect!(rect);

                let w = clip::W::from(rect.w);

                let sprite_x = usize::from(sprite_x);
                let sprite_y = usize::from(sprite_y);

                let src_w = GFX_WIDTH as usize;

                let mut y_iter_count = 0;
                for y in clip_rect.y {
                    let mut x_iter_count = 0;
                    let mut x = clip_rect.x.start;

                    while x < clip_rect.x.end {
                        let mut should_write = [false; wide::WIDTH as usize];
                        let mut dest_indices = [0; wide::WIDTH as usize];
                        for i in 0..wide::WIDTH {
                            let i_usize = i as usize;
                            dest_indices[i_usize] = usize::from(y)
                                * usize::from(unscaled::WIDTH)
                                + usize::from(x) + i_usize;

                            should_write[i as usize] =
                                cell_clip_rect.contains(x + i, y)
                                && x + i < clip_rect.x.end
                                && dest_indices[i_usize] < frame_buffer.unscaled_z_buffer.len();
                        }

                        for i in 0usize..wide::WIDTH as usize {
                            if should_write[i]
                            {
                                let src_i =
                                    (sprite_y + y_iter_count) * src_w
                                    + (sprite_x + x_iter_count + i);

                                let mut gfx_colour: ARGB = GFX[src_i];
                                let is_full_alpha = gfx_colour >= 0xFF00_0000;
                                if is_full_alpha
                                // is not fully transparent
                                && colour_override > 0x00FF_FFFF
                                {
                                    gfx_colour = colour_override;
                                }

                                // If a pixel is fully opaque, then we
                                // can ignore all the pixels beneath it, so
                                // we set the z value. If it is at all
                                // transparent then we need to render
                                // whatever is behind it. So we do not set
                                // the z value.
                                if is_full_alpha {
                                    frame_buffer.unscaled_z_buffer[
                                        dest_indices[i]
                                    ] = z;
                                }
                            }
                        }
                        x_iter_count += wide::WIDTH as usize;
                        x += wide::WIDTH;
                    }

                    y_iter_count += 1;
                }
            }

            // The minimum z of the whole cell. If a given command's z is below this
            // value, then we can skip that entire command, for this cell.
            let mut min_z = Z::MAX;

            for y in cell_clip_rect.y.clone() {
                for x in cell_clip_rect.x.clone() {
                    let d_i = usize::from(y)
                    * usize::from(unscaled::WIDTH)
                    + usize::from(x);

                    if d_i < frame_buffer.unscaled_z_buffer.len() {
                        min_z = core::cmp::min(
                            min_z,
                            frame_buffer.unscaled_z_buffer[d_i]
                        );
                    }
                }
            }

            for (
                command_i,
                &Command {
                    sprite_xy: (sprite_x, sprite_y),
                    colour_override,
                    rect,
                }
            ) in commands.iter().enumerate().skip(min_z.saturating_sub(1)) {
                let z = command_i + 1;

                let clip_rect = calc_clip_rect!(rect);

                let w = clip::W::from(rect.w);

                let sprite_x = usize::from(sprite_x);
                let sprite_y = usize::from(sprite_y);

                let src_w = GFX_WIDTH as usize;

                let mut y_iter_count = 0;
                for y in clip_rect.y {
                    let mut x_iter_count = 0;
                    let mut x = clip_rect.x.start;

                    while x < clip_rect.x.end {
                        let mut should_write = [false; wide::WIDTH as usize];
                        let mut dest_indices = [0; wide::WIDTH as usize];
                        for i in 0..wide::WIDTH {
                            let i_usize = i as usize;
                            dest_indices[i_usize] = usize::from(y)
                                * usize::from(unscaled::WIDTH)
                                + usize::from(x) + i_usize;

                            should_write[i_usize] =
                                cell_clip_rect.contains(x + i, y)
                                && x + i < clip_rect.x.end
                                && dest_indices[i_usize] < frame_buffer.unscaled_buffer.len()
                                && z >= frame_buffer.unscaled_z_buffer[dest_indices[i_usize]];
                        }

                        // SAFETY: The pointers produced by the code generated by
                        // these macros are valid to write 128 bytes to.
                        let unders = unsafe {
                            wide::load!(
                                frame_buffer.unscaled_buffer.as_ptr(),
                                dest_indices[0],
                            )
                        };

                        // These arrays should eventually be unneeded.
                        let mut under_array = [0; wide::WIDTH as usize];
                        // SAFETY: The pointers produced by the code generated by
                        // these macros are valid to read 128 bytes from.
                        unsafe {
                            wide::store!(
                                unders,
                                &mut under_array
                            );
                        }

                        let mut output = under_array.clone();

                        for i in 0usize..wide::WIDTH as usize {
                            if should_write[i]
                            {
                                let src_i =
                                    (sprite_y + y_iter_count) * src_w
                                    + (sprite_x + x_iter_count + i);

                                let mut gfx_colour: ARGB = GFX[src_i];
                                let is_full_alpha = gfx_colour >= 0xFF00_0000;
                                if is_full_alpha
                                // is not fully transparent
                                && colour_override > 0x00FF_FFFF
                                {
                                    gfx_colour = colour_override;
                                }

                                fn f32_to_u8(x: f32) -> u8 {
                                    // This saturates instead of being UB
                                    // as of rust 1.45.0
                                    x as u8
                                }
                                // Interprets 1.0 as full bright.
                                fn linear_to_gamma(x: f32) -> u8 {
                                    f32_to_u8(255. * x.sqrt())
                                }

                                fn gamma_to_linear(x: u8) -> f32 {
                                    let f = (x as f32)/255.;
                                    f * f
                                }

                                // `_g` for gfx.
                                let a_g = ((gfx_colour >> 24) & 255) as u8;
                                let r_g = ((gfx_colour >> 16) & 255) as u8;
                                let g_g = ((gfx_colour >>  8) & 255) as u8;
                                let b_g = ((gfx_colour      ) & 255) as u8;

                                let under = under_array[i];

                                // `_u` for under.
                                let a_u = ((under >> 24) & 255) as u8;
                                let r_u = ((under >> 16) & 255) as u8;
                                let g_u = ((under >>  8) & 255) as u8;
                                let b_u = ((under      ) & 255) as u8;

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

                                output[i] =
                                      (ARGB::from(a_o) << 24)
                                    | (ARGB::from(r_o) << 16)
                                    | (ARGB::from(g_o) <<  8)
                                    | (ARGB::from(b_o)      );
                            }
                        }

                        // SAFETY: The pointers produced by the code generated by
                        // these macros are valid to read 128 bytes from.
                        let output = unsafe {
                            wide::load!(
                                output.as_ptr()
                            )
                        };

                        // This check is necessary to prevent commands that include
                        // parts that are outside the bounds of the buffer from
                        // causing a segmentation fault. Not all cases where this
                        // condition is true would cause a segmentation fault, but
                        // this condition is true for all known cases that would
                        // cause a segmentation fault.
                        if should_write != [false, false, false, false] {
                            // SAFETY: The pointers produced by the code generated by
                            // these macros are valid to write 128 bytes to.
                            unsafe {
                                wide::store!(
                                    output,
                                    frame_buffer.unscaled_buffer.as_mut_ptr(),
                                    dest_indices[0],
                                );
                            }
                        }

                        x_iter_count += wide::WIDTH as usize;
                        x += wide::WIDTH;
                    }

                    y_iter_count += 1;
                }
            }
        }
    }


    let mut src_i = 0;
    let mut src_i_row_start = src_i;
    let mut y_remaining = multiplier;
    for y in outer_clip_rect.y {
        let mut x_remaining = multiplier;

        src_i_row_start = src_i;

        for x in outer_clip_rect.x.clone() {
            let d_i = usize::from(y)
            * usize::from(frame_buffer.width)
            + usize::from(x);

            if d_i < frame_buffer.buffer.len() {
                frame_buffer.buffer[d_i] =
                    frame_buffer.unscaled_buffer[src_i];
            }

            x_remaining -= 1;
            if x_remaining == 0 {
                src_i += 1;
                x_remaining = multiplier;
            }
        }

        src_i = src_i_row_start;

        y_remaining -= 1;
        if y_remaining == 0 {
            y_remaining = multiplier;
            let src_w = usize::from(unscaled::WIDTH);
            src_i += src_w;
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
