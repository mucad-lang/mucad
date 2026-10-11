// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Compiler High-level API

pub use crate::prelude as mu;

use microcad_lang_base::Issue;

/// Result of the compiler: the output bundled with the diagnostics of all stages run so far.
///
/// - `Ok`: The stage produced an output. The diagnostics may contain errors the stage recovered from.
/// - `Err`: The stage produced no output. The diagnostics contain at least one error.
///
/// A single language stage returns an [`mu::IssueResult`] with typed issues instead,
/// which is converted with [`IssueResultExt::into_compile_result`].
type CompileResult<T> = Result<(T, mu::Diagnostics), mu::Diagnostics>;

/// Convert the typed issues of a language stage into diagnostics.
pub trait IssueResultExt<T> {
    fn into_compile_result(self) -> CompileResult<T>;
}

impl<T, I: Issue> IssueResultExt<T> for mu::IssueResult<T, I> {
    fn into_compile_result(self) -> CompileResult<T> {
        self.map(|(value, issues)| (value, issues.into_diagnostics()))
            .map_err(|issues| issues.into_diagnostics())
    }
}

/// Chain compiler stages.
pub trait CompileResultExt<T> {
    /// Run the next stage on the output and merge the diagnostics of both stages.
    ///
    /// If this stage failed, the next stage is not run and the diagnostics are passed on.
    fn then<U>(self, next: impl FnOnce(T) -> CompileResult<U>) -> CompileResult<U>;
}

impl<T> CompileResultExt<T> for CompileResult<T> {
    fn then<U>(self, next: impl FnOnce(T) -> CompileResult<U>) -> CompileResult<U> {
        let (value, mut diags) = self?;
        match next(value) {
            Ok((value, next_diags)) => {
                diags.append(next_diags);
                Ok((value, diags))
            }
            Err(next_diags) => {
                diags.append(next_diags);
                Err(diags)
            }
        }
    }
}

pub struct Compiler {
    resolve_context: mu::ResolveContext,
}

impl Compiler {
    pub fn new() -> Compiler {
        Self {
            resolve_context: mu::ResolveContext::new(),
        }
    }

    /// Compile a source into a model tree.
    ///
    /// Parsing and lowering happen while resolving, because resolving loads further source files.
    pub fn compile(&mut self, source: impl Into<mu::Source>) -> CompileResult<mu::ModelTree> {
        mu::resolve::resolve_source(source, &mut self.resolve_context)
            .into_compile_result()
            .then(|lib| {
                let lib_cache = self.resolve_context.lib_cache();
                let lib_id = lib_cache.write_unwrap().insert_root(lib);
                let eval_context = mu::EvalContext::builder()
                    .with_lib_cache(lib_cache)
                    .with_lib_id(lib_id)
                    .build();
                mu::eval::eval(eval_context).into_compile_result()
            })
    }
}
