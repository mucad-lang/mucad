// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::prelude::*;

/// Parse a document from source.
pub trait Parse {
    fn parse(&mut self) -> Result;
}

/// Lower the AST of a Document into an intermediate representation (IR).
pub trait Lower {
    fn lower(&mut self) -> Result;
}

/// Parameters for resolve
#[derive(Clone, Default)]
pub struct ResolveParameters {
    pub search_paths: Vec<std::path::PathBuf>,
}

/// Resolve the IR into a symbol tree.
pub trait Resolve {
    fn resolve(&mut self, _params: impl Into<ResolveParameters>) -> Result;
}

/// Evaluate the symbol into a model.
pub trait Eval {
    //fn eval(&mut self) -> Result<Model>;
}

/// Compile parameters
#[derive(Default, Clone)]
pub struct CompileParameters {
    /// Resolve parameters.
    pub resolve: ResolveParameters,
}

/// Trait for compilation toolchain.
pub trait Compile /*+ Eval */ {
    /// Compile a document into a `Model`.
    fn compile(&mut self, _parameters: impl Into<CompileParameters>) -> Result {
        todo!()
    }
}
