use std::fmt;

use super::{error::LexError, Position};

/// Set of all keyword tokens
#[derive(Clone, Debug, PartialEq)]
pub enum Keyword {
    As,
}

impl fmt::Display for Keyword {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Keyword::*;

        Ok(match self{
            As => write!(f, "as")?,
        })
    }
}

/// Set of allowed number tokens
#[derive(Clone, Debug, PartialEq)]
pub enum Number {
    Int(String),
    Float(String),
}

impl fmt::Display for Number {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Number::*;

        Ok(match self{
            Int(n) => write!(f, "{}", n)?,
            Float(n) => write!(f, "{}", n)?,
        })
    }
}

/// Set of possible stand-alone symbol tokens
#[derive(Clone, Debug, PartialEq)]
pub enum Symbol {
    AtSign,
    Period,
    Underscore,
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Symbol::*;

        Ok(match self{
            AtSign => write!(f, "@")?,
            Period => write!(f, ".")?,
            Underscore => write!(f, "_")?,
        })
    }
}

/// Set of possible whitespace tokens
#[derive(Clone, Debug, PartialEq)]
pub enum Whitespace {
    Inline(String),
    Newline,
}

impl fmt::Display for Whitespace {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Whitespace::*;

        Ok(match self{
            Inline(i) => write!(f, "whitespace `{}`", i)?,
            Newline => write!(f, "newline")?,
        })
    }
}

/// Set of all possible tokens
#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    Boolean(bool),
    Identifier(String),
    Keyword(Keyword),
    Number(Number),
    QuotedIdentifier(String),
    Symbol(Symbol),
    Text(String),
    Whitespace(Whitespace),
}

/// A token with its position
#[derive(Clone, Debug, PartialEq)]
pub struct TokenPosition {
    token: Token,
    start_position: Position,
    end_position: Position,
}

#[derive(Clone, Debug, PartialEq)]
pub enum State {
    Start,
    Float,
    Identifier,
    Integer,
    QuotedIdentifier,
    Period,
    Text,
    Underscore,
    Whitespace,
}

#[derive(Debug)]
pub(super) struct Tokenizer {
    state: State,
    stack: Vec<char>,
    start_position: Position,
    end_position: Position,
    pub tokens: Vec<TokenPosition>,
}

impl Tokenizer {
    pub fn new() -> Self {
        Self {
            start_position: Position { line: 1, column: 1 },
            end_position: Position { line: 1, column: 1 },
            state: State::Start,
            stack: Vec::new(),
            tokens: Vec::new(),
        }
    }

    fn add_token(&mut self, token: Token) {
        self.tokens.push(TokenPosition {
            token,
            start_position: self.start_position,
            end_position: self.end_position,
        })
    }

    fn receive(&mut self, c: char) -> Result<State, LexError> {
        Ok(match self.state {
            State::Start => match c {
                '\0' => {
                    State::Start
                }
                '@' => {
                    self.add_token(Token::Symbol(Symbol::AtSign));
                    self.reset_position();
                    State::Start
                }
                '_' => {
                    self.stack.push(c);
                    State::Underscore
                }
                '\'' => {
                    State::Text
                }
                '"' => {
                    State::QuotedIdentifier
                }
                '.' => {
                    State::Period
                }
                '0'..='9' => {
                    self.stack.push(c);
                    State::Integer
                }
                c if is_newline(c) => {
                    self.add_token(Token::Whitespace(Whitespace::Newline));

                    self.start_position.line += 1;
                    self.start_position.column = 1;
                    self.end_position = self.start_position;

                    State::Start
                }
                c if is_inline_whitespace(c) => {
                    self.stack.push(c);
                    State::Whitespace
                }
                c if is_valid_identifier(c) => {
                    self.stack.push(c);
                    State::Identifier
                }
                _ => return Err(self.unexpected(c))
            }
            State::Float => match c {
                '0'..='9' => {
                    self.stack.push(c);
                    self.end_position.column += 1;
                    State::Float
                }
                '.' => return Err(self.unexpected(c)),
                _ => {
                    let stack = self.drain_stack();
                    let token = Token::Number(Number::Float(stack));

                    self.add_token(token);
                    self.reset_with(c)?
                }
            }
            State::Identifier => match c {
                c if is_valid_identifier(c) => {
                    self.stack.push(c);
                    self.end_position.column += 1;
                    State::Identifier
                }
                _ => {
                    let stack = self.drain_stack();
                    let token = identifier_to_token(stack);

                    self.add_token(token);
                    self.reset_with(c)?
                }
            }
            State::Integer => match c {
                '0'..='9' => {
                    self.stack.push(c);
                    self.end_position.column += 1;
                    State::Integer
                }
                '.' => {
                    self.stack.push(c);
                    self.end_position.column += 1;
                    State::Float
                }
                _ => {
                    let stack = self.drain_stack();
                    let token = Token::Number(Number::Int(stack));

                    self.add_token(token);
                    self.reset_with(c)?
                }
            }
            State::QuotedIdentifier => match c {
                '"' => {
                    let stack = self.drain_stack();
                    let token = Token::QuotedIdentifier(stack);

                    self.add_token_and_advance(token)
                }
                _ => {
                    self.stack.push(c);
                    self.end_position.column += 1;
                    State::QuotedIdentifier
                }
            }
            State::Period => match c {
                '0'..='9' => {
                    self.stack.push('.');
                    self.stack.push(c);
                    self.end_position.column += 1;
                    State::Float
                }
                _ => {
                    self.add_token(Token::Symbol(Symbol::Period));
                    self.reset_with(c)?
                }
            }
            State::Text => match c {
                '\'' => {
                    let stack = self.drain_stack();
                    let token = Token::Text(stack);

                    self.add_token_and_advance(token)
                }
                _ => {
                    self.stack.push(c);
                    self.end_position.column += 1;
                    State::Text
                }
            }
            State::Underscore => match c {
                c if is_valid_identifier(c) => {
                    self.stack.push(c);
                    self.end_position.column += 1;
                    State::Identifier
                }
                _ => {
                    self.add_token(Token::Symbol(Symbol::Underscore));
                    self.reset_with(c)?
                }
            }
            State::Whitespace => match c {
                ' ' | '\t' => {
                    self.stack.push(c);
                    self.end_position.column += 1;
                    State::Whitespace
                }
                _ => {
                    let stack = self.drain_stack();
                    let token = Token::Whitespace(Whitespace::Inline(stack));

                    self.add_token(token);
                    self.reset_with(c)?
                }
            }
        })
    }

    fn unexpected(&self, c: char) -> LexError {
        let mut position = self.end_position;

        position.column += 1;
        LexError::unexpected_character(position, c)
    }

    /// Useful for adding a token when a character is encountered that needs
    /// to be skipped, such as closing single- or double- quotes
    fn add_token_and_advance(&mut self, token: Token) -> State {
        self.end_position.column += 1;
        self.add_token(token);
        self.reset_position();
        State::Start
    }

    fn reset_position(&mut self) {
        self.end_position.column += 1;
        self.start_position = self.end_position;
    }

    fn reset_with(&mut self, c: char) -> Result<State, LexError> {
        self.reset_position();
        self.state = State::Start;

        self.receive(c)
    }

    fn drain_stack(&mut self) -> String {
        self.stack.drain(..).collect()
    }

    pub fn tokenize(mut self, input: &str) -> Result<Self, LexError> {
        for c in input.chars() {
            self.state = self.receive(c)?;
        }

        // An 'escape hatch' to make sure the last state/stack are processed
        // if not ending back at the 'start' state
        self.receive('\0')?;

        Ok(self)
    }
}

fn is_newline(c: char) -> bool {
    c == '\n'
}

fn is_inline_whitespace(c: char) -> bool {
    c == ' ' || c == '\t'
}

fn is_valid_identifier(c: char) -> bool {
    c == '_' || (
        // "alphabetic" isn't enough because that precludes other unicode chars
        // that are valid in postgres identifiers
        !c.is_control() && !c.is_whitespace() && !c.is_ascii_punctuation()
    )
}

fn identifier_to_token(s: String) -> Token {
    match s.as_ref() {
        "_" => Token::Symbol(Symbol::Underscore),
        "true" | "t" => Token::Boolean(true),
        "false" | "f" => Token::Boolean(false),
        "as" => Token::Keyword(Keyword::As),
        _ => Token::Identifier(s),
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::super::error::LexErrorKind;
    use super::*;

    mod helpers {
        use super::super::*;

        pub fn at_sign() -> Token {
            Token::Symbol(Symbol::AtSign)
        }

        pub fn boolean(b: bool) -> Token {
            Token::Boolean(b)
        }

        pub fn float(n: &str) -> Token {
            Token::Number(Number::Float(n.to_owned()))
        }

        pub fn identifier(i: &str) -> Token {
            Token::Identifier(i.to_owned())
        }

        pub fn int(n: &str) -> Token {
            Token::Number(Number::Int(n.to_owned()))
        }

        pub fn newline() -> Token {
            Token::Whitespace(Whitespace::Newline)
        }

        pub fn period() -> Token {
            Token::Symbol(Symbol::Period)
        }

        pub fn quoted(i: &str) -> Token {
            Token::QuotedIdentifier(i.to_owned())
        }

        pub fn space() -> Token {
            spaces(1)
        }

        pub fn spaces(n: usize) -> Token {
            Token::Whitespace(Whitespace::Inline(" ".repeat(n)))
        }

        pub fn text(t: &str) -> Token {
            Token::Text(t.to_owned())
        }

        pub fn underscore() -> Token {
            Token::Symbol(Symbol::Underscore)
        }

        pub fn whitespace(ws: &str) -> Token {
            Token::Whitespace(Whitespace::Inline(ws.to_owned()))
        }

        pub fn tokenize(input: &str) -> Result<Vec<TokenPosition>, LexError> {
            Ok(Tokenizer::new().tokenize(input)?.tokens)
        }

        #[test]
        fn tests() {
            assert_eq!(at_sign(), Token::Symbol(Symbol::AtSign));

            assert_eq!(boolean(true), Token::Boolean(true));
            assert_eq!(boolean(false), Token::Boolean(false));

            assert_eq!(float("0.12"), Token::Number(Number::Float("0.12".to_owned())));
            assert_eq!(float("1.23"), Token::Number(Number::Float("1.23".to_owned())));

            assert_eq!(identifier("wat"), Token::Identifier("wat".to_owned()));
            assert_eq!(identifier("hey"), Token::Identifier("hey".to_owned()));

            assert_eq!(int("12"), Token::Number(Number::Int("12".to_owned())));
            assert_eq!(int("123"), Token::Number(Number::Int("123".to_owned())));

            assert_eq!(newline(), Token::Whitespace(Whitespace::Newline));

            assert_eq!(period(), Token::Symbol(Symbol::Period));

            assert_eq!(quoted("wat"), Token::QuotedIdentifier("wat".to_owned()));
            assert_eq!(quoted("hey"), Token::QuotedIdentifier("hey".to_owned()));

            assert_eq!(space(), Token::Whitespace(Whitespace::Inline(" ".to_owned())));
            assert_eq!(spaces(3), Token::Whitespace(Whitespace::Inline("   ".to_owned())));
            assert_eq!(spaces(7), Token::Whitespace(Whitespace::Inline("       ".to_owned())));

            assert_eq!(text("wat"), Token::Text("wat".to_owned()));
            assert_eq!(text("hey"), Token::Text("hey".to_owned()));

            assert_eq!(underscore(), Token::Symbol(Symbol::Underscore));

            assert_eq!(whitespace("   "), Token::Whitespace(Whitespace::Inline("   ".to_owned())));
            assert_eq!(whitespace(" \t \t "), Token::Whitespace(Whitespace::Inline(" \t \t ".to_owned())));
        }
    }

    use helpers::*;

    fn tp(start: (usize, usize), end: (usize, usize), token: Token) -> TokenPosition {
        TokenPosition {
            start_position: Position {
                line: start.0,
                column: start.1,
            },
            end_position: Position {
                line: end.0,
                column: end.1,
            },
            token,
        }
    }

    #[test]
    fn bools() {
        let input = "true \t  false";

        assert_eq!(
            tokenize(input),
            Ok(vec![
                tp((1, 1), (1,  4), boolean(true)),
                tp((1, 5), (1,  8), whitespace(" \t  ")),
                tp((1, 9), (1, 13), boolean(false)),
            ])
        );
    }

    #[test]
    fn identifiers() {
        let input = "other \t  _things \n Here_that_are_Identifiers";

        assert_eq!(
            tokenize(input),
            Ok(vec![
                tp((1,  1), (1,  5), identifier("other")),
                tp((1,  6), (1,  9), whitespace(" \t  ")),
                tp((1, 10), (1, 16), identifier("_things")),
                tp((1, 17), (1, 17), space()),
                tp((1, 18), (1, 18), newline()),

                tp((2,  1), (2,  1), space()),
                tp((2,  2), (2, 26), identifier("Here_that_are_Identifiers")),
            ])
        );
    }

    #[test]
    fn keywords() {
        let input = "one_thing as another_thing";

        assert_eq!(
            tokenize(input),
            Ok(vec![
                tp((1,  1), (1,  9), identifier("one_thing")),
                tp((1, 10), (1, 10), space()),
                tp((1, 11), (1, 12), Token::Keyword(Keyword::As)),
                tp((1, 13), (1, 13), space()),
                tp((1, 14), (1, 26), identifier("another_thing")),
            ])
        )
    }

    #[test]
    fn numbers() {
        let input = "12 12. 12.34 .34";

        assert_eq!(
            tokenize(input),
            Ok(vec![
                tp((1,  1), (1,  2), int("12")),
                tp((1,  3), (1,  3), space()),
                tp((1,  4), (1,  6), float("12.")),
                tp((1,  7), (1,  7), space()),
                tp((1,  8), (1, 12), float("12.34")),
                tp((1, 13), (1, 13), space()),
                tp((1, 14), (1, 16), float(".34")),
            ])
        );
    }

    #[test]
    fn numbers_double_decimal_fails() {
        let input = "12.34.56";

        assert_eq!(
            tokenize(input),
            Err(LexError {
                kind: LexErrorKind::UnexpectedCharacter('.'),
                position: Position {
                    line: 1,
                    column: 6,
                },
            })
        )
    }

    #[test]
    fn quoted_identifiers() {
        let input = "some \"quoted identifier\" \n \"and here too\"";

        assert_eq!(
            tokenize(input),
            Ok(vec![
                tp((1,  1), (1,  4), identifier("some")),
                tp((1,  5), (1,  5), space()),
                tp((1,  6), (1, 24), quoted("quoted identifier")),
                tp((1, 25), (1, 25), space()),
                tp((1, 26), (1, 26), newline()),
                tp((2,  1), (2,  1), space()),
                tp((2,  2), (2, 15), quoted("and here too")),
            ])
        );
    }

    #[test]
    fn texts() {
        let input = "identifier 'some text' \"and a quoted identifier\"";

        assert_eq!(
            tokenize(input),
            Ok(vec![
                tp((1,  1), (1, 10), identifier("identifier")),
                tp((1, 11), (1, 11), space()),
                tp((1, 12), (1, 22), text("some text")),
                tp((1, 23), (1, 23), space()),
                tp((1, 24), (1, 48), quoted("and a quoted identifier")),
            ])
        );
    }

    #[test]
    fn symbols() {
        // This doesn't test single quotes or double quotes because they
        // are special cases that are part of other tokens
        let input = "@._";

        assert_eq!(
            tokenize(input),
            Ok(vec![
                tp((1, 1), (1, 1), at_sign()),
                tp((1, 2), (1, 2), period()),
                tp((1, 3), (1, 3), underscore()),
            ])
        );
    }

    // #[test]
    fn full_syntax() {
        let input =
"schema1
  person as p
    _
      name 'anon'

    p1
      name       'person 1'
      birthdate  '1900-01-01'
      likes_pizza true

  pet
    p1
      name     'pet 1'
      person_id p@p1.id
      species  'cat'

    _
      name     'pet 2'
      person_id p@p2.id
      species   @p1.species

  things
    _
      cost  123.456
      units 5

  \"quoted identifier table name\"";

        assert_eq!(
            tokenize(input),
            Ok(vec![
                tp((1,  1), (1,  7), identifier("schema1")),
                tp((1,  8), (1,  8), newline()),

                tp((2,  1), (2,  2), spaces(2)),
                tp((2,  3), (2,  8), identifier("person")),
                tp((2,  9), (2,  9), spaces(1)),
                tp((2, 10), (2, 11), Token::Keyword(Keyword::As)),
                tp((2, 12), (2, 12), space()),
                tp((2, 13), (2, 13), identifier("p")),
                tp((2, 14), (2, 14), newline()),

                tp((3,  1), (3,  4), spaces(4)),
                tp((3,  5), (3,  5), underscore()),
                tp((3,  6), (3,  6), newline()),

                tp((4,  1), (4,  4), spaces(6)),
                tp((4,  5), (4,  5), identifier("name")),
                tp((4,  1), (4,  4), space()),
                tp((4,  5), (4,  5), text("anon")),
                tp((4,  6), (4,  6), newline()),
            ])
        );
    }
}
