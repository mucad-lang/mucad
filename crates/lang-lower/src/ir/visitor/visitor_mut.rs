// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! IR tree visitor API.

use crate::ir;

/// Visitor for IR leaf nodes.
pub trait LeafVisitorMut: Sized {
    fn visit_path(&mut self, _path: &mut ir::Path) {}
    fn visit_name(&mut self, _name: &mut ir::Identifier) {}
}

/// Visitor for items containing constant expressions.
pub trait ConstantVisitorMut: LeafVisitorMut {
    fn visit_constant(&mut self, constant: &mut ir::Constant) {
        self.visit_constant_expr(&mut constant.expr);
    }

    fn visit_parameter(&mut self, parameter: &mut ir::Parameter) {
        self.visit_name(&mut parameter.id);
        if let Some(default_value) = &mut parameter.default_value {
            self.visit_constant_expr(default_value)
        }
    }

    fn visit_parameter_list(&mut self, parameter_list: &mut ir::ParameterList) {
        parameter_list
            .iter_mut()
            .for_each(|param| self.visit_parameter(param));
    }

    fn visit_constant_expr(&mut self, expr: &mut ir::ConstantExpression) {
        match expr {
            ir::ConstantExpression::Invalid => {}
            ir::ConstantExpression::Value(value) => self.visit_constant_value(value),
            ir::ConstantExpression::Path(path) => self.visit_path(path),
            ir::ConstantExpression::Call(call) => self.visit_constant_call(call),
        }
    }

    fn visit_constant_call(&mut self, call: &mut ir::Call<ir::ConstantExpression>) {
        self.visit_path(&mut call.path);
        call.args.args.iter_mut().for_each(|arg| match arg {
            ir::Argument::Unnamed(expr) => self.visit_constant_expr(expr),
            ir::Argument::Named { name, expr, .. } => {
                self.visit_name(name);
                self.visit_constant_expr(expr);
            }
        });
    }

    fn visit_constant_value(&mut self, _value: &ir::ConstantValue) {}
}

/// Visitor for workbench statements and expressions.
pub trait WorkbenchExpressionVisitorMut: ConstantVisitorMut {
    fn visit_workbench_statement(&mut self, statement: &mut ir::workbench::WorkbenchStatement) {
        self.visit_model_attributes(&mut statement.attr);
        if let Some(name) = &mut statement.name {
            self.visit_name(name);
        }
        self.visit_workbench_expr(&mut statement.expression);
    }

    fn visit_model_attributes(&mut self, attr: &mut ir::ModelAttributes) {
        if let Some(expr) = &mut attr.color {
            self.visit_constant_expr(expr);
        }
        if let Some(expr) = &mut attr.layer {
            self.visit_constant_expr(expr);
        }
        if let Some(expr) = &mut attr.name {
            self.visit_constant_expr(expr);
        }
        if let Some(expr) = &mut attr.resolution {
            self.visit_constant_expr(expr);
        }
    }

    fn visit_workbench_expr(&mut self, expr: &mut ir::workbench::WorkbenchExpression) {
        match expr {
            ir::WorkbenchExpression::Invalid => {}
            ir::WorkbenchExpression::Value(constant_value) => {
                self.visit_constant_value(constant_value)
            }
            ir::WorkbenchExpression::Path(path) => self.visit_path(path),
            ir::WorkbenchExpression::Group(group) => self.visit_workbench_group(group),
            ir::WorkbenchExpression::If(if_) => self.visit_workbench_if(if_),
            ir::WorkbenchExpression::Call(call) => self.visit_workbench_call(call),
            ir::WorkbenchExpression::Marker(_) => {} // TODO implement marker
        }
    }

    fn visit_workbench_call(&mut self, call: &mut ir::workbench::WorkbenchCall) {
        self.visit_path(&mut call.path);
        call.args.args.iter_mut().for_each(|arg| match arg {
            ir::Argument::Unnamed(expr) => self.visit_workbench_expr(expr),
            ir::Argument::Named { name, expr, .. } => {
                self.visit_name(name);
                self.visit_workbench_expr(expr);
            }
        });
    }

    fn visit_workbench_if(&mut self, if_: &mut ir::workbench::WorkbenchIf) {
        self.visit_workbench_expr(&mut if_.cond);
        self.visit_workbench_group(&mut if_.body);
        if let Some(body_else) = &mut if_.body_else {
            self.visit_workbench_group(body_else);
        }
        if let Some(next_if) = &mut if_.next_if {
            self.visit_workbench_if(next_if);
        }
    }

    fn visit_workbench_group(&mut self, group: &mut ir::workbench::Group) {
        self.visit_model_attributes(&mut group.attr);
        group
            .statements
            .iter_mut()
            .for_each(|statement| self.visit_workbench_statement(statement));
    }
}

/// Visitor for workbenches
pub trait WorkbenchVisitorMut: WorkbenchExpressionVisitorMut {
    fn visit_workbench(&mut self, workbench: &mut ir::workbench::Workbench) {
        self.visit_workbench_signature(&mut workbench.signature);

        workbench
            .statements
            .iter_mut()
            .for_each(|stmt| self.visit_workbench_statement(stmt))
    }

    fn visit_workbench_signature(&mut self, signature: &mut ir::workbench::WorkbenchSignature) {
        self.visit_parameter_list(&mut signature.parameters);
        signature
            .inits
            .iter_mut()
            .for_each(|init| self.visit_workbench_init(init));
    }

    fn visit_workbench_init(&mut self, init: &mut ir::workbench::Init) {
        self.visit_workbench_init_attr(&mut init.attr);
        self.visit_parameter_list(&mut init.parameters);
        init.statements.iter_mut().for_each(|init_statement| {
            self.visit_name(&mut init_statement.name);
            self.visit_workbench_expr(&mut init_statement.expression);
        });
    }

    fn visit_workbench_init_attr(&mut self, _attr: &mut ir::workbench::InitAttributes) {}
}

/// Visitor for functions.
pub trait FnVisitorMut: ConstantVisitorMut {
    fn visit_fn(&mut self, function: &mut ir::function::Function) {
        self.visit_fn_signature(&mut function.signature);
        function
            .statements
            .iter_mut()
            .for_each(|stmt| self.visit_fn_statement(stmt));
    }

    fn visit_fn_expr(&mut self, expr: &mut ir::function::FunctionExpression) {
        match expr {
            ir::FunctionExpression::Invalid => {}
            ir::FunctionExpression::Value(value) => self.visit_constant_value(value),
            ir::FunctionExpression::Path(path) => self.visit_path(path),
            ir::FunctionExpression::Scope(scope) => self.visit_fn_scope(scope),
            ir::FunctionExpression::If(if_) => self.visit_fn_if(if_),
            ir::FunctionExpression::Call(call) => self.visit_fn_call(call),
        }
    }

    fn visit_fn_signature(&mut self, signature: &mut ir::function::FunctionSignature) {
        self.visit_parameter_list(&mut signature.parameters);
    }

    fn visit_fn_scope(&mut self, scope: &mut ir::function::Scope) {
        scope
            .statements
            .iter_mut()
            .for_each(|stmt| self.visit_fn_statement(stmt))
    }

    fn visit_fn_local_assignment(
        &mut self,
        local_assignment: &mut ir::function::FunctionLocalAssignment,
    ) {
        self.visit_name(&mut local_assignment.id);
        self.visit_fn_expr(&mut local_assignment.expression);
    }

    fn visit_return_statement(&mut self, return_statement: &mut ir::function::ReturnStatement) {
        if let Some(expr) = &mut return_statement.expr {
            self.visit_fn_expr(expr);
        }
    }

    fn visit_fn_statement(&mut self, stmt: &mut ir::function::FunctionStatement) {
        match stmt {
            ir::FunctionStatement::Local(local_assignment) => {
                self.visit_fn_local_assignment(local_assignment)
            }
            ir::FunctionStatement::Scope(scope) => self.visit_fn_scope(scope),
            ir::FunctionStatement::Call(call) => self.visit_fn_call(call),
            ir::FunctionStatement::If(if_) => self.visit_fn_if(if_),
            ir::FunctionStatement::Tail(expr) => self.visit_fn_expr(expr),
            ir::FunctionStatement::Return(return_statement) => {
                self.visit_return_statement(return_statement)
            }
        }
    }
    fn visit_fn_call(&mut self, call: &mut ir::function::FunctionCall) {
        self.visit_path(&mut call.path);
        call.args.args.iter_mut().for_each(|arg| match arg {
            ir::Argument::Unnamed(expr) => self.visit_fn_expr(expr),
            ir::Argument::Named { name, expr, .. } => {
                self.visit_name(name);
                self.visit_fn_expr(expr);
            }
        });
    }
    fn visit_fn_if(&mut self, if_: &mut ir::FunctionIf) {
        self.visit_fn_expr(&mut if_.cond);
        self.visit_fn_scope(&mut if_.body);
        if let Some(body_else) = &mut if_.body_else {
            self.visit_fn_scope(body_else);
        }
        if let Some(next_if) = &mut if_.next_if {
            self.visit_fn_if(next_if);
        }
    }
}

pub trait SourceVisitorMut: ConstantVisitorMut + WorkbenchVisitorMut {
    fn visit_source(&mut self, source: &mut ir::Source) {
        source
            .statements
            .iter_mut()
            .for_each(|stmt| self.visit_source_statement(stmt));
    }

    fn visit_source_statement(&mut self, statement: &mut ir::SourceStatement) {
        if let Some(name) = &mut statement.name {
            self.visit_name(name);
        }
        self.visit_model_attributes(&mut statement.attr);
        self.visit_workbench_expr(&mut statement.expression);
    }
}

/// Visitor for an IR tree.
pub trait VisitorMut:
    SourceVisitorMut + FnVisitorMut + WorkbenchVisitorMut + ConstantVisitorMut
{
    fn visit(&mut self, ir: &mut crate::Ir) {
        self.visit_tree(&mut ir.tree);
    }

    fn visit_tree(&mut self, ir_tree: &mut ir::Tree) {
        ir_tree.root_mut().transform(|node, item| {
            self.visit_item(node, item);
        })
    }

    fn visit_item<'a>(&mut self, _node: ir::NodeRef<'a>, item: &mut ir::Item) {
        self.visit_meta(&mut item.meta);
        self.visit_def(&mut item.def);
    }

    fn visit_meta(&mut self, _meta: &mut ir::Meta) {}
    fn visit_inline_module(&mut self, _inline_module: &mut ir::InlineModule) {}
    fn visit_file_module(&mut self, _file_module: &mut ir::FileModule) {}
    fn visit_alias(&mut self, alias: &mut ir::Alias) {
        self.visit_path(&mut alias.path);
    }
    fn visit_wildcard(&mut self, wildcard: &mut ir::Wildcard) {
        self.visit_path(&mut wildcard.path);
    }

    fn visit_def(&mut self, def: &mut ir::Def) {
        match def {
            ir::Def::Source(source) => {
                self.visit_source(source);
            }
            ir::Def::InlineModule(inline_module) => self.visit_inline_module(inline_module),
            ir::Def::FileModule(file_module) => self.visit_file_module(file_module),
            ir::Def::Workbench(workbench) => self.visit_workbench(workbench),
            ir::Def::Function(function) => self.visit_fn(function),
            ir::Def::Constant(constant) => self.visit_constant(constant),
            ir::Def::Alias(alias) => self.visit_alias(alias),
            ir::Def::Wildcard(wildcard) => self.visit_wildcard(wildcard),
        }
    }
}
