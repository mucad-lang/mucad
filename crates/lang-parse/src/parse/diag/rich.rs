// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! A copy of Rich, but with a generic `Custom` error type instead of a string
//!
//! Hopefully we can remove this again in the future if https://github.com/zesterer/chumsky/issues/959 gets resolved

// to minimize code changes from upstream, we disable some warnings
#![allow(dead_code, clippy::unwrap_used)]

use chumsky::error::{Error, LabelError};
use chumsky::input::Input;
use chumsky::util::MaybeRef;
use chumsky::{DefaultExpected, text};
use mucad_lang_base::Span;
use std::borrow::Cow;

use crate::parse::ParseErrorKind;
use crate::token::Token;

/// An expected pattern for a [`Rich`] error.
#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[non_exhaustive]
pub enum RichPattern<'a, T> {
    /// A specific token.
    Token(MaybeRef<'a, T>),
    /// A labelled pattern.
    Label(Cow<'a, str>),
    /// A specific keyword.
    Identifier(String),
    /// Anything other than the end of input.
    Any,
    /// Something other than the provided input.
    SomethingElse,
    /// The end of input.
    EndOfInput,
}

impl<'a, T> From<DefaultExpected<'a, T>> for RichPattern<'a, T> {
    fn from(expected: DefaultExpected<'a, T>) -> Self {
        match expected {
            DefaultExpected::Token(tok) => Self::Token(tok),
            DefaultExpected::Any => Self::Any,
            DefaultExpected::SomethingElse => Self::SomethingElse,
            DefaultExpected::EndOfInput => Self::EndOfInput,
            _ => Self::Any,
        }
    }
}

impl<'a, Slice: core::fmt::Debug, T> From<text::TextExpected<Slice>> for RichPattern<'a, T> {
    fn from(expected: text::TextExpected<Slice>) -> Self {
        match expected {
            text::TextExpected::Whitespace => Self::Label(Cow::Borrowed("whitespace")),
            text::TextExpected::InlineWhitespace => Self::Label(Cow::Borrowed("inline whitespace")),
            text::TextExpected::Newline => Self::Label(Cow::Borrowed("newline")),
            text::TextExpected::Digit(start, _end) if start > 0 => {
                Self::Label(Cow::Borrowed("non-zero digit"))
            }
            text::TextExpected::Digit(_, _) => Self::Label(Cow::Borrowed("digit")),
            text::TextExpected::AnyIdentifier => Self::Label(Cow::Borrowed("identifier")),
            text::TextExpected::Identifier(i) => Self::Identifier(format!("{i:?}")),
            text::TextExpected::Int => Self::Label(Cow::Borrowed("int")),
            _ => Self::Any,
        }
    }
}

impl<'a, T> From<MaybeRef<'a, T>> for RichPattern<'a, T> {
    fn from(tok: MaybeRef<'a, T>) -> Self {
        Self::Token(tok)
    }
}

impl<T> From<&'static str> for RichPattern<'_, T> {
    fn from(label: &'static str) -> Self {
        Self::Label(Cow::Borrowed(label))
    }
}

impl<T> From<String> for RichPattern<'_, T> {
    fn from(label: String) -> Self {
        Self::Label(Cow::Owned(label))
    }
}

impl From<char> for RichPattern<'_, char> {
    fn from(c: char) -> Self {
        Self::Token(MaybeRef::Val(c))
    }
}

impl<'a, T> RichPattern<'a, T> {
    /// Transform this pattern's tokens using the given function.
    ///
    /// This is useful when you wish to combine errors from multiple compilation passes (lexing and parsing, say) where
    /// the token type for each pass is different (`char` vs `MyToken`, say).
    pub fn map_token<U, F: FnMut(T) -> U>(self, mut f: F) -> RichPattern<'a, U>
    where
        T: Clone,
    {
        match self {
            Self::Token(t) => RichPattern::Token(f(t.into_inner()).into()),
            Self::Label(l) => RichPattern::Label(l),
            Self::Identifier(i) => RichPattern::Identifier(i),
            Self::Any => RichPattern::Any,
            Self::SomethingElse => RichPattern::SomethingElse,
            Self::EndOfInput => RichPattern::EndOfInput,
        }
    }

    /// Convert this pattern into an owned version of itself by cloning any borrowed internal tokens, if necessary.
    pub fn into_owned<'b>(self) -> RichPattern<'b, T>
    where
        T: Clone,
    {
        match self {
            Self::Token(tok) => RichPattern::Token(tok.into_owned()),
            Self::Label(l) => RichPattern::Label(Cow::Owned(l.into_owned())),
            Self::Identifier(i) => RichPattern::Identifier(i),
            Self::Any => RichPattern::Any,
            Self::SomethingElse => RichPattern::SomethingElse,
            Self::EndOfInput => RichPattern::EndOfInput,
        }
    }

    fn write(
        &self,
        f: &mut std::fmt::Formatter,
        mut fmt_token: impl FnMut(&T, &mut std::fmt::Formatter<'_>) -> std::fmt::Result,
    ) -> std::fmt::Result {
        match self {
            Self::Token(tok) => {
                write!(f, "'")?;
                fmt_token(tok, f)?;
                write!(f, "'")
            }
            Self::Label(l) => write!(f, "{l}"),
            Self::Identifier(i) => write!(f, "'{i}'"),
            Self::Any => write!(f, "any"),
            Self::SomethingElse => write!(f, "something else"),
            Self::EndOfInput => write!(f, "end of input"),
        }
    }
}

impl<T> std::fmt::Debug for RichPattern<'_, T>
where
    T: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.write(f, |t, f| write!(f, "{t:?}"))
    }
}

impl<T> std::fmt::Display for RichPattern<'_, T>
where
    T: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.write(f, |t, f| write!(f, "{t}"))
    }
}

/// The reason for a [`Rich`] error.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum RichReason<'a, T, C = String> {
    /// An unexpected input was found
    ExpectedFound {
        /// The tokens expected
        expected: Vec<RichPattern<'a, T>>,
        /// The tokens found
        found: Option<MaybeRef<'a, T>>,
    },
    /// An error with a custom message
    Custom(C),
}

impl<'a, T, C> RichReason<'a, T, C> {
    /// Return the token that was found by this error reason. `None` implies that the end of input was expected.
    pub fn found(&self) -> Option<&T> {
        match self {
            Self::ExpectedFound { found, .. } => found.as_deref(),
            Self::Custom(_) => None,
        }
    }

    /// Convert this reason into an owned version of itself by cloning any borrowed internal tokens, if necessary.
    pub fn into_owned<'b>(self) -> RichReason<'b, T, C>
    where
        T: Clone,
    {
        match self {
            Self::ExpectedFound { found, expected } => RichReason::ExpectedFound {
                expected: expected.into_iter().map(RichPattern::into_owned).collect(),
                found: found.map(MaybeRef::into_owned),
            },
            Self::Custom(msg) => RichReason::Custom(msg),
        }
    }

    fn take_found(&mut self) -> Option<MaybeRef<'a, T>> {
        match self {
            RichReason::ExpectedFound { found, .. } => found.take(),
            RichReason::Custom(_) => None,
        }
    }

    /// Transform this `RichReason`'s tokens using the given function.
    ///
    /// This is useful when you wish to combine errors from multiple compilation passes (lexing and parsing, say) where
    /// the token type for each pass is different (`char` vs `MyToken`, say).
    pub fn map_token<U, F: FnMut(T) -> U>(self, mut f: F) -> RichReason<'a, U, C>
    where
        T: Clone,
    {
        match self {
            RichReason::ExpectedFound { expected, found } => RichReason::ExpectedFound {
                expected: expected
                    .into_iter()
                    .map(|pat| pat.map_token(&mut f))
                    .collect(),
                found: found.map(|found| f(found.into_inner()).into()),
            },
            RichReason::Custom(msg) => RichReason::Custom(msg),
        }
    }
}

impl<'a, T, C: std::fmt::Display> RichReason<'a, T, C> {
    fn inner_fmt<S>(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        mut fmt_token: impl FnMut(&T, &mut std::fmt::Formatter<'_>) -> std::fmt::Result,
        mut fmt_span: impl FnMut(&S, &mut std::fmt::Formatter<'_>) -> std::fmt::Result,
        span: Option<&S>,
        context: &[(RichPattern<'a, T>, S)],
    ) -> std::fmt::Result {
        match self {
            RichReason::ExpectedFound { expected, found } => {
                write!(f, "found ")?;
                write_token(f, &mut fmt_token, found.as_deref())?;
                if let Some(span) = span {
                    write!(f, " at ")?;
                    fmt_span(span, f)?;
                }
                write!(f, " expected ")?;
                match &expected[..] {
                    [] => write!(f, "something else")?,
                    [expected] => expected.write(f, &mut fmt_token)?,
                    _ => {
                        for expected in &expected[..expected.len() - 1] {
                            expected.write(f, &mut fmt_token)?;
                            write!(f, ", ")?;
                        }
                        write!(f, "or ")?;
                        expected.last().unwrap().write(f, &mut fmt_token)?;
                    }
                }
            }
            RichReason::Custom(custom) => {
                write!(f, "{custom}")?;
                if let Some(span) = span {
                    write!(f, " at ")?;
                    fmt_span(span, f)?;
                }
            }
        }
        for (l, s) in context {
            write!(f, " in ")?;
            l.write(f, &mut fmt_token)?;
            write!(f, " at ")?;
            fmt_span(s, f)?;
        }
        Ok(())
    }
}

impl<T, C> RichReason<'_, T, C>
where
    T: PartialEq,
{
    #[inline]
    fn flat_merge(self, other: Self) -> Self {
        match (self, other) {
            // Prefer first error, if ambiguous
            (a @ RichReason::Custom(_), _) => a,
            (_, b @ RichReason::Custom(_)) => b,
            (
                RichReason::ExpectedFound {
                    expected: mut this_expected,
                    found,
                },
                RichReason::ExpectedFound {
                    expected: mut other_expected,
                    ..
                },
            ) => {
                // Try to avoid allocations if we possibly can by using the longer vector
                if other_expected.len() > this_expected.len() {
                    core::mem::swap(&mut this_expected, &mut other_expected);
                }
                for expected in other_expected {
                    if !this_expected[..].contains(&expected) {
                        this_expected.push(expected);
                    }
                }
                RichReason::ExpectedFound {
                    expected: this_expected,
                    found,
                }
            }
        }
    }
}

impl<T> std::fmt::Display for RichReason<'_, T>
where
    T: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner_fmt(f, T::fmt, |_: &(), _| Ok(()), None, &[])
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Rich<'a, T, S = Span, C = String> {
    span: S,
    reason: Box<RichReason<'a, T, C>>,
    context: Vec<(RichPattern<'a, T>, S)>,
}

impl<T, S, C: std::fmt::Display> Rich<'_, T, S, C> {
    fn inner_fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        fmt_token: impl FnMut(&T, &mut std::fmt::Formatter<'_>) -> std::fmt::Result,
        fmt_span: impl FnMut(&S, &mut std::fmt::Formatter<'_>) -> std::fmt::Result,
        with_spans: bool,
    ) -> std::fmt::Result {
        self.reason.inner_fmt(
            f,
            fmt_token,
            fmt_span,
            if with_spans { Some(&self.span) } else { None },
            &self.context,
        )
    }
}

impl<'a, T, S, C> Rich<'a, T, S, C> {
    /// Create an error with a custom message and span
    #[inline]
    pub fn custom<M: Into<C>>(span: S, custom: M) -> Self {
        Rich {
            span,
            reason: Box::new(RichReason::Custom(custom.into())),
            context: Vec::new(),
        }
    }

    /// Get the span associated with this error.
    ///
    /// If the span type is unspecified, it is [`SimpleSpan`].
    pub fn span(&self) -> &S {
        &self.span
    }

    /// Get the reason for this error.
    pub fn reason(&self) -> &RichReason<'a, T, C> {
        &self.reason
    }

    /// Take the reason from this error.
    pub fn into_reason(self) -> RichReason<'a, T, C> {
        *self.reason
    }

    /// Get the token found by this error when parsing. `None` implies that the error expected the end of input.
    pub fn found(&self) -> Option<&T> {
        self.reason.found()
    }

    /// Return an iterator over the labelled contexts of this error, from least general to most.
    ///
    /// 'Context' here means parser patterns that the parser was in the process of parsing when the error occurred. To
    /// add labelled contexts, see [`Parser::labelled`].
    pub fn contexts(&self) -> impl Iterator<Item = (&RichPattern<'a, T>, &S)> {
        self.context.iter().map(|(l, s)| (l, s))
    }

    /// Convert this error into an owned version of itself by cloning any borrowed internal tokens, if necessary.
    pub fn into_owned<'b>(self) -> Rich<'b, T, S, C>
    where
        T: Clone,
    {
        Rich {
            reason: Box::new(self.reason.into_owned()),
            context: self
                .context
                .into_iter()
                .map(|(p, s)| (p.into_owned(), s))
                .collect(),
            ..self
        }
    }

    /// Get an iterator over the expected items associated with this error
    pub fn expected(&self) -> impl ExactSizeIterator<Item = &RichPattern<'a, T>> {
        match &*self.reason {
            RichReason::ExpectedFound { expected, .. } => expected.iter(),
            RichReason::Custom(_) => [].iter(),
        }
    }

    /// Transform this error's tokens using the given function.
    ///
    /// This is useful when you wish to combine errors from multiple compilation passes (lexing and parsing, say) where
    /// the token type for each pass is different (`char` vs `MyToken`, say).
    pub fn map_token<U, F: FnMut(T) -> U>(self, mut f: F) -> Rich<'a, U, S, C>
    where
        T: Clone,
    {
        Rich {
            span: self.span,
            reason: Box::new(self.reason.map_token(&mut f)),
            context: self
                .context
                .into_iter()
                .map(|(p, s)| (p.map_token(&mut f), s))
                .collect(),
        }
    }
}

impl<'a, I: Input<'a>, C> Error<'a, I> for Rich<'a, I::Token, I::Span, C>
where
    I::Token: PartialEq,
{
    #[inline]
    fn merge(self, other: Self) -> Self {
        let new_reason = self.reason.flat_merge(*other.reason);
        Self {
            span: self.span,
            reason: Box::new(new_reason),
            context: self.context, // TODO: Merge contexts
        }
    }
}

impl<'a, I: Input<'a>, L, C> LabelError<'a, I, L> for Rich<'a, I::Token, I::Span, C>
where
    I::Token: PartialEq,
    L: Into<RichPattern<'a, I::Token>>,
{
    #[inline]
    fn expected_found<E: IntoIterator<Item = L>>(
        expected: E,
        found: Option<MaybeRef<'a, I::Token>>,
        span: I::Span,
    ) -> Self {
        Self {
            span,
            reason: Box::new(RichReason::ExpectedFound {
                expected: expected.into_iter().map(|tok| tok.into()).collect(),
                found,
            }),
            context: Vec::new(),
        }
    }

    #[inline]
    fn merge_expected_found<E: IntoIterator<Item = L>>(
        mut self,
        new_expected: E,
        new_found: Option<MaybeRef<'a, I::Token>>,
        _span: I::Span,
    ) -> Self {
        match &mut *self.reason {
            RichReason::ExpectedFound { expected, found } => {
                for new_expected in new_expected {
                    let new_expected = new_expected.into();
                    if !expected[..].contains(&new_expected) {
                        expected.push(new_expected);
                    }
                }
                *found = found.take().or(new_found); //land
            }
            RichReason::Custom(_) => {}
        }
        // TODO: Merge contexts
        self
    }

    #[inline]
    fn replace_expected_found<E: IntoIterator<Item = L>>(
        mut self,
        new_expected: E,
        new_found: Option<MaybeRef<'a, I::Token>>,
        span: I::Span,
    ) -> Self {
        self.span = span;
        match &mut *self.reason {
            RichReason::ExpectedFound { expected, found } => {
                expected.clear();
                expected.extend(new_expected.into_iter().map(|tok| tok.into()));
                *found = new_found;
            }
            _ => {
                *self.reason = RichReason::ExpectedFound {
                    expected: new_expected.into_iter().map(|tok| tok.into()).collect(),
                    found: new_found,
                };
            }
        }
        self.context.clear();
        self
    }

    #[inline]
    fn label_with(&mut self, label: L) {
        // Opportunistically attempt to reuse allocations if we can
        match &mut *self.reason {
            RichReason::ExpectedFound { expected, found: _ } => {
                expected.clear();
                expected.push(label.into());
            }
            _ => {
                *self.reason = RichReason::ExpectedFound {
                    expected: vec![label.into()],
                    found: self.reason.take_found(),
                };
            }
        }
    }

    #[inline]
    fn in_context(&mut self, label: L, span: I::Span) {
        let label = label.into();
        if self.context.iter().all(|(l, _)| l != &label) {
            self.context.push((label, span));
        }
    }
}

impl<T, S, C> std::fmt::Debug for Rich<'_, T, S, C>
where
    T: std::fmt::Debug,
    S: std::fmt::Debug,
    C: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.inner_fmt(f, T::fmt, S::fmt, true)
    }
}

impl<T, S, C> std::fmt::Display for Rich<'_, T, S, C>
where
    T: std::fmt::Display,
    S: std::fmt::Display,
    C: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner_fmt(f, T::fmt, S::fmt, false)
    }
}

fn write_token<T>(
    f: &mut std::fmt::Formatter,
    mut fmt_token: impl FnMut(&T, &mut std::fmt::Formatter<'_>) -> std::fmt::Result,
    tok: Option<&T>,
) -> std::fmt::Result {
    match tok {
        Some(tok) => {
            write!(f, "'")?;
            fmt_token(tok, f)?;
            write!(f, "'")
        }
        None => write!(f, "end of input"),
    }
}

/// Type alias for RichError
pub type RichError<'tokens> = Rich<'tokens, Token<'tokens>, Span, ParseErrorKind>;
