//! This module implements all of the [Token]s used in the JavaScript programing language.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!
//! [spec]: https://tc39.es/ecma262/#sec-tokens

use super::regex::RegExpFlags;

use crate::{
    builtins::BigInt,
    syntax::ast::{Keyword, Punctuator, Span},
    Interner, Sym,
};

use std::fmt::{self, Debug, Display, Formatter};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// This represents the smallest individual words, phrases, or characters that JavaScript can understand.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-tokens
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    /// The token kind, which contains the actual data of the token.
    kind: TokenKind,
    /// The token position in the original source code.
    span: Span,
}

impl Token {
    /// Create a new detailed token from the token data, line number and column number
    #[inline]
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }

    /// Gets the kind of the token.
    #[inline]
    pub fn kind(&self) -> &TokenKind {
        &self.kind
    }

    /// Gets the token span in the original source code.
    #[inline]
    pub fn span(&self) -> Span {
        self.span
    }

    /// Retrieves a structure ready for display.
    pub fn display<'d>(&self, interner: &'d Interner) -> TokenDisplay<'_, 'd> {
        TokenDisplay {
            token: &self,
            interner,
        }
    }
}

/// Structure to allow displaying of tokens.
#[derive(Debug)]
pub struct TokenDisplay<'k, 'i> {
    token: &'k Token,
    interner: &'i Interner,
}

impl<'k, 'i> Display for TokenDisplay<'k, 'i> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.token.display(self.interner), f)
    }
}

/// Represents the type differenct types of numeric literals.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, PartialEq, Debug)]
pub enum Numeric {
    /// A floating point number
    Rational(f64),

    /// An integer
    Integer(i32),

    // A BigInt
    BigInt(BigInt),
}

impl From<f64> for Numeric {
    #[inline]
    fn from(n: f64) -> Self {
        Self::Rational(n)
    }
}

impl From<i32> for Numeric {
    #[inline]
    fn from(n: i32) -> Self {
        Self::Integer(n)
    }
}

impl From<BigInt> for Numeric {
    #[inline]
    fn from(n: BigInt) -> Self {
        Self::BigInt(n)
    }
}

/// Represents the type of Token and the data it has inside.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, PartialEq, Debug)]
pub enum TokenKind {
    /// A boolean literal, which is either `true` or `false`.
    BooleanLiteral(bool),

    /// The end of the file.
    EOF,

    /// An identifier.
    Identifier(Sym),

    /// A keyword.
    ///
    /// see: [`Keyword`](../keyword/enum.Keyword.html)
    Keyword(Keyword),

    /// A `null` literal.
    NullLiteral,

    /// A numeric literal.
    NumericLiteral(Numeric),

    /// A piece of punctuation
    ///
    /// see: [`Punctuator`](../punc/enum.Punctuator.html)
    Punctuator(Punctuator),

    /// A string literal.
    StringLiteral(Sym),

    /// A string template literal.
    TemplateLiteral(Sym),

    /// A regular expression, consisting of body and flags.
    RegularExpressionLiteral(Sym, RegExpFlags),

    /// Indicates the end of a line (`\n`).
    LineTerminator,

    /// Indicates a comment, the content isn't stored.
    Comment,
}

impl From<bool> for TokenKind {
    fn from(oth: bool) -> Self {
        Self::BooleanLiteral(oth)
    }
}

impl From<Keyword> for TokenKind {
    fn from(kw: Keyword) -> Self {
        Self::Keyword(kw)
    }
}

impl From<Punctuator> for TokenKind {
    fn from(punc: Punctuator) -> Self {
        Self::Punctuator(punc)
    }
}

impl From<Numeric> for TokenKind {
    fn from(num: Numeric) -> Self {
        Self::NumericLiteral(num)
    }
}

impl TokenKind {
    /// Creates a `BooleanLiteral` token kind.
    pub fn boolean_literal(lit: bool) -> Self {
        Self::BooleanLiteral(lit)
    }

    /// Creates an `EOF` token kind.
    pub fn eof() -> Self {
        Self::EOF
    }

    /// Creates an `Identifier` token type.
    pub fn identifier<I>(ident: I, interner: &mut Interner) -> Self
    where
        I: AsRef<str>,
    {
        Self::Identifier(interner.get_or_intern(ident))
    }

    /// Creates a `Keyword` token kind.
    pub fn keyword(keyword: Keyword) -> Self {
        Self::Keyword(keyword)
    }

    /// Creates a `NumericLiteral` token kind.
    pub fn numeric_literal<L>(lit: L) -> Self
    where
        L: Into<Numeric>,
    {
        Self::NumericLiteral(lit.into())
    }

    /// Creates a `Punctuator` token type.
    pub fn punctuator(punc: Punctuator) -> Self {
        Self::Punctuator(punc)
    }

    /// Creates a `StringLiteral` token type.
    pub fn string_literal<S>(lit: S, interner: &mut Interner) -> Self
    where
        S: AsRef<str>,
    {
        Self::StringLiteral(interner.get_or_intern(lit))
    }

    /// Creates a `TemplateLiteral` token type.
    pub fn template_literal<S>(lit: S, interner: &mut Interner) -> Self
    where
        S: AsRef<str>,
    {
        Self::TemplateLiteral(interner.get_or_intern(lit))
    }

    /// Creates a `RegularExpressionLiteral` token kind.
    pub fn regular_expression_literal<B, R>(body: B, flags: R, interner: &mut Interner) -> Self
    where
        B: AsRef<str>,
        R: Into<RegExpFlags>,
    {
        Self::RegularExpressionLiteral(interner.get_or_intern(body), flags.into())
    }

    /// Creates a `LineTerminator` token kind.
    pub fn line_terminator() -> Self {
        Self::LineTerminator
    }

    /// Creates a 'Comment' token kind.
    pub fn comment() -> Self {
        Self::Comment
    }

    /// Creates a display object for this token kind.
    pub fn display<'d>(&self, interner: &'d Interner) -> TokenKindDisplay<'_, 'd> {
        TokenKindDisplay {
            kind: &self,
            interner,
        }
    }
}

/// Structure that allows displaying a token kind.
#[derive(Debug)]
pub struct TokenKindDisplay<'k, 'i> {
    kind: &'k TokenKind,
    interner: &'i Interner,
}

impl<'k, 'i> Display for TokenKindDisplay<'k, 'i> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match *self.kind {
            TokenKind::BooleanLiteral(ref val) => write!(f, "{}", val),
            TokenKind::EOF => write!(f, "end of file"),
            TokenKind::Identifier(ident) => write!(
                f,
                "{}",
                self.interner.resolve(ident).expect("string disappeared")
            ),
            TokenKind::Keyword(ref word) => write!(f, "{}", word),
            TokenKind::NullLiteral => write!(f, "null"),
            TokenKind::NumericLiteral(Numeric::Rational(num)) => write!(f, "{}", num),
            TokenKind::NumericLiteral(Numeric::Integer(num)) => write!(f, "{}", num),
            TokenKind::NumericLiteral(Numeric::BigInt(ref num)) => write!(f, "{}n", num),
            TokenKind::Punctuator(ref punc) => write!(f, "{}", punc),
            TokenKind::StringLiteral(lit) => write!(
                f,
                "{}",
                self.interner.resolve(lit).expect("string disappeared")
            ),
            TokenKind::TemplateLiteral(lit) => write!(
                f,
                "{}",
                self.interner.resolve(lit).expect("string disappeared")
            ),
            TokenKind::RegularExpressionLiteral(body, flags) => write!(
                f,
                "/{}/{}",
                self.interner.resolve(body).expect("string disappeared"),
                flags
            ),
            TokenKind::LineTerminator => write!(f, "line terminator"),
            TokenKind::Comment => write!(f, "comment"),
        }
    }
}
