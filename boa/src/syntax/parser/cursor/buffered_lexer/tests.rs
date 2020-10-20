use super::BufferedLexer;
use crate::{
    syntax::lexer::{Lexer, Token, TokenKind},
    Interner,
};

#[test]
fn peek_skip_accending() {
    let mut interner = Interner::new();
    let mut cur = BufferedLexer::from(Lexer::new(&b"a b c d e f g h i"[..], &interner));

    assert_eq!(
        *cur.peek(0, false)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier("a", &mut interner)
    );
    assert_eq!(
        *cur.peek(1, false)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier("b", &mut interner)
    );
    assert_eq!(
        *cur.peek(2, false)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier("c", &mut interner)
    );
    assert_eq!(
        *cur.peek(2, false)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier("c", &mut interner)
    );
    assert_eq!(
        *cur.peek(1, false)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier("b", &mut interner)
    );
    assert_eq!(
        *cur.peek(0, false)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier("a", &mut interner)
    );
}

#[test]
fn peek_skip_next() {
    let mut interner = Interner::new();
    let mut cur = BufferedLexer::from(Lexer::new(&b"a b c d e f g h i"[..], &interner));

    assert_eq!(
        *cur.peek(0, false)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier("a", &mut interner)
    );
    assert_eq!(
        *cur.peek(1, false)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier("b", &mut interner)
    );
    assert_eq!(
        *cur.peek(2, false)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier("c", &mut interner)
    );
    assert_eq!(
        *cur.next(false)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier("a", &mut interner)
    );
    assert_eq!(
        *cur.next(false)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier("b", &mut interner)
    );
    assert_eq!(
        *cur.next(false)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier("c", &mut interner)
    );
    assert_eq!(
        *cur.next(false)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier("d", &mut interner)
    );
    assert_eq!(
        *cur.next(false)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier("e", &mut interner)
    );
    assert_eq!(
        *cur.peek(0, false)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier("f", &mut interner)
    );
    assert_eq!(
        *cur.peek(1, false)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier("g", &mut interner)
    );
    assert_eq!(
        *cur.peek(2, false)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier("h", &mut interner)
    );
}

#[test]
fn peek_skip_next_alternating() {
    let mut interner = Interner::new();
    let mut cur = BufferedLexer::from(Lexer::new(&b"a b c d e f g h i"[..], &interner));

    assert_eq!(
        *cur.peek(0, false)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier("a", &mut interner)
    );
    assert_eq!(
        *cur.next(false)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier("a", &mut interner)
    );
    assert_eq!(
        *cur.peek(1, false)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier("c", &mut interner)
    );
    assert_eq!(
        *cur.next(false)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier("b", &mut interner)
    );
    assert_eq!(
        *cur.peek(1, false)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier("d", &mut interner)
    );
    assert_eq!(
        *cur.next(false)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier("c", &mut interner)
    );
    assert_eq!(
        *cur.peek(2, false)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier("f", &mut interner)
    );
}

#[test]
fn peek_next_till_end() {
    let mut interner = Interner::new();
    let mut cur = BufferedLexer::from(Lexer::new(&b"a b c d e f g h i"[..], &interner));

    loop {
        let peek = cur.peek(0, false).unwrap().cloned();
        let next = cur.next(false).unwrap();

        assert_eq!(peek, next);

        if peek.is_none() {
            break;
        }
    }
}

#[test]
fn peek_skip_next_till_end() {
    let mut interner = Interner::new();
    let mut cur = BufferedLexer::from(Lexer::new(&b"a b c d e f g h i"[..], &interner));

    let mut peeked: [Option<Token>; super::MAX_PEEK_SKIP + 1] =
        [None::<Token>, None::<Token>, None::<Token>];

    loop {
        for (i, peek) in peeked.iter_mut().enumerate() {
            *peek = cur.peek(i, false).unwrap().cloned();
        }

        for peek in &peeked {
            assert_eq!(&cur.next(false).unwrap(), peek);
        }

        if peeked[super::MAX_PEEK_SKIP - 1].is_none() {
            break;
        }
    }
}

#[test]
fn skip_peeked_terminators() {
    let mut interner = Interner::new();
    let mut cur = BufferedLexer::from(Lexer::new(&b"A \n B"[..], &interner));
    assert_eq!(
        *cur.peek(0, false)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier("A", &mut interner)
    );
    assert_eq!(
        *cur.peek(0, true)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier("A", &mut interner)
    );

    assert_eq!(
        *cur.peek(1, false)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::LineTerminator,
    );
    assert_eq!(
        *cur.peek(1, true)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier("B", &mut interner) // This value is after the line terminator
    );

    assert_eq!(
        *cur.peek(2, false)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier("B", &mut interner)
    );
    // End of stream
    assert!(cur.peek(2, true).unwrap().is_none());
}
