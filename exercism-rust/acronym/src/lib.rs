#[derive(PartialEq, PartialOrd)]
enum ParserState {
    WordStart,
    UpperCase,
    LowerCase,
}

pub fn abbreviate(phrase: &str) -> String {
    phrase
        .chars()
        .fold(
            (ParserState::WordStart, "".to_owned()),
            |(previous, subresult), c| {
                assert!(subresult != "" || previous == ParserState::WordStart);
                if c.is_whitespace() || c == '-' || c == '_' {
                    return (ParserState::WordStart, subresult);
                }

                match (previous, c.is_uppercase()) {
                    (ParserState::WordStart, _) => (ParserState::UpperCase, format!("{subresult}{c}", c = c.to_uppercase())),
                    (ParserState::UpperCase, true) => (ParserState::UpperCase, subresult),
                    (ParserState::UpperCase, false) => (ParserState::LowerCase, subresult),
                    
                    (ParserState::LowerCase, false) => (ParserState::LowerCase, subresult),
                    (ParserState::LowerCase, true) => (ParserState::UpperCase, format!("{subresult}{c}")),
                }
            },
        )
        .1
        .to_owned()
}