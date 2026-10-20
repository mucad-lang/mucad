// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! IR tree visitor API.

use crate::ir;

mod human_readable;
mod visitor_mut;
pub use visitor_mut::*;

pub use human_readable::MakeHumanReadable;

/// Visitor for IR leaf nodes.
pub trait LeafVisitor: Sized {
    fn visit_path(&mut self, _path: &ir::Path) {}
    /// Visit the path of a call, e.g. `f` in `f(x)` or `center` in `c.center()`.
    fn visit_callee(&mut self, path: &ir::Path) {
        self.visit_path(path)
    }
    fn visit_name(&mut self, _name: &ir::Identifier) {}
}

/// Visitor for items containing constant expressions.
pub trait ConstantVisitor: LeafVisitor {
    fn visit_constant(&mut self, constant: &ir::Constant) {
        self.visit_constant_expr(&constant.expr);
    }

    fn visit_parameter(&mut self, parameter: &ir::Parameter) {
        self.visit_name(&parameter.id);
        if let Some(default_value) = &parameter.default_value {
            self.visit_constant_expr(default_value)
        }
    }

    fn visit_parameter_list(&mut self, parameter_list: &ir::ParameterList) {
        parameter_list
            .iter()
            .for_each(|param| self.visit_parameter(param));
    }

    fn visit_constant_expr(&mut self, expr: &ir::ConstantExpression) {
        match &expr {
            ir::ConstantExpression::Invalid => {}
            ir::ConstantExpression::Value(value) => self.visit_constant_value(value),
            ir::ConstantExpression::Path(path) => self.visit_path(path),
            ir::ConstantExpression::Call(call) => self.visit_constant_call(call),
        }
    }

    fn visit_constant_call(&mut self, call: &ir::Call<ir::ConstantExpression>) {
        self.visit_callee(&call.path);
        call.args.args.iter().for_each(|arg| match arg {
            ir::Argument::Unnamed(expr) => self.visit_constant_expr(expr),
            ir::Argument::Named { name, expr, .. } => {
                self.visit_name(name);
                self.visit_constant_expr(expr);
            }
            ir::Argument::Inline { name, expr, .. } => {
                self.visit_name(name);
                self.visit_constant_expr(expr);
            }
        });
    }

    fn visit_constant_value(&mut self, _value: &ir::ConstantValue) {}
}

/// Visitor for workbench statements and expressions.
pub trait WorkbenchExpressionVisitor: ConstantVisitor {
    fn visit_workbench_statement(&mut self, statement: &ir::workbench::WorkbenchStatement) {
        self.visit_model_attributes(&statement.attr);
        if let Some(name) = &statement.name {
            self.visit_name(name);
        }
        self.visit_workbench_expr(&statement.expression);
    }

    fn visit_model_attributes(&mut self, attr: &ir::ModelAttributes) {
        if let Some(expr) = &attr.color {
            self.visit_constant_expr(expr);
        }
        if let Some(expr) = &attr.layer {
            self.visit_constant_expr(expr);
        }
        if let Some(expr) = &attr.name {
            self.visit_constant_expr(expr);
        }
        if let Some(expr) = &attr.resolution {
            self.visit_constant_expr(expr);
        }
    }

    fn visit_workbench_expr(&mut self, expr: &ir::workbench::WorkbenchExpression) {
        match &expr {
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

    fn visit_workbench_call(&mut self, call: &ir::workbench::WorkbenchCall) {
        self.visit_callee(&call.path);
        call.args.args.iter().for_each(|arg| match arg {
            ir::Argument::Unnamed(expr) => self.visit_workbench_expr(expr),
            ir::Argument::Named { name, expr, .. } => {
                self.visit_name(name);
                self.visit_workbench_expr(expr);
            }
            ir::Argument::Inline { name, expr, .. } => {
                self.visit_name(name);
                self.visit_workbench_expr(expr);
            }
        });
    }

    fn visit_workbench_if(&mut self, if_: &ir::workbench::WorkbenchIf) {
        self.visit_workbench_expr(&if_.cond);
        self.visit_workbench_group(&if_.body);
        if let Some(body_else) = &if_.body_else {
            self.visit_workbench_group(body_else);
        }
        if let Some(next_if) = &if_.next_if {
            self.visit_workbench_if(next_if);
        }
    }

    fn visit_workbench_group(&mut self, group: &ir::workbench::Group) {
        self.visit_model_attributes(&group.attr);
        group
            .statements
            .iter()
            .for_each(|statement| self.visit_workbench_statement(statement));
    }
}

/// Visitor for workbenches
pub trait WorkbenchVisitor: WorkbenchExpressionVisitor {
    fn visit_workbench(&mut self, workbench: &ir::workbench::Workbench) {
        self.visit_workbench_signature(&workbench.signature);

        workbench
            .statements
            .iter()
            .for_each(|stmt| self.visit_workbench_statement(stmt))
    }

    fn visit_workbench_signature(&mut self, signature: &ir::workbench::WorkbenchSignature) {
        self.visit_parameter_list(&signature.parameters);
        signature
            .inits
            .iter()
            .for_each(|init| self.visit_workbench_init(init));
    }

    fn visit_workbench_init(&mut self, init: &ir::workbench::Init) {
        self.visit_workbench_init_attr(&init.attr);
        self.visit_parameter_list(&init.parameters);
        init.statements.iter().for_each(|init_statement| {
            self.visit_name(&init_statement.name);
            self.visit_workbench_expr(&init_statement.expression);
        });
    }

    fn visit_workbench_init_attr(&mut self, _attr: &ir::workbench::InitAttributes) {}
}

/// Visitor for functions.
pub trait FnVisitor: ConstantVisitor {
    fn visit_fn(&mut self, function: &ir::function::Function) {
        self.visit_fn_signature(&function.signature);
        function
            .statements
            .iter()
            .for_each(|stmt| self.visit_fn_statement(stmt));
    }

    fn visit_fn_expr(&mut self, expr: &ir::function::FunctionExpression) {
        match expr {
            ir::FunctionExpression::Invalid => {}
            ir::FunctionExpression::Value(value) => self.visit_constant_value(value),
            ir::FunctionExpression::Path(path) => self.visit_path(path),
            ir::FunctionExpression::Scope(scope) => self.visit_fn_scope(scope),
            ir::FunctionExpression::If(if_) => self.visit_fn_if(if_),
            ir::FunctionExpression::Call(call) => self.visit_fn_call(call),
        }
    }

    fn visit_fn_signature(&mut self, signature: &ir::function::FunctionSignature) {
        self.visit_parameter_list(&signature.parameters);
    }

    fn visit_fn_scope(&mut self, scope: &ir::function::Scope) {
        scope
            .statements
            .iter()
            .for_each(|stmt| self.visit_fn_statement(stmt))
    }

    fn visit_fn_local_assignment(
        &mut self,
        local_assignment: &ir::function::FunctionLocalAssignment,
    ) {
        self.visit_name(&local_assignment.id);
        self.visit_fn_expr(&local_assignment.expression);
    }

    fn visit_return_statement(&mut self, return_statement: &ir::function::ReturnStatement) {
        if let Some(expr) = &return_statement.expr {
            self.visit_fn_expr(expr);
        }
    }

    fn visit_fn_statement(&mut self, stmt: &ir::function::FunctionStatement) {
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
    fn visit_fn_call(&mut self, call: &ir::function::FunctionCall) {
        self.visit_callee(&call.path);
        call.args.args.iter().for_each(|arg| match &arg {
            ir::Argument::Unnamed(expr) => self.visit_fn_expr(expr),
            ir::Argument::Named { name, expr, .. } => {
                self.visit_name(name);
                self.visit_fn_expr(expr);
            }
            ir::Argument::Inline { name, expr, .. } => {
                self.visit_name(name);
                self.visit_fn_expr(expr);
            }
        });
    }
    fn visit_fn_if(&mut self, if_: &ir::FunctionIf) {
        self.visit_fn_expr(&if_.cond);
        self.visit_fn_scope(&if_.body);
        if let Some(body_else) = &if_.body_else {
            self.visit_fn_scope(body_else);
        }
        if let Some(next_if) = &if_.next_if {
            self.visit_fn_if(next_if);
        }
    }
}

pub trait SourceVisitor: WorkbenchExpressionVisitor {
    fn visit_source(&mut self, source: &ir::Source) {
        source
            .statements
            .iter()
            .for_each(|stmt| self.visit_source_statement(stmt));
    }

    fn visit_source_statement(&mut self, statement: &ir::SourceStatement) {
        if let Some(name) = &statement.name {
            self.visit_name(name);
        }
        self.visit_model_attributes(&statement.attr);
        self.visit_export_attributes(&statement.exports);
        self.visit_workbench_expr(&statement.expression);
    }

    /// Visit the file expressions of export attributes, e.g. `#[export = "file.svg"]`.
    fn visit_export_attributes(&mut self, exports: &[ir::ExportAttribute]) {
        exports
            .iter()
            .filter_map(|export| export.file.as_ref())
            .for_each(|file| self.visit_constant_expr(file));
    }
}

/// Visitor for an IR tree.
pub trait Visitor: SourceVisitor + FnVisitor + WorkbenchVisitor + ConstantVisitor {
    fn visit_tree(&mut self, ir_tree: &ir::Tree) {
        ir_tree
            .root()
            .descendants()
            .for_each(|node| self.visit_node(node));
    }

    fn visit_node<'a>(&mut self, node: ir::NodeRef<'a>) {
        self.visit_item(node.get());
    }

    fn visit_item(&mut self, item: &ir::Item) {
        self.visit_meta(&item.meta);
        self.visit_def(&item.def);
    }

    fn visit_meta(&mut self, _meta: &ir::Meta) {}
    fn visit_inline_module(&mut self, _inline_module: &ir::InlineModule) {}
    fn visit_file_module(&mut self, _file_module: &ir::FileModule) {}
    fn visit_alias(&mut self, alias: &ir::Alias) {
        self.visit_path(&alias.path);
    }
    fn visit_wildcard(&mut self, wildcard: &ir::Wildcard) {
        self.visit_path(&wildcard.path);
    }

    fn visit_def(&mut self, def: &ir::Def) {
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
