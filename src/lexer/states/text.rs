use crate::lexer::error::LexErrorKind;
use crate::lexer::tokens::TokenKind;
use super::prelude::*;
use super::start::Start;

/// State after receiving a single quote and inside a string literal.
#[derive(Debug)]
pub(super) struct InText(pub Stack);

impl State for InText {
    fn receive(self: Box<Self>, c: Option<char>) -> ReceiveResult {
        use Action::{ContinueToken, NoAction};
        use LexErrorKind::UnclosedString;
        use TransitionErrorPosition::CurrentPosition;

        let mut stack = self.0;

        match c {
            Some('\'') => {
                to(AfterText(stack), NoAction) // TODO: Action
            }
            Some(c) => {
                stack.push(c);
                to(InText(stack), ContinueToken)
            }
            None => Err(TransitionError {
                kind: UnclosedString,
                position: CurrentPosition,
            }),
        }
    }
}

/// State after receiving what might be a closing quote unless the next
/// character received is another single quote, which indicates the previous
/// quote was being escaped and is part of the text string.
#[derive(Debug)]
pub(super) struct AfterText(pub Stack);

impl State for AfterText {
    fn receive(self: Box<Self>, c: Option<char>) -> ReceiveResult {
        use Action::{AddToken, ContinueToken};

        let mut stack = self.0;
        stack.push('\'');

        match c {
            Some('\'') => {
                stack.push('\'');
                to(InText(stack), ContinueToken)
            }
            _ => {
                let kind = TokenKind::Text(stack.consume());
                defer_to(Start, c, AddToken(kind))
            }
        }
    }
}