fn main() {
    let json = include_str!("../monogram-bitmap.json");

    #[derive(Clone, Copy, Debug)]
    enum State {
        OpenQuote,
        Char,
        CloseQuote(u8),
        SkippingCloseQuote,
        SkippingOpenQuote,
        OpenBracket(u8),
        // We skip the first three rows because we only care about English right now.
        FirstComma(u8),
        SecondComma(u8),
        ThirdComma(u8),
        FourthComma(u8),
    }
    use State::*;

    //const CHAR_W: u8 = 5;
    //const CHAR_ASC: u8 = 2;
    //const CHAR_DESC: u8 = 2;
    //const CHAR_H: u8 = CHAR_ASC + 5 + CHAR_DESC;

    //let mut output = [0u8; CHAR_W as usize * 16 * CHAR_H as usize * 16];
    let mut state = State::OpenQuote;

    for json_char in json.chars() {
        macro_rules! bail {
            () => {
                panic!("Got '{json_char}' in state {state:?}")
            }
        }

        state = match (state, json_char) {
            (OpenQuote, '"') => Char,
            (Char, c) if c.is_ascii() => CloseQuote(c as u8),
            (Char, _) => SkippingCloseQuote,
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
            (FirstComma(c), ',') => SecondComma(c),
            (SecondComma(c), ',') => ThirdComma(c),
            (ThirdComma(c), ',') => FourthComma(c),
            (FourthComma(c), ch) if ch.is_ascii_digit() => {
                todo!()
            },
            (FourthComma(c), ',') => {
                todo!()
            },
            (FourthComma(c), _) => {
                todo!()
            },
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
                | OpenBracket(_),
                _
            ) => bail!(),
        };
    }

    todo!()
}
