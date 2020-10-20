/*!
This is an experimental Javascript lexer, parser and compiler written in Rust. Currently, it has support for some of the language.

# Crate Features
 - **serde** - Enables serialization and deserialization of the AST (Abstract Syntax Tree).
 - **console** - Enables `boa`s WHATWG `console` object implementation.
 - **profiler** - Enables profiling with measureme (this is mostly internal).

**/

#![doc(
    html_logo_url = "https://raw.githubusercontent.com/jasonwilliams/boa/master/assets/logo.svg",
    html_favicon_url = "https://raw.githubusercontent.com/jasonwilliams/boa/master/assets/logo.svg"
)]
#![deny(
    unused_qualifications,
    clippy::all,
    unused_qualifications,
    unused_import_braces,
    unused_lifetimes,
    unreachable_pub,
    trivial_numeric_casts,
    // rustdoc,
    missing_debug_implementations,
    missing_copy_implementations,
    deprecated_in_future,
    meta_variable_misuse,
    non_ascii_idents,
    rust_2018_compatibility,
    rust_2018_idioms,
    future_incompatible,
    nonstandard_style,
)]
#![warn(clippy::perf, clippy::single_match_else, clippy::dbg_macro)]
#![allow(
    clippy::missing_inline_in_public_items,
    clippy::cognitive_complexity,
    clippy::must_use_candidate,
    clippy::missing_errors_doc,
    clippy::as_conversions,
    clippy::let_unit_value,
    missing_doc_code_examples
)]

pub mod builtins;
pub mod class;
pub mod environment;
pub mod exec;
pub mod gc;
pub mod object;
pub mod profiler;
pub mod property;
pub mod realm;
pub mod syntax;
pub mod value;

pub mod context;

pub(crate) use crate::{
    exec::Executable,
    gc::{empty_trace, Finalize, Trace},
    profiler::BoaProfiler,
};
use rustc_hash::FxHasher;
use std::{hash::BuildHasherDefault, num::NonZeroUsize, result::Result as StdResult};
use string_interner::{backend::BucketBackend, StringInterner, Symbol};

// Export things to root level
use crate::syntax::{
    ast::node::StatementList,
    parser::{ParseError, Parser},
};
#[doc(inline)]
pub use crate::{context::Context, value::Value};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// The result of a Javascript expression is represented like this so it can succeed (`Ok`) or fail (`Err`)
#[must_use]
pub type Result<T> = StdResult<T, Value>;

/// Type used as a symbol for the string interner.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Finalize)]
struct Sym {
    val: NonZeroUsize,
}

impl Symbol for Sym {
    fn try_from_usize(val: usize) -> Option<Self> {
        Some(Self {
            val: NonZeroUsize::new(val + 1)?,
        })
    }

    fn to_usize(self) -> usize {
        self.val.get() - 1
    }
}

// TODO: waiting for <https://github.com/Manishearth/rust-gc/issues/87> to remove unsafe code.
unsafe impl Trace for Sym {
    empty_trace!();
}

/// Type used as a string interner.
pub type Interner = StringInterner<Sym, BucketBackend<Sym>, BuildHasherDefault<FxHasher>>;

/// Parses the given source code.
///
/// It will return either the statement list AST node for the code, or a parsing error if something
/// goes wrong.
#[inline]
pub fn parse(
    src: &str,
    strict_mode: bool,
    interner: Option<&Interner>,
) -> StdResult<StatementList, ParseError> {
    Parser::new(src.as_bytes(), strict_mode, interner).parse_all()
}

/// Execute the code using an existing Context
/// The str is consumed and the state of the Context is changed
#[cfg(test)]
pub(crate) fn forward(engine: &mut Context, src: &str) -> String {
    // Setup executor
    let expr = match parse(src, false) {
        Ok(res) => res,
        Err(e) => {
            return format!(
                "Uncaught {}",
                engine
                    .throw_syntax_error(e.to_string())
                    .expect_err("interpreter.throw_syntax_error() did not return an error")
                    .display()
            );
        }
    };
    expr.run(engine).map_or_else(
        |e| format!("Uncaught {}", e.display()),
        |v| v.display().to_string(),
    )
}

/// Execute the code using an existing Context.
/// The str is consumed and the state of the Context is changed
/// Similar to `forward`, except the current value is returned instad of the string
/// If the interpreter fails parsing an error value is returned instead (error object)
#[allow(clippy::unit_arg, clippy::drop_copy)]
#[cfg(test)]
pub(crate) fn forward_val(engine: &mut Context, src: &str) -> Result<Value> {
    let main_timer = BoaProfiler::global().start_event("Main", "Main");
    // Setup executor
    let result = parse(src, false)
        .map_err(|e| {
            engine
                .throw_syntax_error(e.to_string())
                .expect_err("interpreter.throw_syntax_error() did not return an error")
        })
        .and_then(|expr| expr.run(engine));

    // The main_timer needs to be dropped before the BoaProfiler is.
    drop(main_timer);
    BoaProfiler::global().drop();

    result
}

/// Create a clean Context and execute the code
#[cfg(test)]
pub(crate) fn exec(src: &str) -> String {
    match Context::new().eval(src) {
        Ok(value) => value.display().to_string(),
        Err(error) => error.display().to_string(),
    }
}
