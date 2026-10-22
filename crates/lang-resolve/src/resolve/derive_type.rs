// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! The types of expressions, as far as they are known before evaluation.

use mucad_builtin::BuiltinLibrary;
use mucad_lang_base::{ExternalId, SymbolId};
use mucad_lang_lower::ir;
use mucad_lang_types::{Ty, Type};

use crate::{Library, LibraryCache, SymbolDef, SymbolNodeRef};

/// The depth to follow constants and aliases, which stops at cyclic definitions.
const MAX_DEPTH: usize = 16;

/// The symbols to derive types from: of a library, of the libraries it uses and the built-ins.
#[derive(Clone, Copy)]
pub struct TypeContext<'a> {
    lib: &'a Library,
    /// The other libraries, e.g. `std`. Without it, their symbols have no known types.
    lib_cache: Option<&'a LibraryCache>,
    builtins: &'a BuiltinLibrary,
    /// The depth of followed constants and aliases.
    depth: usize,
}

/// Derive the type of an expression, as far as it is known before evaluation.
pub trait DeriveType {
    /// The type of the expression or `Any` if it is unknown, e.g. of a local variable.
    ///
    /// `Any` matches every type, so an unknown type is never a type error.
    fn derive_type(&self, ctx: &TypeContext) -> Type;
}

impl<'a> TypeContext<'a> {
    /// A context for the symbols of `lib` and the built-ins.
    pub fn new(lib: &'a Library, builtins: &'a BuiltinLibrary) -> Self {
        Self {
            lib,
            lib_cache: None,
            builtins,
            depth: 0,
        }
    }

    /// Derive the types of symbols of other libraries, e.g. of `std`.
    pub fn with_lib_cache(mut self, lib_cache: &'a LibraryCache) -> Self {
        self.lib_cache = Some(lib_cache);
        self
    }

    /// The type of the value of the symbol of `path`, e.g. of a constant.
    fn path_type(&self, path: &ir::Path) -> Type {
        self.with_symbol(path.symbol_id(), |symbol, ctx| match symbol {
            Symbol::Def(SymbolDef::Constant(constant)) if constant.ty.ty != Type::Any => {
                constant.ty.ty.clone()
            }
            Symbol::Def(SymbolDef::Constant(constant)) => constant.expr.derive_type(ctx),
            Symbol::Def(SymbolDef::Function(function)) => function.ty(),
            Symbol::Def(_) => Type::Any,
            Symbol::Builtin(item) => item.ty().unwrap_or_default(),
        })
    }

    /// The type of the result of calling the symbol of `path`, e.g. the return type of a function.
    fn call_type(&self, path: &ir::Path) -> Type {
        self.with_symbol(path.symbol_id(), |symbol, _| match symbol {
            Symbol::Def(SymbolDef::Function(function)) => {
                function.return_type().unwrap_or_default()
            }
            // A sketch is a 2D model, a part a 3D model.
            Symbol::Def(SymbolDef::Workbench(workbench)) => workbench.signature.kind.ty(),
            Symbol::Def(_) => Type::Any,
            // A built-in without return type, e.g. `debug::print`, returns nothing.
            Symbol::Builtin(item) => item.return_type().unwrap_or_default(),
        })
    }

    /// Run `f` with the symbol `id` stands for and a context of its library.
    ///
    /// Aliases are followed, e.g. `std::print` for `__mu::debug::print`.
    /// Locals and unknown symbols have the type `Any`.
    fn with_symbol(
        &self,
        id: Option<&SymbolId>,
        f: impl FnOnce(Symbol, &TypeContext) -> Type,
    ) -> Type {
        if self.depth > MAX_DEPTH {
            return Type::Any;
        }
        let ctx = Self {
            depth: self.depth + 1,
            ..*self
        };

        match id {
            Some(SymbolId::Item(node)) => match self.lib.symbol_id(*node, None) {
                SymbolId::Item(node) => match self.lib.arena.get(node) {
                    Some(node) => f(Symbol::Def(&node.get().def), &ctx),
                    None => Type::Any,
                },
                // An alias of a symbol of another library or of a built-in.
                id => ctx.with_symbol(Some(&id), f),
            },
            Some(SymbolId::External(ExternalId { lib_id, id: node })) => {
                let Some(lib) = self.lib_cache.and_then(|lib_cache| lib_cache.get(lib_id)) else {
                    return Type::Any;
                };
                match lib.symbol_id(*node, Some(*lib_id)) {
                    SymbolId::External(ExternalId {
                        lib_id: alias_lib,
                        id,
                    }) if alias_lib == *lib_id => {
                        let node = SymbolNodeRef::new(id, &lib.arena);
                        // A context of the other library, which only lives while it is borrowed.
                        let lib_ctx = TypeContext {
                            lib: &lib,
                            lib_cache: ctx.lib_cache,
                            builtins: ctx.builtins,
                            depth: ctx.depth,
                        };
                        f(Symbol::Def(&node.get().def), &lib_ctx)
                    }
                    // An alias of a symbol of a further library or of a built-in.
                    id => ctx.with_symbol(Some(&id), f),
                }
            }
            Some(SymbolId::Builtin(id)) => match self.builtins.get(*id) {
                Some(item) => f(Symbol::Builtin(item), &ctx),
                None => Type::Any,
            },
            Some(SymbolId::Local(_)) | None => Type::Any,
        }
    }
}

/// A symbol to derive a type from.
enum Symbol<'a> {
    /// A symbol of a library.
    Def(&'a SymbolDef),
    /// A built-in item.
    Builtin(&'static mucad_builtin::BuiltinItem),
}

/// The common type of the branches of an `if`, or `Any` if they differ or one is missing.
fn if_type<Expr: ir::ExprSpec>(
    if_: &ir::If<Expr>,
    body_type: &dyn Fn(&Expr::Body) -> Type,
) -> Type {
    let else_type = match (&if_.next_if, &if_.body_else) {
        (Some(next_if), _) => if_type(next_if, body_type),
        (None, Some(body_else)) => body_type(body_else),
        // Without `else`, the `if` has no value if its condition is false.
        (None, None) => return Type::Any,
    };
    match body_type(&if_.body) {
        ty if ty == else_type => ty,
        _ => Type::Any,
    }
}

impl DeriveType for ir::ConstantExpression {
    fn derive_type(&self, ctx: &TypeContext) -> Type {
        match self {
            ir::ConstantExpression::Value(value) => value.ty(),
            ir::ConstantExpression::Path(path) => ctx.path_type(path),
            ir::ConstantExpression::Call(call) => ctx.call_type(&call.path),
            ir::ConstantExpression::Invalid => Type::Any,
        }
    }
}

impl DeriveType for ir::WorkbenchExpression {
    fn derive_type(&self, ctx: &TypeContext) -> Type {
        match self {
            ir::WorkbenchExpression::Value(value) => value.ty(),
            ir::WorkbenchExpression::Path(path) => ctx.path_type(path),
            ir::WorkbenchExpression::Call(call) => ctx.call_type(&call.path),
            // The model of a group depends on its statements.
            ir::WorkbenchExpression::If(if_) => if_type(if_, &|_| Type::Any),
            ir::WorkbenchExpression::Group(_)
            | ir::WorkbenchExpression::Marker(_)
            | ir::WorkbenchExpression::Invalid => Type::Any,
            _ => Type::Any,
        }
    }
}

impl DeriveType for ir::FunctionExpression {
    fn derive_type(&self, ctx: &TypeContext) -> Type {
        match self {
            ir::FunctionExpression::Value(value) => value.ty(),
            ir::FunctionExpression::Path(path) => ctx.path_type(path),
            ir::FunctionExpression::Call(call) => ctx.call_type(&call.path),
            ir::FunctionExpression::Scope(scope) => scope.derive_type(ctx),
            ir::FunctionExpression::If(if_) => if_type(if_, &|scope| scope.derive_type(ctx)),
            ir::FunctionExpression::Invalid => Type::Any,
            _ => Type::Any,
        }
    }
}

/// The type of the value of a scope: of its last statement, e.g. a calculation, an `if` or a
/// `return`.
impl DeriveType for ir::function::Scope {
    fn derive_type(&self, ctx: &TypeContext) -> Type {
        match self.statements.last() {
            Some(statement) => statement.derive_type(ctx),
            None => Type::Any,
        }
    }
}

/// The type of the value of a statement, if it is the last one of a scope.
impl DeriveType for ir::FunctionStatement {
    fn derive_type(&self, ctx: &TypeContext) -> Type {
        match self {
            ir::FunctionStatement::Tail(expr) => expr.derive_type(ctx),
            ir::FunctionStatement::Call(call) => ctx.call_type(&call.path),
            ir::FunctionStatement::If(if_) => if_type(if_, &|scope| scope.derive_type(ctx)),
            ir::FunctionStatement::Scope(scope) => scope.derive_type(ctx),
            ir::FunctionStatement::Return(ir::function::ReturnStatement {
                expr: Some(expr),
                ..
            }) => expr.derive_type(ctx),
            ir::FunctionStatement::Return(_) | ir::FunctionStatement::Local(_) => Type::Any,
            _ => Type::Any,
        }
    }
}
