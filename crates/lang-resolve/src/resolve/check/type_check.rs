// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Type check of parameters, constants and statements, and the inference of parameter types.

use mucad_builtin::BuiltinLibrary;
use mucad_lang_base::{IssueList, PushIssue, SrcRef, SrcReferrer, element::WorkbenchKind};
use mucad_lang_lower::ir;
use mucad_lang_types::{Ty, Type};

use super::{Check, CheckContext};
use crate::{
    DeriveType, Library, LibraryCache, ResolveErrorKind, ResolveIssue, SymbolDef, TypeContext,
    library::{Symbol, visitor},
};

/// Give the parameters without type the type of their default value, if it is known,
/// e.g. `Length` for `radius = 1mm`. An explicit `Any` stays.
///
/// Call it after the paths have been resolved, which finds the symbols of the default values.
pub(crate) fn infer_parameter_types(
    lib: &mut Library,
    builtins: &BuiltinLibrary,
    lib_cache: &LibraryCache,
) {
    // Derive the types with the unchanged library, then set them.
    let inferred: Vec<_> = {
        let ctx = TypeContext::new(lib, builtins).with_lib_cache(lib_cache);
        lib.root
            .descendants(&lib.arena)
            .map(|node| {
                let types: Vec<Vec<Option<ir::Type>>> = lib.arena[node]
                    .get()
                    .def
                    .parameter_lists()
                    .into_iter()
                    .map(|list| {
                        list.parameters
                            .iter()
                            .map(|param| match (&param.ty, &param.default_value) {
                                (None, Some(default)) => Some(default.derive_type(&ctx))
                                    .filter(|ty| *ty != Type::Any)
                                    .map(|ty| ir::Type {
                                        ty,
                                        src_ref: default.src_ref(),
                                    }),
                                _ => None,
                            })
                            .collect()
                    })
                    .collect();
                (node, types)
            })
            .collect()
    };

    for (node, types) in inferred {
        let lists = lib.arena[node].get_mut().def.parameter_lists_mut();
        for (list, types) in lists.into_iter().zip(types) {
            for (param, ty) in list.parameters.iter_mut().zip(types) {
                if ty.is_some() {
                    param.ty = ty;
                }
            }
        }
    }
}

/// Checks the types of parameters, constants and statements:
/// - a parameter needs a type or a default value, a default value must match the type,
/// - the value of a constant with type must match the type, e.g. `const X: Length = 1mm`,
/// - the expression of a statement with type must match the type, e.g. `x: Length = 1mm`.
pub(crate) struct TypeCheck<'a> {
    ctx: TypeContext<'a>,
    issues: IssueList<ResolveIssue>,
}

impl Check for TypeCheck<'_> {
    /// Check the parameters and statements of the library.
    fn check(ctx: &CheckContext) -> IssueList<ResolveIssue> {
        use visitor::Visitor;

        let mut type_check = TypeCheck {
            ctx: ctx.types(),
            issues: IssueList::default(),
        };
        type_check.visit(ctx.lib);
        type_check.issues
    }
}

/// The expected primary output of a workbench.
struct WorkbenchOutput {
    kind: WorkbenchKind,
    /// A 2D model for a sketch, a 3D model for a part.
    expected: Type,
    keyword_src_ref: SrcRef,
}

impl TypeCheck<'_> {
    /// Check if the statements without name of a sketch are 2D models, of a part 3D models.
    ///
    /// The output of an operation depends on its input.
    fn check_workbench_output(&mut self, workbench: &ir::Workbench, keyword_src_ref: SrcRef) {
        let kind = workbench.signature.kind;
        if kind == WorkbenchKind::Op {
            return;
        }
        let output = WorkbenchOutput {
            kind,
            expected: kind.ty(),
            keyword_src_ref,
        };
        self.check_output(&workbench.statements, &output);
    }

    /// Check the statements of the primary output of a workbench: the statements without name,
    /// also in groups and branches of `if`s without name.
    ///
    /// Named statements and properties, with all statements in them, are no primary output.
    fn check_output(&mut self, statements: &[ir::WorkbenchStatement], output: &WorkbenchOutput) {
        for statement in statements
            .iter()
            .filter(|statement| statement.name.is_none())
        {
            match &statement.expression {
                ir::WorkbenchExpression::Group(group) => {
                    self.check_output(&group.statements, output)
                }
                ir::WorkbenchExpression::If(if_) => self.check_output_if(if_, output),
                expression => {
                    let actual = expression.derive_type(&self.ctx);
                    if !actual.matches(&output.expected) {
                        self.issues
                            .push_err(ResolveErrorKind::WorkbenchOutputMismatch {
                                kind: output.kind.to_string(),
                                expected: output.expected.clone(),
                                actual,
                                src_ref: expression.src_ref(),
                                keyword_src_ref: output.keyword_src_ref,
                            });
                    }
                }
            }
        }
    }

    /// Check the branches of an `if` of the primary output, see [`Self::check_output`].
    fn check_output_if(&mut self, if_: &ir::workbench::WorkbenchIf, output: &WorkbenchOutput) {
        self.check_output(&if_.body.statements, output);
        if let Some(body_else) = &if_.body_else {
            self.check_output(&body_else.statements, output);
        }
        if let Some(next_if) = &if_.next_if {
            self.check_output_if(next_if, output);
        }
    }

    /// Check if the derived type of an expression matches the type `ty`.
    ///
    /// Without type, `ty` is `Any`, which matches every type.
    fn check_type(&mut self, ty: &ir::Type, expr: &(impl DeriveType + SrcReferrer)) {
        let actual = expr.derive_type(&self.ctx);
        if !actual.matches(&ty.ty) {
            self.issues.push_err(ResolveErrorKind::TypeMismatch {
                specified: ty.ty.clone(),
                specified_src_ref: ty.src_ref,
                actual,
                actual_src_ref: expr.src_ref(),
            })
        }
    }
}

impl visitor::LeafVisitor for TypeCheck<'_> {}
impl visitor::WorkbenchVisitor for TypeCheck<'_> {
    fn visit_workbench_signature(&mut self, signature: &ir::workbench::WorkbenchSignature) {
        use visitor::ConstantVisitor;

        self.visit_parameter_list(&signature.parameters);
        // The implicit initializer has the parameters of the building plan, which are checked.
        signature
            .inits
            .iter()
            .filter(|init| !init.is_implicit())
            .for_each(|init| self.visit_workbench_init(init));
    }
}
impl visitor::Visitor for TypeCheck<'_> {
    fn visit_symbol(&mut self, symbol: &Symbol) {
        if let SymbolDef::Workbench(workbench) = &symbol.def {
            self.check_workbench_output(workbench, symbol.meta.keyword_src_ref);
        }
        self.visit_meta(&symbol.meta);
        self.visit_def(&symbol.def);
    }
}

impl visitor::ConstantVisitor for TypeCheck<'_> {
    fn visit_constant(&mut self, constant: &ir::Constant) {
        self.check_type(&constant.ty, &constant.expr);
        self.visit_constant_expr(&constant.expr);
    }

    fn visit_parameter(&mut self, param: &ir::Parameter) {
        match (&param.ty, &param.default_value) {
            // `Any` must be written, e.g. `x: Any`.
            (None, None) => self
                .issues
                .push_err(ResolveErrorKind::ParameterWithoutType {
                    name: param.id.to_string(),
                    src_ref: param.id.src_ref(),
                }),
            (Some(ty), Some(default)) => self.check_type(ty, default),
            // Without known type of the default value, its type is checked when calling.
            (None, Some(_)) | (Some(_), None) => {}
        }
    }
}

impl visitor::WorkbenchExpressionVisitor for TypeCheck<'_> {
    fn visit_workbench_statement(&mut self, statement: &ir::workbench::WorkbenchStatement) {
        self.check_type(&statement.ty, &statement.expression);
        // Statements of groups, e.g. `{ x: Length = 1mm; }`.
        self.visit_workbench_expr(&statement.expression);
    }
}

impl visitor::SourceVisitor for TypeCheck<'_> {
    fn visit_source_statement(&mut self, statement: &ir::SourceStatement) {
        use visitor::WorkbenchExpressionVisitor;

        self.check_type(&statement.ty, &statement.expression);
        self.visit_workbench_expr(&statement.expression);
    }
}

impl visitor::FnVisitor for TypeCheck<'_> {
    fn visit_fn_local_assignment(
        &mut self,
        local_assignment: &ir::function::FunctionLocalAssignment,
    ) {
        self.check_type(&local_assignment.ty, &local_assignment.expression);
        // Statements of scopes, e.g. `x = { y: Length = 1mm; y };`.
        self.visit_fn_expr(&local_assignment.expression);
    }
}
