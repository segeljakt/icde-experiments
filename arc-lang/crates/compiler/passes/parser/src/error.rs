//! Module for converting errors emitted by `LALRPOP` into compiler diagnostics.

use diagnostics::Error;
use info::ByteIndex;
use info::Info;
use info::SourceId;
use lalrpop_util::ParseError;
use lexer::tokens::Token;

/// Converts an LALRPOP `ErrorRecovery` into a `Diagnostic`.
pub(crate) fn parser_error(error: ParseError<ByteIndex, Token, ()>, file: SourceId) -> Error {
    match error {
        // User errors (lexer errors) are handled by the lexer.
        ParseError::User { error: () } => unreachable!("Lexer should have caught this error"),
        // Error generated by the parser when it encounters additional, unexpected tokens.
        ParseError::ExtraToken { token: (l, t, r) } => Error::ParserExtraToken {
            found: format!("{t:?}"),
            info: Info::from_range(file, l..r),
        },
        // Error generated by the parser when it encounters a token (or EOF) it did not expect.
        ParseError::InvalidToken { .. } => unreachable!("Lexer should have caught this error"),
        // Error generated by the parser when it encounters an EOF it did not expect.
        ParseError::UnrecognizedEof { location, expected } => Error::ParserUnrecognizedEof {
            info: Info::from_range(file, location..location),
            expected,
        },
        // Error generated by the parser when it encounters a token it did not expect.
        ParseError::UnrecognizedToken {
            token: (l, t, r),
            expected,
        } => Error::ParserUnrecognizedToken {
            found: format!("{t}"),
            info: Info::from_range(file, l..r),
            expected,
        },
    }
}
