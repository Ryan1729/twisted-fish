use models::{Card, get_rank, get_suit, get_zinger, ranks, suits, zingers};

use platform_types::{ARGB, Command, Kind, PaletteIndex, sprite, unscaled::{self, Rect}, CHAR_W, CHAR_H, CHAR_WIDTH, CHAR_HEIGHT, FONT_WIDTH};

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
        rect: unscaled::Rect,
    ) {
        self.commands.push(
            Command {
                kind: Kind::Gfx(sprite_xy, 0),
                rect,
            }
        );
    }

    pub fn print_char(
        &mut self,
        character: u8,
        x: unscaled::X,
        y: unscaled::Y,
        colour: ARGB
    ) {
        let char_xy = {
            const SPRITES_PER_ROW: u8 = FONT_WIDTH / CHAR_WIDTH;
            const FONT_OFFSET: sprite::H = unscaled::h_const_mul(card::IMAGE_H, models::RANK_COUNT as _);

            (
                sprite::X(Into::into(
                    (character % SPRITES_PER_ROW) * CHAR_WIDTH,
                )),
                sprite::Y(Into::into(
                    (character / SPRITES_PER_ROW) * CHAR_HEIGHT,
                )) + FONT_OFFSET,
            )
        };

        self.commands.push(
            Command {
                kind: Kind::Gfx(
                    char_xy,
                    colour
                ),
                rect: Rect {
                    x,
                    y,
                    w: CHAR_W,
                    h: CHAR_H,
                },
            }
        );
    }

    pub fn print_line(
        &mut self,
        bytes: &[u8],
        mut x: unscaled::X,
        y: unscaled::Y,
        colour: ARGB,
    ) {
        for &c in bytes.iter() {
            self.print_char(c, x, y, colour);
            x += CHAR_ADVANCE_W;
        }
    }

    pub fn draw_card(
        &mut self,
        card: Card,
        x: unscaled::X,
        y: unscaled::Y
    ) {
        let suit_opt = get_suit(card);
        let rank_opt = get_rank(card);
        let zinger_opt = get_zinger(card);

        self.sspr(
            (
                card::BACKING_SPRITE_X,
                card::BACKING_SPRITE_BASE_Y
                + card::HEIGHT
                * sprite::Inner::from(card / models::RANK_COUNT)
            ),
            Rect {
                x,
                y,
                w: card::WIDTH,
                h: card::HEIGHT,
            }
        );

        let image_x = match suit_opt {
            Some(suit) => card::IMAGE_BASE_X
                + unscaled::Inner::from(suit)
                * card::IMAGE_W,
            None => card::ZINGER_IMAGE_X,
        };

        let image_y = match rank_opt {
            Some(rank) => card::IMAGE_BASE_Y
                + unscaled::Inner::from(rank)
                * card::IMAGE_H,
            None => match zinger_opt {
                Some(zinger) => card::IMAGE_BASE_Y
                + unscaled::Inner::from(zinger)
                * card::IMAGE_H,
                None => {
                    debug_assert!(false, "No suit or zinger for card: {card}");
                    card::IMAGE_BASE_Y
                }
            },
        };

        self.sspr(
            (image_x, image_y),
            Rect {
                x: x + card::IMAGE_W_OFFSET,
                y: y + card::IMAGE_H_OFFSET,
                w: card::IMAGE_W,
                h: card::IMAGE_H,
            }
        );

        let (line1, line2) = match (suit_opt, rank_opt, zinger_opt) {
            (Some(suits::RED), Some(ranks::BARNACLE), None) => (
                b"Red       ",
                b"Barnacle  ",
            ),
            (Some(suits::GREEN), Some(ranks::BARNACLE), None) => (
                b"Green     ",
                b"Barnacle  ",
            ),
            (Some(suits::BLUE), Some(ranks::BARNACLE), None) => (
                b"Blue      ",
                b"Barnacle  ",
            ),
            (Some(suits::YELLOW), Some(ranks::BARNACLE), None) => (
                b"Yellow    ",
                b"Barnacle  ",
            ),
            (Some(suits::PURPLE), Some(ranks::BARNACLE), None) => (
                b"Purple    ",
                b"Barnacle  ",
            ),
            (Some(suits::RED), Some(ranks::CRAB), None) => (
                b"Red       ",
                b"Crab      ",
            ),
            (Some(suits::GREEN), Some(ranks::CRAB), None) => (
                b"Green     ",
                b"Crab      ",
            ),
            (Some(suits::BLUE), Some(ranks::CRAB), None) => (
                b"Blue      ",
                b"Crab      ",
            ),
            (Some(suits::YELLOW), Some(ranks::CRAB), None) => (
                b"Yellow    ",
                b"Crab      ",
            ),
            (Some(suits::PURPLE), Some(ranks::CRAB), None) => (
                b"Purple    ",
                b"Crab      ",
            ),
            (Some(suits::RED), Some(ranks::DOGFISH), None) => (
                b"Red       ",
                b"Dogfish   ",
            ),
            (Some(suits::GREEN), Some(ranks::DOGFISH), None) => (
                b"Green     ",
                b"Dogfish   ",
            ),
            (Some(suits::BLUE), Some(ranks::DOGFISH), None) => (
                b"Blue      ",
                b"Dogfish   ",
            ),
            (Some(suits::YELLOW), Some(ranks::DOGFISH), None) => (
                b"Yellow    ",
                b"Dogfish   ",
            ),
            (Some(suits::PURPLE), Some(ranks::DOGFISH), None) => (
                b"Purple    ",
                b"Dogfish   ",
            ),
            (Some(suits::RED), Some(ranks::EEL), None) => (
                b"Red       ",
                b"Eel       ",
            ),
            (Some(suits::GREEN), Some(ranks::EEL), None) => (
                b"Green     ",
                b"Eel       ",
            ),
            (Some(suits::BLUE), Some(ranks::EEL), None) => (
                b"Blue      ",
                b"Eel       ",
            ),
            (Some(suits::YELLOW), Some(ranks::EEL), None) => (
                b"Yellow    ",
                b"Eel       ",
            ),
            (Some(suits::PURPLE), Some(ranks::EEL), None) => (
                b"Purple    ",
                b"Eel       ",
            ),
            (Some(suits::RED), Some(ranks::FLYING_FISH), None) => (
                b"Red Flying",
                b"fish      ",
            ),
            (Some(suits::GREEN), Some(ranks::FLYING_FISH), None) => (
                b"Green Fly-",
                b"ing fish  ",
            ),
            (Some(suits::BLUE), Some(ranks::FLYING_FISH), None) => (
                b"Blue Fly- ",
                b"ing fish  ",
            ),
            (Some(suits::YELLOW), Some(ranks::FLYING_FISH), None) => (
                b"Yellow Fl-",
                b"ying fish ",
            ),
            (Some(suits::PURPLE), Some(ranks::FLYING_FISH), None) => (
                b"Purple Fl-",
                b"ying fish ",
            ),
            (Some(suits::RED), Some(ranks::HAMMERHEAD), None) => (
                b"Red       ",
                b"Hammerhead",
            ),
            (Some(suits::GREEN), Some(ranks::HAMMERHEAD), None) => (
                b"Green     ",
                b"Hammerhead",
            ),
            (Some(suits::BLUE), Some(ranks::HAMMERHEAD), None) => (
                b"Blue      ",
                b"Hammerhead",
            ),
            (Some(suits::YELLOW), Some(ranks::HAMMERHEAD), None) => (
                b"Yellow    ",
                b"Hammerhead",
            ),
            (Some(suits::PURPLE), Some(ranks::HAMMERHEAD), None) => (
                b"Purple    ",
                b"Hammerhead",
            ),
            (Some(suits::RED), Some(ranks::JELLYFISH), None) => (
                b"Red       ",
                b"Jellyfish ",
            ),
            (Some(suits::GREEN), Some(ranks::JELLYFISH), None) => (
                b"Green     ",
                b"Jellyfish ",
            ),
            (Some(suits::BLUE), Some(ranks::JELLYFISH), None) => (
                b"Blue      ",
                b"Jellyfish ",
            ),
            (Some(suits::YELLOW), Some(ranks::JELLYFISH), None) => (
                b"Yellow    ",
                b"Jellyfish ",
            ),
            (Some(suits::PURPLE), Some(ranks::JELLYFISH), None) => (
                b"Purple    ",
                b"Jellyfish ",
            ),
            (Some(suits::RED), Some(ranks::SHRIMP), None) => (
                b"Red       ",
                b"Shrimp    ",
            ),
            (Some(suits::GREEN), Some(ranks::SHRIMP), None) => (
                b"Green     ",
                b"Shrimp    ",
            ),
            (Some(suits::BLUE), Some(ranks::SHRIMP), None) => (
                b"Blue      ",
                b"Shrimp    ",
            ),
            (Some(suits::YELLOW), Some(ranks::SHRIMP), None) => (
                b"Yellow    ",
                b"Shrimp    ",
            ),
            (Some(suits::PURPLE), Some(ranks::SHRIMP), None) => (
                b"Purple    ",
                b"Shrimp    ",
            ),
            (Some(suits::RED), Some(ranks::BLOWFISH), None) => (
                b"Red       ",
                b"Blowfish  ",
            ),
            (Some(suits::GREEN), Some(ranks::BLOWFISH), None) => (
                b"Green     ",
                b"Blowfish  ",
            ),
            (Some(suits::BLUE), Some(ranks::BLOWFISH), None) => (
                b"Blue      ",
                b"Blowfish  ",
            ),
            (Some(suits::YELLOW), Some(ranks::BLOWFISH), None) => (
                b"Yellow    ",
                b"Blowfish  ",
            ),
            (Some(suits::PURPLE), Some(ranks::BLOWFISH), None) => (
                b"Purple    ",
                b"Blowfish  ",
            ),
            (Some(suits::RED), Some(ranks::CLOWNFISH), None) => (
                b"Red       ",
                b"Clownfish ",
            ),
            (Some(suits::GREEN), Some(ranks::CLOWNFISH), None) => (
                b"Green     ",
                b"Clownfish ",
            ),
            (Some(suits::BLUE), Some(ranks::CLOWNFISH), None) => (
                b"Blue      ",
                b"Clownfish ",
            ),
            (Some(suits::YELLOW), Some(ranks::CLOWNFISH), None) => (
                b"Yellow    ",
                b"Clownfish ",
            ),
            (Some(suits::PURPLE), Some(ranks::CLOWNFISH), None) => (
                b"Purple    ",
                b"Clownfish ",
            ),
            (Some(suits::RED), Some(ranks::STARFISH), None) => (
                b"Red       ",
                b"Starfish  ",
            ),
            (Some(suits::GREEN), Some(ranks::STARFISH), None) => (
                b"Green     ",
                b"Starfish  ",
            ),
            (Some(suits::BLUE), Some(ranks::STARFISH), None) => (
                b"Blue      ",
                b"Starfish  ",
            ),
            (Some(suits::YELLOW), Some(ranks::STARFISH), None) => (
                b"Yellow    ",
                b"Starfish  ",
            ),
            (Some(suits::PURPLE), Some(ranks::STARFISH), None) => (
                b"Purple    ",
                b"Starfish  ",
            ),
            (Some(suits::RED), Some(ranks::WHALE), None) => (
                b"Red       ",
                b"Whale     ",
            ),
            (Some(suits::GREEN), Some(ranks::WHALE), None) => (
                b"Green     ",
                b"Whale     ",
            ),
            (Some(suits::BLUE), Some(ranks::WHALE), None) => (
                b"Blue      ",
                b"Whale     ",
            ),
            (Some(suits::YELLOW), Some(ranks::WHALE), None) => (
                b"Yellow    ",
                b"Whale     ",
            ),
            (Some(suits::PURPLE), Some(ranks::WHALE), None) => (
                b"Purple    ",
                b"Whale     ",
            ),
            (Some(suits::RED), Some(ranks::CARD_SHARK), None) => (
                b"Red       ",
                b"Card Shark",
            ),
            (Some(suits::GREEN), Some(ranks::CARD_SHARK), None) => (
                b"Green     ",
                b"Card Shark",
            ),
            (Some(suits::BLUE), Some(ranks::CARD_SHARK), None) => (
                b"Blue      ",
                b"Card Shark",
            ),
            (Some(suits::YELLOW), Some(ranks::CARD_SHARK), None) => (
                b"Yellow    ",
                b"Card Shark",
            ),
            (Some(suits::PURPLE), Some(ranks::CARD_SHARK), None) => (
                b"Purple    ",
                b"Card Shark",
            ),
            (None, None, Some(zingers::DEAD_SCUBA_DIVER)) => (
                b"Dead Scuba",
                b"Diver     ",
            ),
            (None, None, Some(zingers::DIVINE_INTERVENTION)) => (
                b"Divine In-",
                b"tervention",
            ),
            (None, None, Some(zingers::GLASS_BOTTOM_BOAT)) => (
                b"Glass Bot-",
                b"tom boat  ",
            ),
            (None, None, Some(zingers::NO_FISHING)) => (
                b"No        ",
                b"Fishing   ",
            ),
            (None, None, Some(zingers::THE_GAME_WARDEN)) => (
                b"The Game  ",
                b"Warden    ",
            ),
            (None, None, Some(zingers::THE_LURE)) => (
                b"The       ",
                b"Lure      ",
            ),
            (None, None, Some(zingers::THE_NET)) => (
                b"The       ",
                b"Net       ",
            ),
            (None, None, Some(zingers::TWO_FISTED_FISHERMAN)) => (
                b"2-fisted  ",
                b"fisherman ",
            ),
            _ => {
                // TODO uncomment
                //debug_assert!(false, "No lines for card: {card}");
                (
                    b"line 1 ???",
                    b"line 2 ???",
                )
            }
        };

        self.print_line(
            line1,
            x + card::LINE_W_OFFSET,
            y + card::LINE_H_1_OFFSET,
            card::TEXT_COLOUR,
        );
        self.print_line(
            line2,
            x + card::LINE_W_OFFSET,
            y + card::LINE_H_2_OFFSET,
            card::TEXT_COLOUR,
        );
    }
}

pub mod card {
    use super::*;

    use unscaled::{W, H, Inner, w_const_mul, h_const_add};
    use sprite::{x_const_add_w};

    pub const WIDTH: W = W(74);
    pub const HEIGHT: H = H(105);

    pub const IMAGE_W: W = W(72);
    pub const IMAGE_H: H = H(72);

    pub const IMAGE_BASE_X: sprite::X = sprite::X(0);
    pub const IMAGE_BASE_Y: sprite::Y = sprite::Y(0);

    pub const ZINGER_IMAGE_X: sprite::X =
        x_const_add_w(
            IMAGE_BASE_X,
            w_const_mul(IMAGE_W, models::SUIT_COUNT as Inner)
        );

    pub const BACKING_SPRITE_X: sprite::X =
        x_const_add_w(
            ZINGER_IMAGE_X,
            IMAGE_W
        );
    pub const BACKING_SPRITE_BASE_Y: sprite::Y = sprite::Y(0);

    // TODO: refer to palette instead of hardcoding this.
    pub const TEXT_COLOUR: platform_types::ARGB = 0xFF222222;

    pub const IMAGE_W_OFFSET: W = W(1);
    pub const IMAGE_H_OFFSET: H = H(4);

    pub const LINE_W_OFFSET: W = CHAR_SPACING_W;
    pub const LINE_H_1_OFFSET: H = h_const_add(IMAGE_H_OFFSET, IMAGE_H);
    pub const LINE_H_2_OFFSET: H = h_const_add(LINE_H_1_OFFSET, CHAR_ADVANCE_H);
}

pub const TEN_CHAR: u8 = 27;

pub const CLUB_CHAR: u8 = 31;
pub const DIAMOND_CHAR: u8 = 29;
pub const HEART_CHAR: u8 = 30;
pub const SPADE_CHAR: u8 = 28;

pub const CHAR_SPACING: u8 = 2;
pub const CHAR_SPACING_W: unscaled::W = unscaled::W(CHAR_SPACING as _);
pub const CHAR_SPACING_H: unscaled::H = unscaled::H(CHAR_SPACING as _);

const CHAR_ADVANCE_WIDTH: unscaled::Inner =
    CHAR_WIDTH as unscaled::Inner
    + CHAR_SPACING as unscaled::Inner;

pub const CHAR_ADVANCE_W: unscaled::W = unscaled::W(CHAR_ADVANCE_WIDTH);

const CHAR_ADVANCE_HEIGHT: unscaled::Inner =
    CHAR_HEIGHT as unscaled::Inner
    + CHAR_SPACING as unscaled::Inner;

pub const CHAR_ADVANCE_H: unscaled::H = unscaled::H(CHAR_ADVANCE_HEIGHT);

// TODO `CharCount` type?
pub const WIDTH_IN_CHARS: unscaled::Inner =
    unscaled::WIDTH
    / CHAR_ADVANCE_WIDTH;
