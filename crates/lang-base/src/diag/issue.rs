// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Issues are pieces of information that record what happened during execution.
//!
//! Unlike fatal errors returned via `Result<T, E>`, an `Issue` is non-blocking.
//! It captures domain occurrences (errors, warnings, advices)
//! and accumulates them inside an [`IssueList`] without interrupting the
//! primary evaluation flow.

use miette::{Diagnostic, Severity};

use crate::{HashId, SrcReferrer};

/// A trait to define an issue.
pub trait Issue:
    From<Self::Error>
    + From<Self::Warning>
    + From<Self::Advice>
    + SrcReferrer
    + Diagnostic
    + Send
    + Sync
    + 'static
{
    type Error;
    type Warning;
    type Advice;

    fn severity(&self) -> Severity;

    /// The diagnostic of the error, warning or advice with its help and labels.
    fn diagnostic(&self) -> &dyn Diagnostic;

    /// A hash to find an equal issue, see [`IssueList`]: of its severity, code and message and
    /// of its position in the source.
    ///
    /// Source references have no hash of their own, so that they do not change the hashes of
    /// the code, their position is hashed instead. `#[derive(Issue)]` implements it.
    fn hash(&self) -> HashId;
}

/// An issue list contains issues (errors, warnings, advices) and tracks error and warning count.
///
/// An issue equal to one in the list, see [`Issue::hash`], is not added again, e.g. of code,
/// which is checked or evaluated twice.
///
/// Issues without a position in the source, e.g. of `__mu::debug::warning("…")` evaluated
/// twice, are all added, because they may be reported at different places.
#[derive(Debug)]
pub struct IssueList<I: Issue> {
    issues: Vec<I>,
    err_count: usize,
    warn_count: usize,
    /// The hashes of the issues with a position in the source.
    hashes: crate::HashSet<HashId>,
}

impl<I: Issue> IssueList<I> {
    /// Create new issue list.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the total number of issues (errors, warnings, and advices).
    #[inline]
    pub fn len(&self) -> usize {
        self.issues.len()
    }

    /// Returns `true` if no issues have been recorded.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.issues.is_empty()
    }

    /// Returns the total number of recorded fatal/hard errors.
    #[inline]
    pub fn err_count(&self) -> usize {
        self.err_count
    }

    /// Returns the total number of recorded warnings.
    #[inline]
    pub fn warn_count(&self) -> usize {
        self.warn_count
    }

    /// Returns `true` if at least one error was recorded.
    #[inline]
    pub fn has_errors(&self) -> bool {
        self.err_count > 0
    }

    /// Returns an iterator over references to the accumulated issues in emission order.
    #[inline]
    pub fn iter(&self) -> std::slice::Iter<'_, I> {
        self.issues.iter()
    }
}

impl<I: Issue> Default for IssueList<I> {
    fn default() -> Self {
        Self {
            issues: vec![],
            err_count: 0,
            warn_count: 0,
            hashes: Default::default(),
        }
    }
}

impl<I: Issue> IntoIterator for IssueList<I> {
    type Item = I;
    type IntoIter = std::vec::IntoIter<I>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.issues.into_iter()
    }
}

impl<I: Issue> From<Vec<I::Error>> for IssueList<I> {
    fn from(errors: Vec<I::Error>) -> Self {
        let mut issues = IssueList::new();
        errors.into_iter().for_each(|err| {
            issues.push_err(err);
        });
        issues
    }
}

pub trait PushIssue<I: Issue> {
    fn push_issue(&mut self, issue: impl Into<I>);
    fn push_err(&mut self, err: impl Into<I::Error>);
    fn push_warn(&mut self, warn: impl Into<I::Warning>);
    fn push_advice(&mut self, advice: impl Into<I::Advice>);

    /// Pushes the error and returns a default value for `T`, so compilation can continue.
    fn catch<T: Default>(&mut self, err: impl Into<I::Error>) -> Result<T, I::Error> {
        self.push_err(err);
        Ok(T::default())
    }

    /// Append issues
    fn append_issues<T>(&mut self, issues: impl IntoIterator<Item = T>)
    where
        T: Into<I>,
    {
        for issue in issues {
            self.push_issue(issue);
        }
    }
}

impl<I: Issue> PushIssue<I> for IssueList<I> {
    /// Push an issue and count it by its severity, unless an equal issue is in the list.
    fn push_issue(&mut self, issue: impl Into<I>) {
        let issue = issue.into();
        if issue.src_ref().is_some() && !self.hashes.insert(issue.hash()) {
            return;
        }
        match Issue::severity(&issue) {
            Severity::Error => self.err_count += 1,
            Severity::Warning => self.warn_count += 1,
            Severity::Advice => (),
        }
        self.issues.push(issue);
    }

    fn push_err(&mut self, err: impl Into<I::Error>) {
        // Convert input -> I::Error -> I
        let err_type: I::Error = err.into();
        self.push_issue(err_type);
    }

    fn push_warn(&mut self, warn: impl Into<I::Warning>) {
        let warn_type: I::Warning = warn.into();
        self.push_issue(warn_type);
    }

    fn push_advice(&mut self, advice: impl Into<I::Advice>) {
        let advice_type: I::Advice = advice.into();
        self.push_issue(advice_type);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SrcRef;

    /// A warning with a message at a position, e.g. of a deprecated symbol.
    #[derive(Debug)]
    struct TestIssue {
        message: &'static str,
        src_ref: SrcRef,
    }

    impl std::fmt::Display for TestIssue {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.message)
        }
    }

    impl std::error::Error for TestIssue {}
    impl Diagnostic for TestIssue {}

    impl SrcReferrer for TestIssue {
        fn src_ref(&self) -> SrcRef {
            self.src_ref
        }
    }

    impl Issue for TestIssue {
        type Error = Self;
        type Warning = Self;
        type Advice = Self;

        fn severity(&self) -> Severity {
            Severity::Warning
        }

        fn diagnostic(&self) -> &dyn Diagnostic {
            self
        }

        fn hash(&self) -> HashId {
            crate::hash_id!(
                self.message,
                self.src_ref.source_hash,
                self.src_ref.start,
                self.src_ref.end
            )
        }
    }

    fn at(start: usize) -> SrcRef {
        SrcRef {
            start,
            end: start + 1,
            source_hash: mucad_hash::HashId::new(42),
            ..Default::default()
        }
    }

    fn issue(message: &'static str, src_ref: SrcRef) -> TestIssue {
        TestIssue { message, src_ref }
    }

    /// An equal issue at the same position is not added again, issues without a position are.
    #[test]
    fn equal_issues() {
        let mut issues = IssueList::<TestIssue>::new();
        issues.push_issue(issue("Use of deprecated `f`", at(1)));
        issues.push_issue(issue("Use of deprecated `f`", at(1)));
        // Another message or position.
        issues.push_issue(issue("Use of deprecated `g`", at(1)));
        issues.push_issue(issue("Use of deprecated `f`", at(5)));
        // Without a position, e.g. of `__mu::debug::warning("…")` evaluated twice.
        issues.push_issue(issue("Check", SrcRef::none()));
        issues.push_issue(issue("Check", SrcRef::none()));

        let messages: Vec<_> = issues.iter().map(|issue| issue.message).collect();
        assert_eq!(
            messages,
            [
                "Use of deprecated `f`",
                "Use of deprecated `g`",
                "Use of deprecated `f`",
                "Check",
                "Check"
            ]
        );
        assert_eq!(issues.warn_count(), 5);
    }
}
