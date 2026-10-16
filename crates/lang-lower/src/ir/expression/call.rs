// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Syntax elements related to calls.

use crate::{CastInto, ir};
use derive_more::Display;
use microcad_builtin::BuiltinId;
use microcad_lang_base::{Identifier, SrcRef, SrcReferrer, boxed};

use microcad_lang_types::Value;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Hash, Serialize, Deserialize)]
#[serde(bound(serialize = "Expr: Serialize", deserialize = "Expr: Deserialize<'de>"))]
pub enum Argument<Expr> {
    /// Explicitly positional: `10` or `x + 1`
    Unnamed(Expr),
    /// Explicitly named by caller: `b = 12`
    Named {
        name: Identifier,
        expr: Expr,
        src_ref: SrcRef,
    },
    /// Implicitly named by caller, if the expression contains a single identifier.
    Inline {
        name: Identifier,
        expr: Expr,
        src_ref: SrcRef,
    },
}

/// Builder methods for spec.
impl<Expr> Argument<Expr> {
    /// Create a new named argument (e.g. `a = 3.0`).
    pub fn named(name: impl Into<Identifier>, expr: impl Into<Expr>) -> Self {
        Self::Named {
            name: name.into(),
            expr: expr.into(),
            src_ref: SrcRef::none(),
        }
    }

    /// Create a new unnamed argument from expression.
    pub fn unnamed(expr: impl Into<Expr>) -> Self {
        Self::Unnamed(expr.into())
    }
}

impl<Expr> Argument<Expr> {
    pub fn name(&self) -> Option<&Identifier> {
        match self {
            Argument::Unnamed(_) => None,
            Argument::Named { name, .. } | Argument::Inline { name, .. } => Some(name),
        }
    }

    pub fn expr(&self) -> &Expr {
        match self {
            Argument::Unnamed(expr)
            | Argument::Named { expr, .. }
            | Argument::Inline { expr, .. } => expr,
        }
    }
    /// How the caller names the argument, e.g. `r` in `f(r * 2)` is an inline name.
    pub fn argument_name(&self) -> microcad_lang_types::ArgumentName {
        use microcad_lang_types::ArgumentName;
        match self {
            Argument::Unnamed(_) => ArgumentName::None,
            Argument::Named { name, .. } => ArgumentName::Named(name.clone()),
            Argument::Inline { name, .. } => ArgumentName::Inline(name.clone()),
        }
    }
}

impl<Expr> SrcReferrer for Argument<Expr>
where
    Expr: SrcReferrer,
{
    fn src_ref(&self) -> SrcRef {
        match self {
            Argument::Unnamed(expr) => expr.src_ref(),
            Argument::Named { src_ref, .. } | Argument::Inline { src_ref, .. } => *src_ref,
        }
    }
}

impl<Src: ir::ExprSpec, Dst: ir::ExprSpec> CastInto<Argument<Dst>> for Argument<Src>
where
    Src: CastInto<Dst>,
{
    fn cast_into(self) -> Argument<Dst> {
        match self {
            Argument::Unnamed(pos) => Argument::Unnamed(pos.cast_into()),
            Argument::Named {
                name,
                expr,
                src_ref,
            } => Argument::Named {
                name,
                expr: expr.cast_into(),
                src_ref,
            },
            Argument::Inline {
                name,
                expr,
                src_ref,
            } => Argument::Inline {
                name,
                expr: expr.cast_into(),
                src_ref,
            },
        }
    }
}

impl<Expr> std::fmt::Display for Argument<Expr>
where
    Expr: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Argument::Unnamed(expr) => write!(f, "{expr}"),
            Argument::Named { name, expr, .. } => write!(f, "{name} = {expr}"),
            Argument::Inline { name, expr, .. } => write!(f, "{name}? = {expr}"),
        }
    }
}

impl<Expr: ir::ExprSpec> From<Expr> for Argument<Expr> {
    fn from(expr: Expr) -> Self {
        match expr.single_identifier() {
            Some(name) => Self::Named {
                name: name.clone(),
                src_ref: expr.src_ref(),
                expr,
            },
            None => Self::Unnamed(expr),
        }
    }
}

/// Arguments in a [`Call`].
#[derive(Debug, Clone, PartialEq, Hash, Serialize, Deserialize)]
#[serde(bound(serialize = "Expr: Serialize", deserialize = "Expr: Deserialize<'de>"))]
pub struct ArgumentList<Expr> {
    /// Source code reference
    pub src_ref: SrcRef,

    /// The arguments in the list
    pub args: Box<[Argument<Expr>]>,
}

impl<Expr> ArgumentList<Expr> {
    /// Prepends an argument to the front of the argument list.
    pub fn prepend(&mut self, arg: impl Into<Argument<Expr>>) {
        let mut new_args = Vec::with_capacity(self.args.len() + 1);
        new_args.push(arg.into());
        new_args.extend(Vec::from(std::mem::take(&mut self.args)));
        self.args = new_args.into_boxed_slice();
    }

    /// Consumes `self` and returns a new `ArgumentList` with `expr` prepended to `unnamed_args`.
    pub fn prepended(mut self, arg: impl Into<Argument<Expr>>) -> Self {
        self.prepend(arg);
        self
    }

    /// Prepend the `self` argument to [`ArgumentList`].
    ///
    /// This is used when desugaring AST method calls into IR calls, e.g.:
    /// `Circle(radius = 4.0).translate(x = 1.0mm, y = 1.0mm, z = 1.0mm)`
    ///
    /// becomes
    ///
    /// translate(self = Circle(radius = 4.0), x = 1.0mm, y = 1.0mm, z = 1.0mm)
    pub fn desugar_method(self, expr: impl Into<Expr>) -> Self {
        self.prepended(Argument::named("self", expr))
    }
}

impl<Expr: ir::ExprSpec> ArgumentList<Expr> {
    /// Extract first argument value of unnamed arguments.
    pub fn extract_first_unnamed_arg_expr(&self) -> Option<&Expr> {
        for arg in self.args.iter() {
            if let ir::Argument::Unnamed(expr) = arg {
                return Some(expr);
            }
        }

        None
    }

    /// Extract argument value from named arguments.
    pub fn extract_named_arg_expr(&self, name: impl AsRef<str>) -> Option<&Expr> {
        let arg_name = name.as_ref();

        for arg in self.args.iter() {
            match arg {
                ir::Argument::Named {
                    name,
                    expr,
                    src_ref,
                } if name.to_string().as_str() == arg_name => {
                    return Some(expr);
                }
                _ => {}
            }
        }

        None
    }

    /// Extract argument value from named arguments.
    pub fn extract_arg_value(&self, name: impl AsRef<str>) -> Option<&Value> {
        let arg_name = name.as_ref();

        for arg in self.args.iter() {
            match arg {
                ir::Argument::Named {
                    name,
                    expr,
                    src_ref,
                } if name.to_string().as_str() == arg_name => {
                    return expr.value();
                }
                _ => {}
            }
        }

        None
    }
}

impl<Expr> Default for ArgumentList<Expr> {
    fn default() -> Self {
        Self {
            src_ref: SrcRef::none(),
            args: Box::new([]),
        }
    }
}

impl<Expr> From<Vec<Argument<Expr>>> for ArgumentList<Expr> {
    fn from(args: Vec<Argument<Expr>>) -> Self {
        Self {
            args: args.into_boxed_slice(),
            src_ref: SrcRef::none(),
        }
    }
}

impl<S, Expr> FromIterator<S> for ArgumentList<Expr>
where
    S: Into<Argument<Expr>>,
{
    fn from_iter<T: IntoIterator<Item = S>>(iter: T) -> Self {
        Self {
            src_ref: SrcRef::default(),
            args: boxed(iter.into_iter().map(|arg| arg.into())),
        }
    }
}

impl<Src: ir::ExprSpec, Dst: ir::ExprSpec> CastInto<ArgumentList<Dst>> for ArgumentList<Src>
where
    Src: CastInto<Dst>,
{
    fn cast_into(self) -> ArgumentList<Dst> {
        ArgumentList {
            src_ref: self.src_ref,
            args: self.args.cast_into(),
        }
    }
}

/// Call of a *workbench* or *function*.
#[derive(Debug, Display, Clone, PartialEq, Hash, Serialize, Deserialize)]
#[display("{path}({args})")]
#[serde(bound(serialize = "Expr: Serialize", deserialize = "Expr: Deserialize<'de>"))]
pub struct Call<Expr: ir::ExprSpec> {
    /// Path of the call.
    pub path: ir::Path,
    /// Argument list of the call.
    pub args: ir::ArgumentList<Expr>,
    /// Source code reference.
    pub src_ref: SrcRef,
}

impl<Expr: ir::ExprSpec> SrcReferrer for Call<Expr> {
    fn src_ref(&self) -> SrcRef {
        self.src_ref
    }
}

/// Builder methods
impl<Expr: ir::ExprSpec> Call<Expr> {
    pub fn builtin(builtin_id: BuiltinId) -> Self {
        Self {
            path: ir::Path::from(builtin_id),
            args: ir::ArgumentList::default(),
            src_ref: SrcRef::none(),
        }
    }

    pub fn with_args(mut self, args: impl Into<ArgumentList<Expr>>) -> Self {
        self.args = args.into();
        self
    }
}

impl<Src: ir::ExprSpec, Dst: ir::ExprSpec> CastInto<Call<Dst>> for Call<Src>
where
    Src: CastInto<Dst>,
{
    fn cast_into(self) -> Call<Dst> {
        Call {
            path: self.path,
            args: self.args.cast_into(),
            src_ref: self.src_ref,
        }
    }
}
