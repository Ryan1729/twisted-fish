#![deny(clippy::shadow_unrelated)]
use platform_types::{
    Command,
    GFX_WIDTH,
    ARGB,
    colours,
    command::{self, Rect},
    sprite,
    unscaled,
};

use assets::GFX;

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
    }
}

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
                x_min,
                y_min,
                x_max,
                y_max,
            },
            sprite_xy,
            colour_override,
        } = command;

        u16(hash, x_min.get().get());
        u16(hash, y_min.get().get());
        u16(hash, x_max.get().get());
        u16(hash, y_max.get().get());

        u16(hash, sprite_xy.x.0);
        u16(hash, sprite_xy.y.0);

        bytes(hash, &colour_override.to_ne_bytes());
    }
}

#[derive(Copy, Clone, Default)]
enum CurrentCells {
    #[default]
    A,
    B
}

pub struct FrameBuffer {
    pub buffer: Vec<ARGB>,
    pub unscaled_buffer: Box<[ARGB; command::LENGTH]>,
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
            unscaled_buffer: Box::new([0; command::LENGTH]),
            width,
            height,
            cells: HashCells::default(),
        }
    }
}

#[derive(Default)]
pub struct HashCells {
    current_cells: CurrentCells,
    cells_a: hash::Cell,
    cells_b: hash::Cell,
}

impl HashCells {
    fn current_mut(&mut self) -> &mut hash::Cell {
        match self.current_cells {
            CurrentCells::A => &mut self.cells_a,
            CurrentCells::B => &mut self.cells_b,
        }
    }

    fn current_and_prev(&self) -> (&hash::Cell, &hash::Cell) {
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
    ) {
        let cells = self.current_mut();
        *cells = <_>::default();

        hash::u16(cells, w);
        hash::u16(cells, h);

        for command in commands {
            hash::command(cells, command);
        }
    }
}

#[cfg(test)]
mod reset_then_hash_commands_around_a_swap_produces_identical_current_and_prev_cells {
    use super::*;
    const CELLS_W: u16 = 16;
    const CELLS_H: u16 = 16;
    #[test]
    fn on_the_empty_slice() {
        let mut h_c = HashCells::default();

        h_c.reset_then_hash_commands(&[], (CELLS_W, CELLS_H));
        h_c.swap();
        h_c.reset_then_hash_commands(&[], (CELLS_W, CELLS_H));

        let (current, prev) = h_c.current_and_prev();

        assert_eq!(current, prev);
    }

    #[test]
    fn on_this_one_element_slice() {
        let mut h_c = HashCells::default();

        let mut command = command::Command::default();
        command.rect = command::Rect::from_unscaled(
            unscaled::Rect {
                x: unscaled::X(0),
                y: unscaled::Y(0),
                w: unscaled::W(CELLS_W),
                h: unscaled::H(CELLS_H),
            }
        );

        let commands = &[command];

        h_c.reset_then_hash_commands(commands, (CELLS_W, CELLS_H));
        h_c.swap();
        h_c.reset_then_hash_commands(commands, (CELLS_W, CELLS_H));

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
            $addr: expr $(,)?
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

    // SAFETY: This operation is only unsafe because it uses intrinsics, and we wrap
    // this module with `cfg`, so this is safe to use in all cases where it compiles.
    #[macro_export]
    macro_rules! _pick_via_mask {
        (
            $zero_case: expr,
            $one_case: expr,
            $mask: expr $(,)?
        ) => ({
            let mask = $mask;
            #[allow(unused_unsafe)]
            unsafe {
                core::arch::x86_64::_mm_or_si128(
                    core::arch::x86_64::_mm_and_si128(
                        mask,
                        $one_case,
                    ),
                    core::arch::x86_64::_mm_andnot_si128(
                        mask,
                        $zero_case,
                    )
                )
            }
        });
    }
    pub use _pick_via_mask as pick_via_mask;

    #[macro_export]
    macro_rules! _i32x4 {
        (
            $e0: expr,
            $e1: expr,
            $e2: expr,
            $e3: expr $(,)?
        ) => ({#[allow(unused_unsafe)]
            unsafe {
                core::arch::x86_64::_mm_set_epi32(
                    $e3,
                    $e2,
                    $e1,
                    $e0,
                )
            }
        });
    }
    pub use _i32x4 as i32x4;

    #[macro_export]
    macro_rules! _and {
        (
            $a: expr,
            $b: expr $(,)?
        ) => ({#[allow(unused_unsafe)]
            unsafe {
                core::arch::x86_64::_mm_and_si128($a, $b)
            }
        });
    }
    pub use _and as and;

    #[macro_export]
    macro_rules! _and_not {
        (
            $just_anded: expr,
            $notted: expr $(,)?
        ) => ({#[allow(unused_unsafe)]
            unsafe {
                // "Compute the bitwise NOT of 128 bits (representing integer data) in
                // `a` and then AND with `b`, and store the result in dst."
                // I prefer to not the second operand, so `a` and `b` are switched here.
                core::arch::x86_64::_mm_andnot_si128($notted, $just_anded)
            }
        });
    }
    pub use _and_not as and_not;

    #[macro_export]
    macro_rules! _or {
        (
            $a: expr,
            $b: expr $(,)?
        ) => ({#[allow(unused_unsafe)]
            unsafe {
                core::arch::x86_64::_mm_or_si128($a, $b)
            }
        });
    }
    pub use _or as or;

    #[macro_export]
    macro_rules! _left_shift_32 {
        (
            $a: expr,
            $imm8: expr $(,)?
        ) => ({#[allow(unused_unsafe)]
            unsafe {
                core::arch::x86_64::_mm_slli_epi32($a, $imm8)
            }
        });
    }
    pub use _left_shift_32 as left_shift_32;

    #[macro_export]
    macro_rules! _right_shift_32 {
        (
            $a: expr,
            $imm8: expr $(,)?
        ) => ({#[allow(unused_unsafe)]
            unsafe {
                core::arch::x86_64::_mm_srli_epi32($a, $imm8)
            }
        });
    }
    pub use _right_shift_32 as right_shift_32;

    #[macro_export]
    macro_rules! _f32_to_u32 {
        (
            $a: expr $(,)?
        ) => ({#[allow(unused_unsafe)]
            unsafe {
                core::arch::x86_64::_mm_cvtps_epi32($a)
            }
        });
    }
    pub use _f32_to_u32 as f32_to_u32;

    #[macro_export]
    macro_rules! _u32_to_f32 {
        (
            $a: expr $(,)?
        ) => ({#[allow(unused_unsafe)]
            unsafe {
                core::arch::x86_64::_mm_cvtepi32_ps($a)
            }
        });
    }
    pub use _u32_to_f32 as u32_to_f32;

    #[macro_export]
    macro_rules! _i32 {
        (
            $a: expr $(,)?
        ) => ({#[allow(unused_unsafe)]
            unsafe {
                core::arch::x86_64::_mm_set1_epi32($a)
            }
        });
    }
    pub use _i32 as i32;

    #[macro_export]
    macro_rules! _f32 {
        (
            $a: expr $(,)?
        ) => ({#[allow(unused_unsafe)]
            unsafe {
                core::arch::x86_64::_mm_set_ps1($a)
            }
        });
    }
    pub use _f32 as f32;

    #[macro_export]
    macro_rules! _add_f32 {
        (
            $a: expr,
            $b: expr $(,)?
        ) => ({#[allow(unused_unsafe)]
            unsafe {
                core::arch::x86_64::_mm_add_ps($a, $b)
            }
        });
    }
    pub use _add_f32 as add_f32;

    #[macro_export]
    macro_rules! _add_i32 {
        (
            $a: expr,
            $b: expr $(,)?
        ) => ({#[allow(unused_unsafe)]
            unsafe {
                core::arch::x86_64::_mm_add_epi32($a, $b)
            }
        });
    }
    pub use _add_i32 as add_i32;

    #[macro_export]
    macro_rules! _sub {
        (
            $a: expr,
            $b: expr $(,)?
        ) => ({#[allow(unused_unsafe)]
            unsafe {
                core::arch::x86_64::_mm_sub_ps($a, $b)
            }
        });
    }
    pub use _sub as sub;

    #[macro_export]
    macro_rules! _mul {
        (
            $a: expr,
            $b: expr $(,)?
        ) => ({#[allow(unused_unsafe)]
            unsafe {
                core::arch::x86_64::_mm_mul_ps($a, $b)
            }
        });
    }
    pub use _mul as mul;

    #[macro_export]
    macro_rules! _recip {
        (
            $a: expr $(,)?
        ) => ({#[allow(unused_unsafe)]
            unsafe {
                core::arch::x86_64::_mm_rcp_ps($a)
            }
        });
    }
    pub use _recip as recip;

    #[macro_export]
    macro_rules! _sqrt {
        (
            $a: expr $(,)?
        ) => ({#[allow(unused_unsafe)]
            unsafe {
                core::arch::x86_64::_mm_sqrt_ps($a)
            }
        });
    }
    pub use _sqrt as sqrt;

    #[macro_export]
    macro_rules! _lt_mask_32 {
        (
            $lhs: expr,
            $rhs: expr $(,)?
        ) => ({#[allow(unused_unsafe)]
            unsafe {
                core::arch::x86_64::_mm_cmplt_epi32($lhs, $rhs)
            }
        });
    }
    pub use _lt_mask_32 as lt_mask_32;

    #[macro_export]
    macro_rules! _eq_mask_u32 {
        (
            $lhs: expr,
            $rhs: expr $(,)?
        ) => ({#[allow(unused_unsafe)]
            unsafe {
                core::arch::x86_64::_mm_cmpeq_epi32($lhs, $rhs)
            }
        });
    }
    pub use _eq_mask_u32 as eq_mask_u32;

    #[macro_export]
    macro_rules! _gt_mask_32 {
        (
            $lhs: expr,
            $rhs: expr $(,)?
        ) => ({#[allow(unused_unsafe)]
            unsafe {
                core::arch::x86_64::_mm_cmpgt_epi32($lhs, $rhs)
            }
        });
    }
    pub use _gt_mask_32 as gt_mask_32;
}

#[cfg(all(
    target_arch = "wasm32",
    target_feature = "simd128",
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
            $addr: expr $(,)?
        ) => ({
            
            use core::arch::wasm32::v128 as V;

            let addr: *const _ = $addr;
            core::arch::wasm32::v128_load(
                addr as *const V
            )
        });
        (
            $addr: expr,
            $index: expr $(,)?
        ) => ({
            use core::arch::wasm32::v128 as V;

            let addr: *const _ = $addr;
            core::arch::wasm32::v128_load(
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
            use core::arch::wasm32::v128 as V;

            let vector: V = $vector;
            let addr: *mut _ = $addr;
            core::arch::wasm32::v128_store(
                addr as *mut V,
                vector,
            )
        });
        (
            $vector: expr,
            $addr: expr,
            $index: expr $(,)?
        ) => ({
            use core::arch::wasm32::v128 as V;
            let vector: V = $vector;

            let addr: *mut _ = $addr;
            core::arch::wasm32::v128_store(
                addr.add($index) as *mut V,
                vector,
            )
        });
    }
    pub use _store as store;

    // SAFETY: This operation is only unsafe because it uses intrinsics, and we wrap
    // this module with `cfg`, so this is safe to use in all cases where it compiles.
    #[macro_export]
    macro_rules! _pick_via_mask {
        (
            $zero_case: expr,
            $one_case: expr,
            $mask: expr $(,)?
        ) => ({
            let mask = $mask;
            #[allow(unused_unsafe)]
            unsafe {
                core::arch::wasm32::v128_or(
                    core::arch::wasm32::v128_and(
                        mask,
                        $one_case,
                    ),
                    core::arch::wasm32::v128_andnot(
                        $zero_case,
                        mask,
                    )
                )
            }
        });
    }
    pub use _pick_via_mask as pick_via_mask;

    #[macro_export]
    macro_rules! _i32x4 {
        (
            $e0: expr,
            $e1: expr,
            $e2: expr,
            $e3: expr $(,)?
        ) => ({#[allow(unused_unsafe)]
            unsafe {
                core::arch::wasm32::i32x4(
                    $e0,
                    $e1,
                    $e2,
                    $e3,
                )
            }
        });
    }
    pub use _i32x4 as i32x4;

    #[macro_export]
    macro_rules! _and {
        (
            $a: expr,
            $b: expr $(,)?
        ) => ({#[allow(unused_unsafe)]
            unsafe {
                core::arch::wasm32::v128_and($a, $b)
            }
        });
    }
    pub use _and as and;

    #[macro_export]
    macro_rules! _and_not {
        (
            $just_anded: expr,
            $notted: expr $(,)?
        ) => ({#[allow(unused_unsafe)]
            unsafe {
                core::arch::wasm32::v128_andnot($just_anded, $notted)
            }
        });
    }
    pub use _and_not as and_not;

    #[macro_export]
    macro_rules! _or {
        (
            $a: expr,
            $b: expr $(,)?
        ) => ({#[allow(unused_unsafe)]
            unsafe {
                core::arch::wasm32::v128_or($a, $b)
            }
        });
    }
    pub use _or as or;

    #[macro_export]
    macro_rules! _left_shift_32 {
        (
            $a: expr,
            $imm8: expr $(,)?
        ) => ({#[allow(unused_unsafe)]
            unsafe {
                core::arch::wasm32::i32x4_shl($a, $imm8)
            }
        });
    }
    pub use _left_shift_32 as left_shift_32;

    #[macro_export]
    macro_rules! _right_shift_32 {
        (
            $a: expr,
            $imm8: expr $(,)?
        ) => ({#[allow(unused_unsafe)]
            unsafe {
                core::arch::wasm32::u32x4_shr($a, $imm8)
            }
        });
    }
    pub use _right_shift_32 as right_shift_32;

    #[macro_export]
    macro_rules! _f32_to_u32 {
        (
            $a: expr $(,)?
        ) => ({#[allow(unused_unsafe)]
            unsafe {
                core::arch::wasm32::u32x4_trunc_sat_f32x4($a)
            }
        });
    }
    pub use _f32_to_u32 as f32_to_u32;

    #[macro_export]
    macro_rules! _u32_to_f32 {
        (
            $a: expr $(,)?
        ) => ({#[allow(unused_unsafe)]
            unsafe {
                core::arch::wasm32::f32x4_convert_u32x4($a)
            }
        });
    }
    pub use _u32_to_f32 as u32_to_f32;

    #[macro_export]
    macro_rules! _i32 {
        (
            $a: expr $(,)?
        ) => ({#[allow(unused_unsafe)]
            unsafe {
                core::arch::wasm32::i32x4_splat($a)
            }
        });
    }
    pub use _i32 as i32;

    #[macro_export]
    macro_rules! _f32 {
        (
            $a: expr $(,)?
        ) => ({#[allow(unused_unsafe)]
            unsafe {
                core::arch::wasm32::f32x4_splat($a)
            }
        });
    }
    pub use _f32 as f32;

    #[macro_export]
    macro_rules! _add_f32 {
        (
            $a: expr,
            $b: expr $(,)?
        ) => ({#[allow(unused_unsafe)]
            unsafe {
                core::arch::wasm32::f32x4_add($a, $b)
            }
        });
    }
    pub use _add_f32 as add_f32;

    #[macro_export]
    macro_rules! _add_i32 {
        (
            $a: expr,
            $b: expr $(,)?
        ) => ({#[allow(unused_unsafe)]
            unsafe {
                core::arch::wasm32::i32x4_add($a, $b)
            }
        });
    }
    pub use _add_i32 as add_i32;

    #[macro_export]
    macro_rules! _sub {
        (
            $a: expr,
            $b: expr $(,)?
        ) => ({#[allow(unused_unsafe)]
            unsafe {
                core::arch::wasm32::f32x4_sub($a, $b)
            }
        });
    }
    pub use _sub as sub;

    #[macro_export]
    macro_rules! _mul {
        (
            $a: expr,
            $b: expr $(,)?
        ) => ({#[allow(unused_unsafe)]
            unsafe {
                core::arch::wasm32::f32x4_mul($a, $b)
            }
        });
    }
    pub use _mul as mul;

    #[macro_export]
    macro_rules! _recip {
        (
            $a: expr $(,)?
        ) => ({#[allow(unused_unsafe)]
            unsafe {
                // As of this writing, there's no equivalent wasm instrinsic to
                // `_mm_rcp_ps`, so it seems we need to eat the extra divide
                // latency. :(
                core::arch::wasm32::f32x4_div(
                    core::arch::wasm32::f32x4_splat(1.),
                    $a
                )
            }
        });
    }
    pub use _recip as recip;

    #[macro_export]
    macro_rules! _sqrt {
        (
            $a: expr $(,)?
        ) => ({#[allow(unused_unsafe)]
            unsafe {
                core::arch::wasm32::f32x4_sqrt($a)
            }
        });
    }
    pub use _sqrt as sqrt;

    #[macro_export]
    macro_rules! _lt_mask_32 {
        (
            $lhs: expr,
            $rhs: expr $(,)?
        ) => ({#[allow(unused_unsafe)]
            unsafe {
                core::arch::wasm32::i32x4_lt($lhs, $rhs)
            }
        });
    }
    pub use _lt_mask_32 as lt_mask_32;

    #[macro_export]
    macro_rules! _eq_mask_u32 {
        (
            $lhs: expr,
            $rhs: expr $(,)?
        ) => ({#[allow(unused_unsafe)]
            unsafe {
                core::arch::wasm32::i32x4_eq($lhs, $rhs)
            }
        });
    }
    pub use _eq_mask_u32 as eq_mask_u32;

    #[macro_export]
    macro_rules! _gt_mask_32 {
        (
            $lhs: expr,
            $rhs: expr $(,)?
        ) => ({#[allow(unused_unsafe)]
            unsafe {
                core::arch::wasm32::i32x4_gt($lhs, $rhs)
            }
        });
    }
    pub use _gt_mask_32 as gt_mask_32;
}

#[cfg(not(any(
    all(
        target_arch = "x86_64",
        target_feature = "sse",
        target_feature = "sse2",
    ),
    all(
        target_arch = "wasm32",
        target_feature = "simd128",
    ),
)))]
mod wide {
    use super::*;
    pub const WIDTH: unscaled::Inner = 4;

    compile_error!("wide not implemented for this target");
}

#[cfg(test)]
mod wide_tests {
    use super::wide::*;

    /// Thses names chosen to be the same length as `0`.
    const A: i32 = 0xAAAA_AAAAu32 as i32;
    const B: i32 = 0xBBBB_BBBBu32 as i32;
    const C: i32 = 0xCCCC_CCCCu32 as i32;
    const D: i32 = 0xDDDD_DDDDu32 as i32;
    const F: i32 = 0xFFFF_FFFFu32 as i32;

    macro_rules! to_array {
        ($v: expr) => ({
            let mut arr = [0; WIDTH as usize];
            let v = $v;

            // SAFETY: The pointers produced by the code generated by
            // these macros are valid to read 128 bytes from.
            unsafe {
                store!(v, &mut arr);
            }

            arr
        })
    }

    #[test]
    fn pick_via_mask_returns_the_expected_result_in_this_or_like_case() {
        let left = i32x4!(A, B, 0, 0);
        let right = i32x4!(0, 0, C, D);
        let mask = i32x4!(0, 0, F, F);

        let actual = pick_via_mask!(
            left,
            right,
            mask,
        );

        assert_eq!(
            to_array!(actual),
            to_array!(i32x4!(A, B, C, D))
        );
    }
}

pub fn render(
    frame_buffer: &mut FrameBuffer,
    commands: &[Command],
) -> NeedsRedraw {
    if frame_buffer.width < command::WIDTH
    || frame_buffer.height < command::HEIGHT {
        frame_buffer.width = command::WIDTH;
        frame_buffer.height = command::HEIGHT;
    }

    let width_multiplier = frame_buffer.width / command::WIDTH;
    let height_multiplier = frame_buffer.height / command::HEIGHT;
    let multiplier = core::cmp::min(width_multiplier, height_multiplier);
    if multiplier == 0 {
        debug_assert!(multiplier != 0);
        return NeedsRedraw::No;
    }

    let vertical_bars_width: clip::W = frame_buffer.width - (multiplier * command::WIDTH);

    let left_bar_width: clip::W = (vertical_bars_width + 1) / 2;

    let right_bar_width: clip::W = vertical_bars_width / 2;

    let horizontal_bars_height: clip::H = frame_buffer.height - (multiplier * command::HEIGHT);

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


    // This started as cached software rendering based on:
    // https://rxi.github.io/cached_software_rendering.html
    // But, a single cell turned out to be the most performant,
    // in this particular case, so the code was simplifed with
    // that in mind. Maybe it faster to have a single cell, just
    // because the rendered size is so small. Not sure.

    frame_buffer.cells.reset_then_hash_commands(
        commands,
        (frame_buffer.width, frame_buffer.height),
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

    if cells == cells_prev {
        frame_buffer.cells.swap();
        return NeedsRedraw::No;
    }

    let wide_0 = wide::i32!(0);
    let wide_1_f32 = wide::f32!(1.);
    let wide_255_i32 = wide::i32!(255);
    let wide_inv_255_f32 = wide::f32!(1./255.);
    let wide_255_f32 = wide::f32!(255.);
    let wide_0_to_w = wide::i32x4!(0, 1, 2, 3);

    // Hopefully this compiles to something not inefficent
    for i in 0..frame_buffer.unscaled_buffer.len() {
        frame_buffer.unscaled_buffer[i] = colours::BLACK;
    }

    for &Command {
        sprite_xy: sprite::XY { x: sprite_x, y: sprite_y },
        colour_override,
        rect,
    } in commands.iter() {
        let colour_override_value = wide::i32!(colour_override as i32);

        let not_colour_override_mask = wide::eq_mask_u32!(
            colour_override_value,
            wide_0
        );

        let Rect {
            x_min,
            y_min,
            x_max,
            y_max,
        } = rect;

        // TODO make this wide too?
        let x_min = clip::X::from(x_min);
        let y_min = clip::Y::from(y_min);
        let x_max = clip::W::from(x_max);
        let y_max = clip::H::from(y_max);

        let x_end = x_max + 1;
        let y_end = y_max + 1;
        let wide_x_end = wide::i32!(x_end.into());

        let sprite_x = usize::from(sprite_x);
        let sprite_y = usize::from(sprite_y);

        let src_w = GFX_WIDTH as usize;

        let mut y_iter_count = 0;
        for y in y_min..y_end {
            let mut x_iter_count = 0;
            let mut x = x_min;

            while x < x_end {
                let wide_xs = wide::add_i32!(
                    wide::i32!(x.into()),
                    wide_0_to_w
                );

                let dest_index = usize::from(y)
                    * usize::from(command::WIDTH)
                    + usize::from(x);

                let unders = unsafe {
                    wide::load!(
                        frame_buffer.unscaled_buffer.as_ptr(),
                        dest_index,
                    )
                };

                let base_src_i =
                    (sprite_y + y_iter_count) * src_w
                    + (sprite_x + x_iter_count);

                let gfx_colours = unsafe {
                    wide::load!(
                        GFX.as_ptr(),
                        base_src_i
                    )
                };

                let is_full_alpha_mask = wide::eq_mask_u32!(
                    wide::right_shift_32!(
                        gfx_colours,
                        24
                    ),
                    wide_255_i32
                );

                let do_override_mask = wide::and_not!(
                    is_full_alpha_mask,
                    not_colour_override_mask
                );

                let gfx_colours = wide::pick_via_mask!(
                    gfx_colours,
                    colour_override_value,
                    do_override_mask,
                );

                let should_write =
                    wide::lt_mask_32!(
                        wide_xs,
                        wide_x_end
                    );

                // Don't need to mask the shifted in zeroes.
                let gfx_colour_a = wide::right_shift_32!(
                    gfx_colours,
                    24
                );

                let gfx_colour_r = wide::and!(
                    wide::right_shift_32!(
                        gfx_colours,
                        16
                    ),
                    wide_255_i32
                );

                let gfx_colour_g = wide::and!(
                    wide::right_shift_32!(
                        gfx_colours,
                        8
                    ),
                    wide_255_i32
                );

                // Don't need to shift since it's already in the right spot
                let gfx_colour_b = wide::and!(
                    gfx_colours,
                    wide_255_i32
                );

                // Don't need to mask the shifted in zeroes.
                let under_a = wide::right_shift_32!(
                    unders,
                    24
                );

                let under_r = wide::and!(
                    wide::right_shift_32!(
                        unders,
                        16
                    ),
                    wide_255_i32
                );

                let under_g = wide::and!(
                    wide::right_shift_32!(
                        unders,
                        8
                    ),
                    wide_255_i32
                );

                // Don't need to shift since it's already in the right spot
                let under_b = wide::and!(
                    unders,
                    wide_255_i32
                );

                // gamma to linear
                let mut a_g = wide::mul!(
                    wide::u32_to_f32!(
                        gfx_colour_a
                    ),
                    wide_inv_255_f32
                );
                a_g = wide::mul!(a_g, a_g);
                let mut r_g = wide::mul!(
                    wide::u32_to_f32!(
                        gfx_colour_r
                    ),
                    wide_inv_255_f32
                );
                r_g = wide::mul!(r_g, r_g);
                let mut g_g = wide::mul!(
                    wide::u32_to_f32!(
                        gfx_colour_g
                    ),
                    wide_inv_255_f32
                );
                g_g = wide::mul!(g_g, g_g);
                let mut b_g = wide::mul!(
                    wide::u32_to_f32!(
                        gfx_colour_b
                    ),
                    wide_inv_255_f32
                );
                b_g = wide::mul!(b_g, b_g);

                let mut a_u = wide::mul!(
                    wide::u32_to_f32!(
                        under_a
                    ),
                    wide_inv_255_f32
                );
                a_u = wide::mul!(a_u, a_u);
                let mut r_u = wide::mul!(
                    wide::u32_to_f32!(
                        under_r
                    ),
                    wide_inv_255_f32
                );
                r_u = wide::mul!(r_u, r_u);
                let mut g_u = wide::mul!(
                    wide::u32_to_f32!(
                        under_g
                    ),
                    wide_inv_255_f32
                );
                g_u = wide::mul!(g_u, g_u);
                let mut b_u = wide::mul!(
                    wide::u32_to_f32!(
                        under_b
                    ),
                    wide_inv_255_f32
                );
                b_u = wide::mul!(b_u, b_u);

                // perform alpha blending
                let o_a = wide::add_f32!(
                    a_g,
                    wide::mul!(
                        a_u,
                        wide::sub!(wide_1_f32, a_g)
                    )
                );

                let inv_o_a = wide::recip!(o_a);

                let one_minus_a_g = wide::sub!(wide_1_f32, a_g);

                let o_r = wide::mul!(
                    wide::add_f32!(
                        wide::mul!(r_g, a_g),
                        wide::mul!(
                            r_u,
                            one_minus_a_g
                        )
                    ),
                    inv_o_a
                );
                let o_g = wide::mul!(
                    wide::add_f32!(
                        wide::mul!(g_g, a_g),
                        wide::mul!(
                            g_u,
                            one_minus_a_g
                        )
                    ),
                    inv_o_a
                );
                let o_b = wide::mul!(
                    wide::add_f32!(
                        wide::mul!(b_g, a_g),
                        wide::mul!(
                            b_u,
                            one_minus_a_g
                        )
                    ),
                    inv_o_a
                );

                // linear to gamma
                let rendered_a = wide::f32_to_u32!(
                    wide::mul!(
                        wide_255_f32,
                        wide::sqrt!(o_a)
                    )
                );
                let rendered_r = wide::f32_to_u32!(
                    wide::mul!(
                        wide_255_f32,
                        wide::sqrt!(o_r)
                    )
                );
                let rendered_g = wide::f32_to_u32!(
                    wide::mul!(
                        wide_255_f32,
                        wide::sqrt!(o_g)
                    )
                );
                let rendered_b = wide::f32_to_u32!(
                    wide::mul!(
                        wide_255_f32,
                        wide::sqrt!(o_b)
                    )
                );

                let rendered = wide::or!(
                    wide::or!(
                        wide::left_shift_32!(rendered_a, 24),
                        wide::left_shift_32!(rendered_r, 16),
                    ),
                    wide::or!(
                        wide::left_shift_32!(rendered_g, 8),
                        rendered_b,
                    )
                );

                let to_store = wide::pick_via_mask!(
                    unders,
                    rendered,
                    should_write
                );

                // SAFETY: The pointers produced by the code generated by
                // this macro is valid to write 128 bytes to.
                unsafe {
                    wide::store!(
                        to_store,
                        frame_buffer.unscaled_buffer.as_mut_ptr(),
                        dest_index,
                    );
                }

                x_iter_count += wide::WIDTH as usize;
                x += wide::WIDTH;
            }

            y_iter_count += 1;
        }
    }

    // TODO If we need some more render speed, this could be made wide as well.
    let mut src_i = 0;
    let mut src_i_row_start;
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
            let src_w = usize::from(command::WIDTH);
            src_i += src_w;
        }
    }

    frame_buffer.cells.swap();

    NeedsRedraw::Yes
}
