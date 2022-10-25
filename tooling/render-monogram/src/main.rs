fn main() {
    let json = include_str!("../monogram-bitmap.json");

    const CHAR_W: u8 = 5;
    #[cfg(not(feature = "some_accents"))]
    const CHAR_ASC: u8 = 2;
    #[cfg(feature = "some_accents")]
    const CHAR_ASC: u8 = 3;
    const CHAR_DESC: u8 = 2;
    const CHAR_H: u8 = CHAR_ASC + 5 + CHAR_DESC;
    const MAX_ROW_DIGITS: u8 = 2;

    type Chunks = [[u8; MAX_ROW_DIGITS as usize]; CHAR_H as usize];

    #[derive(Clone, Copy, Debug)]
    enum State {
        OpenQuote,
        Char,
        BackslashChar,
        CloseQuote(u8),
        SkippingCloseQuote,
        SkippingOpenQuote,
        OpenBracket(u8),
        // We skip the first n rows because we only care about certain characters 
        // right now.
        FirstComma(u8),
        #[allow(unused)]
        SecondComma(u8),
        ThirdComma(u8),
        RemainingDigits(u8, Chunks, u8),
        RemainingDigitsComma(u8, Chunks, u8),
        CommaOrBrace,
    }
    use State::*;

    const OUTPUT_W_IN_CHARS: usize = 16;
    const OUTPUT_H_IN_CHARS: usize = 16;

    const OUTPUT_W: usize = CHAR_W as usize * OUTPUT_W_IN_CHARS;
    const OUTPUT_H: usize = CHAR_H as usize * OUTPUT_H_IN_CHARS;

    #[cfg(not(feature = "some_accents"))]
    fn is_desired(c: char) -> bool {
        c.is_ascii()
    }

    #[cfg(feature = "some_accents")]
    fn is_desired(c: char) -> bool {
        c.is_ascii() || c == 'é' || c == 'í'
    }

    let mut output = [0u8; OUTPUT_W * OUTPUT_H];
    let mut state = State::OpenQuote;

    for json_char in json.chars() {
        state = match (state, json_char) {
            (OpenQuote, '"') => Char,
            (Char, '\\') => BackslashChar,
            (Char | BackslashChar, c) if is_desired(c) => CloseQuote(c as u8),
            (Char | BackslashChar, _) => SkippingCloseQuote,
            (SkippingCloseQuote, '"') => SkippingOpenQuote,
            (SkippingOpenQuote, '"') => Char,
            (CloseQuote(c), '"') => OpenBracket(c),
            (OpenBracket(c), '[') => FirstComma(c),
            (
                FirstComma(_)
                | SecondComma(_)
                | ThirdComma(_),
                ch
            ) if ch.is_ascii_digit() => state,
            #[cfg(feature = "some_accents")]
            (FirstComma(c), ',') => ThirdComma(c),
            #[cfg(not(feature = "some_accents"))]
            (FirstComma(c), ',') => SecondComma(c),
            (SecondComma(c), ',') => ThirdComma(c),
            (ThirdComma(c), ',') => RemainingDigits(c, <_>::default(), 0),
            (RemainingDigits(c, mut chunks, mut overall_i), ch) if ch.is_ascii_digit() => {
                let which_chunk = overall_i >> 1;
                let which_in_chunk = overall_i & 1;
                chunks[which_chunk as usize][which_in_chunk as usize] = ch as u8;

                if which_in_chunk == 0 {
                    overall_i += 1;

                    if overall_i as usize > chunks.len() * MAX_ROW_DIGITS as usize {
                        panic!("too many array elements for {c}");
                    }
                    RemainingDigits(c, chunks, overall_i)
                } else {
                    RemainingDigitsComma(c, chunks, overall_i)
                }
            },
            (
                RemainingDigits(c, chunks, mut overall_i)
                | RemainingDigitsComma(c, chunks, mut overall_i),
                ','
            ) => {
                overall_i += 1;
                if overall_i as usize > chunks.len() * MAX_ROW_DIGITS as usize {
                    panic!("too many array elements for {c}");
                }
                RemainingDigits(c, chunks, overall_i)
            },
            (
                RemainingDigits(c, chunks, _)
                | RemainingDigitsComma(c, chunks, _),
                ']'
            ) => {
                let (c_x, c_y) = (
                    c as usize % OUTPUT_W_IN_CHARS,
                    c as usize / OUTPUT_W_IN_CHARS
                );

                let base_i =
                    OUTPUT_W * c_y * CHAR_H as usize
                    + CHAR_W as usize * c_x
                ;

                let mut row = 0;
                for chunk in chunks {
                    use std::str::FromStr;

                    let slice: &[u8];
                    if chunk[1] == 0 {
                        slice = &chunk[0..1];
                    } else {
                        slice = &chunk;
                    };
                    // We should have already checked the expect cases.
                    let mut bits = u8::from_str(
                        std::str::from_utf8(slice)
                            .expect("chunk elements should be UTF-8")
                    ).expect("chunk shoud be a valid u8");

                    for col in 0..CHAR_W as usize {
                        let i = base_i + row * OUTPUT_W as usize + col;
                        #[cfg(not(feature = "png_output"))]
                        {
                            output[i] = bits & 1;
                        }

                        #[cfg(feature = "png_output")]
                        {
                            output[i] = if bits & 1 == 1 { 255 } else { 0 };
                        }
                        bits >>= 1;
                    }

                    row += 1;
                }
                CommaOrBrace
            },
            (CommaOrBrace, ','|' '|'\r'|'\n'|'}') => OpenQuote,
            (OpenQuote, '{'|'}'|' '|'\r'|'\n')
            | (SkippingOpenQuote, _)
            | (OpenBracket(_), ':'|' '|'\r'|'\n')
            => state,
            (
                OpenQuote
                | FirstComma(_)
                | SecondComma(_)
                | ThirdComma(_)
                | SkippingCloseQuote
                | CloseQuote(_)
                | OpenBracket(_)
                | RemainingDigits(..)
                | RemainingDigitsComma(..)
                | CommaOrBrace,
                _
            ) => panic!("Got '{json_char}' in state {state:?}"),
        };
    }

    #[cfg(not(feature = "png_output"))]
    {
        println!("[");
        for y in 0..OUTPUT_H {
            for x in 0..OUTPUT_W {
                let i = y * OUTPUT_W + x;
                print!("{},", output[i]);
            }
            println!();
        }
        println!("]");
    }

    #[cfg(feature = "png_output")]
    {
        image::save_buffer_with_format(
            "output.png",
            &output,
            OUTPUT_W.try_into().unwrap(),
            OUTPUT_H.try_into().unwrap(),
            image::ColorType::L8,
            image::ImageFormat::Png
        ).unwrap();
    }
}
