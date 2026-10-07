// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Issues are pieces of information that record what happened during execution.
//!
//! Unlike fatal errors returned via `Result<T, E>`, an `Issue` is non-blocking.
//! It captures domain occurrences (errors, warnings, informational messages)
//! and accumulates them inside an [`IssueList`] without interrupting the
//! primary evaluation flow.

use miette::{Report, Severity};

use crate::SrcReferrer;

/// A trait to define an issue.
pub trait Issue:
    From<Self::Err> + From<Self::Warn> + From<Self::Info> + SrcReferrer + Into<Report>
{
    type Err;
    type Warn;
    type Info;

    fn severity(&self) -> Severity;
}

/// An issue list contains issues (errors, warnings, infos) and tracks error and warning count.
#[derive(Debug)]
pub struct IssueList<I: Issue> {
    issues: Vec<I>,
    err_count: usize,
    warn_count: usize,
}

impl<I: Issue> IssueList<I> {
    /// Create new issue list.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the total number of issues (errors, warnings, and infos).
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

impl<I: Issue> From<Vec<I::Err>> for IssueList<I> {
    fn from(errors: Vec<I::Err>) -> Self {
        let mut issues = IssueList::new();
        errors.into_iter().for_each(|err| {
            issues.push_err(err);
        });
        issues
    }
}

pub trait PushIssue<I: Issue> {
    fn push_issue(&mut self, issue: impl Into<I>);
    fn push_err(&mut self, err: impl Into<I::Err>);
    fn push_warn(&mut self, warn: impl Into<I::Warn>);
    fn push_info(&mut self, info: impl Into<I::Info>);

    /// Pushes the error and returns a default value for `T`, so compilation can continue.
    fn catch<T: Default>(&mut self, err: impl Into<I::Err>) -> Result<T, I::Err> {
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
    fn push_issue(&mut self, issue: impl Into<I>) {
        self.issues.push(issue.into());
    }

    fn push_err(&mut self, err: impl Into<I::Err>) {
        self.err_count += 1;
        // Convert input -> I::Err -> I
        let err_type: I::Err = err.into();
        self.push_issue(err_type);
    }

    fn push_warn(&mut self, warn: impl Into<I::Warn>) {
        self.warn_count += 1;
        let warn_type: I::Warn = warn.into();
        self.push_issue(warn_type);
    }

    fn push_info(&mut self, info: impl Into<I::Info>) {
        let info_type: I::Info = info.into();
        self.push_issue(info_type);
    }
}
