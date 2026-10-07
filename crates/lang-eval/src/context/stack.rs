// Copyright © 2024-2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

use derive_more::From;
use microcad_builtin::BuiltinItem;
use microcad_lang_base::{HashMap, Name, SrcRef, SrcReferrer, ToCompactString};
use microcad_lang_resolve::library::symbol;
use microcad_lang_types::{
    Arguments, Value,
    model::{
        Element, GetProperty, ModelTreeBuilder, ModelTreeBuilderMut, Properties, Property,
        Workpiece,
    },
};

pub trait ContextScope {
    /// Local a local or property value by traversing up the stack
    fn look_up_local(&self, _name: impl AsRef<str>) -> Option<&Value> {
        None
    }

    /// Current symbol name
    fn current_symbol_name(&self) -> Option<String> {
        None
    }

    /// Current source referrer
    fn current_symbol_src_ref(&self) -> SrcRef {
        SrcRef::none()
    }
}

/// A map of locals.
#[derive(Debug, Default)]
pub struct LocalTable(HashMap<Name, Value>);

impl ContextScope for LocalTable {
    fn look_up_local(&self, name: impl AsRef<str>) -> Option<&Value> {
        self.0.get(&Name::from(name.as_ref()))
    }
}

#[derive(Debug, Default)]
pub struct FunctionFrame {
    pub locals: LocalTable,
}

impl ContextScope for FunctionFrame {
    fn look_up_local(&self, name: impl AsRef<str>) -> Option<&Value> {
        self.locals.look_up_local(name)
    }
}

impl FunctionFrame {
    pub fn new(args: Arguments) -> Self {
        let locals = LocalTable(
            args.named_iter()
                .map(|(id, value)| (id.to_compact_string(), value.clone()))
                .collect(),
        );

        Self { locals }
    }
}

#[derive(Debug, Default)]
pub struct FunctionScopeFrame {
    //symbol: mir::SymbolHandle,
    pub locals: LocalTable,
}

impl FunctionScopeFrame {
    pub fn new() -> Self {
        Self::default()
    }
}

impl ContextScope for FunctionScopeFrame {
    fn look_up_local(&self, name: impl AsRef<str>) -> Option<&Value> {
        self.locals.look_up_local(name)
    }
}

#[derive(Debug)]
pub struct WorkpieceFrame {
    pub builder: ModelTreeBuilder,
    /// Input model of an operation, available as `self` (currently `@input`).
    pub input: Option<Value>,
}

impl WorkpieceFrame {
    pub fn new(workpiece: Workpiece) -> Self {
        Self {
            builder: ModelTreeBuilder::new(Element::Workpiece(workpiece)),
            input: None,
        }
    }

    /// Set the input model of an operation.
    pub fn with_input(mut self, input: Option<Value>) -> Self {
        self.input = input;
        self
    }
}

impl ModelTreeBuilderMut for WorkpieceFrame {
    fn model_tree_builder_mut(&mut self) -> &mut ModelTreeBuilder {
        &mut self.builder
    }
}

impl ContextScope for WorkpieceFrame {
    fn look_up_local(&self, name: impl AsRef<str>) -> Option<&Value> {
        match name.as_ref() {
            "self" => self.input.as_ref(),
            name => self.builder.get_property(name).map(|prop| &prop.value),
        }
    }
}

#[derive(Debug)]
pub struct WorkbenchGroupFrame {
    pub builder: ModelTreeBuilder,
}

impl WorkbenchGroupFrame {
    pub fn new() -> Self {
        Self {
            builder: ModelTreeBuilder::new(Element::Group),
        }
    }
}

impl ContextScope for WorkbenchGroupFrame {
    fn look_up_local(&self, name: impl AsRef<str>) -> Option<&Value> {
        self.builder.get_property(name).map(|prop| &prop.value)
    }
}

impl ModelTreeBuilderMut for WorkbenchGroupFrame {
    fn model_tree_builder_mut(&mut self) -> &mut ModelTreeBuilder {
        &mut self.builder
    }
}

#[derive(Debug, Default)]
pub struct WorkbenchInitFrame {
    pub properties: Properties,
}

impl WorkbenchInitFrame {
    pub fn new(arguments: Arguments) -> WorkbenchInitFrame {
        Self {
            properties: Properties::from_iter(
                arguments
                    .named_iter()
                    .map(|(id, value)| Property::hidden(id, value.clone())),
            ),
        }
    }
}

impl ContextScope for WorkbenchInitFrame {
    fn look_up_local(&self, name: impl AsRef<str>) -> Option<&Value> {
        self.properties.get_property(name).map(|prop| &prop.value)
    }
}

#[derive(Debug)]
pub struct SymbolCallFrame {
    pub path: symbol::Path,
    pub src_ref: SrcRef,
}

impl ContextScope for SymbolCallFrame {
    fn look_up_local(&self, _name: impl AsRef<str>) -> Option<&Value> {
        None
    }

    fn current_symbol_name(&self) -> Option<String> {
        Some(self.path.to_string())
    }

    fn current_symbol_src_ref(&self) -> SrcRef {
        self.src_ref
    }
}

#[derive(Debug)]
pub struct SourceFrame {
    pub builder: ModelTreeBuilder,
}

impl SourceFrame {
    pub fn new() -> Self {
        Self {
            builder: ModelTreeBuilder::new(Element::Group),
        }
    }
}

impl ContextScope for SourceFrame {
    fn look_up_local(&self, name: impl AsRef<str>) -> Option<&Value> {
        self.builder.get_property(name).map(|prop| &prop.value)
    }
}

impl ModelTreeBuilderMut for SourceFrame {
    fn model_tree_builder_mut(&mut self) -> &mut ModelTreeBuilder {
        &mut self.builder
    }
}

#[derive(Debug)]
pub struct BuiltinItemFrame {
    src_ref: SrcRef,
    item: &'static BuiltinItem,
}

impl BuiltinItemFrame {
    pub fn new(src_ref: impl SrcReferrer, item: &'static BuiltinItem) -> Self {
        Self {
            src_ref: src_ref.src_ref(),
            item,
        }
    }
}

impl ContextScope for BuiltinItemFrame {
    fn current_symbol_name(&self) -> Option<String> {
        Some(self.item.name().to_string())
    }

    fn current_symbol_src_ref(&self) -> SrcRef {
        self.src_ref
    }
}

#[derive(Debug, From)]
pub enum StackFrame {
    /// Active frame for any calls.
    Call(SymbolCallFrame),

    /// Active frame inside a function scope.
    Function(FunctionFrame),

    /// Active frame for local function scope bindings.
    FunctionScope(FunctionScopeFrame),

    /// Active frame when evaluating a workbench or workpiece execution context.
    Workbench(WorkpieceFrame),

    /// Active frame when evaluating statements within a workbench group block.
    WorkbenchGroup(WorkbenchGroupFrame),

    /// Active frame when evaluating inside a workbench `init` block.
    WorkbenchInit(WorkbenchInitFrame),

    /// Active frame when evaluating top-level statements in a source file.
    Source(SourceFrame),

    /// Active frame during built-in execution.
    BuiltinCall(BuiltinItemFrame),
}

impl StackFrame {
    pub fn get_local(&self, name: &Name) -> Option<&Value> {
        match self {
            StackFrame::Function(FunctionFrame { locals })
            | StackFrame::FunctionScope(FunctionScopeFrame { locals }) => locals.0.get(name),
            _ => None,
        }
    }

    pub fn put_local(&mut self, name: Name, value: Value) {
        match self {
            StackFrame::Function(FunctionFrame { locals })
            | StackFrame::FunctionScope(FunctionScopeFrame { locals }) => {
                locals.0.insert(name, value);
            }
            _ => {}
        }
    }
}

impl ModelTreeBuilderMut for StackFrame {
    fn model_tree_builder_mut(&mut self) -> &mut ModelTreeBuilder {
        match self {
            StackFrame::Workbench(workbench_frame) => workbench_frame.model_tree_builder_mut(),
            StackFrame::WorkbenchGroup(workbench_group_frame) => {
                workbench_group_frame.model_tree_builder_mut()
            }
            StackFrame::Source(source) => source.model_tree_builder_mut(),
            _ => panic!("No model tree builder"),
        }
    }
}

impl ContextScope for StackFrame {
    fn look_up_local(&self, name: impl AsRef<str>) -> Option<&Value> {
        match &self {
            StackFrame::Call(_) | StackFrame::BuiltinCall(_) => None,
            StackFrame::Function(function_frame) => function_frame.look_up_local(name),
            StackFrame::FunctionScope(function_scope_frame) => {
                function_scope_frame.look_up_local(name)
            }
            StackFrame::Workbench(workbench_frame) => workbench_frame.look_up_local(name),
            StackFrame::WorkbenchGroup(workbench_group_frame) => {
                workbench_group_frame.look_up_local(name)
            }
            StackFrame::WorkbenchInit(init) => init.look_up_local(name),
            StackFrame::Source(source) => source.look_up_local(name),
        }
    }

    fn current_symbol_name(&self) -> Option<String> {
        match &self {
            StackFrame::Call(call) => call.current_symbol_name(),
            StackFrame::BuiltinCall(call) => call.current_symbol_name(),
            _ => None,
        }
    }

    fn current_symbol_src_ref(&self) -> SrcRef {
        match &self {
            StackFrame::Call(call) => call.current_symbol_src_ref(),
            StackFrame::BuiltinCall(call) => call.current_symbol_src_ref(),
            _ => SrcRef::none(),
        }
    }
}

/// A generic stack.
#[derive(Debug, Default)]
pub struct Stack(Vec<StackFrame>);

impl Stack {
    pub fn current_call_scope(&self) -> impl Iterator<Item = &StackFrame> {
        self.0.iter().rev().scan(false, |hit_call, frame| {
            if *hit_call {
                return None;
            }
            if matches!(frame, StackFrame::Call(_)) {
                *hit_call = true;
            }
            Some(frame)
        })
    }
}

impl ModelTreeBuilderMut for Stack {
    fn model_tree_builder_mut(&mut self) -> &mut ModelTreeBuilder {
        self.top_mut().model_tree_builder_mut()
    }
}

impl ContextScope for Stack {
    fn look_up_local(&self, name: impl AsRef<str>) -> Option<&Value> {
        let name = name.as_ref();
        self.current_call_scope()
            .find_map(|frame| frame.look_up_local(name))
    }

    fn current_symbol_name(&self) -> Option<String> {
        self.current_call_scope()
            .find_map(|frame| frame.current_symbol_name())
    }

    fn current_symbol_src_ref(&self) -> SrcRef {
        self.current_call_scope()
            .find(|frame| frame.current_symbol_src_ref() != SrcRef::none())
            .map(|frame| frame.current_symbol_src_ref())
            .unwrap_or_default()
    }
}

impl StackRead for Stack {
    type Frame = StackFrame;

    fn get_local(&self, name: &Name) -> Option<&Value> {
        self.current_call_scope()
            .find_map(|frame| frame.get_local(name))
    }

    fn top(&self) -> &StackFrame {
        self.0.last().expect("A stack frame") // Intentionally no error handling here
    }
}

impl StackWrite for Stack {
    fn push(&mut self, frame: impl Into<StackFrame>) {
        self.0.push(frame.into());
    }

    fn pop(&mut self) -> StackFrame {
        self.0.pop().expect("A stack frame")
    }

    fn top_mut(&mut self) -> &mut StackFrame {
        self.0.last_mut().expect("A stack frame")
    }
}

pub trait StackRead {
    type Frame;

    fn get_local(&self, _name: &Name) -> Option<&Value> {
        None
    }

    fn top(&self) -> &Self::Frame;
}

pub trait StackWrite: StackRead {
    fn top_mut(&mut self) -> &mut Self::Frame;
    fn pop(&mut self) -> Self::Frame {
        unimplemented!("Implement stack pop")
    }
    fn push(&mut self, _: impl Into<Self::Frame>) {
        unimplemented!("Implement stack push")
    }
}
