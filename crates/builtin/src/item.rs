// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! µcad built-in items.

use derive_more::{Debug, From};

use microcad_lang_base::{BuiltinId, BuiltinInfo};
use microcad_lang_types::{Arguments, FunctionType, Model, ModelTree, Tuple, Value};

use crate::{BuiltinEvalContext, BuiltinEvalFn, BuiltinFn, BuiltinResult};

#[derive(Debug, Clone)]
#[debug("{}", info)]
pub struct BuiltinConstant {
    pub info: BuiltinInfo,
    pub f: BuiltinFn<Value>,
}

impl BuiltinConstant {
    pub const fn new(info: BuiltinInfo, f: BuiltinFn<Value>) -> Self {
        Self { info, f }
    }

    pub fn value(&self) -> Value {
        (self.f)()
    }
}

/// A built-in function.
#[derive(Debug, Clone)]
#[debug("{}", info)]
pub struct BuiltinFunction {
    pub info: BuiltinInfo,
    pub ty: BuiltinFn<FunctionType>,
    /// Default values of parameters.
    pub defaults: BuiltinFn<Tuple>,
    pub f: BuiltinEvalFn,
}

impl BuiltinFunction {
    /// Construct a new BuiltinFunction
    pub const fn new(info: BuiltinInfo, ty: BuiltinFn<FunctionType>, f: BuiltinEvalFn) -> Self {
        Self {
            info,
            ty,
            defaults: Tuple::default,
            f,
        }
    }

    /// Default values of parameters, e.g. `|| tuple!(message = "Assertion failed")`.
    pub const fn with_defaults(mut self, defaults: BuiltinFn<Tuple>) -> Self {
        self.defaults = defaults;
        self
    }

    /// Get the default values of parameters.
    pub fn default_values(&self) -> Tuple {
        (self.defaults)()
    }

    /// Check if function is impure.
    pub fn is_impure(&self) -> bool {
        self.ty().is_impure
    }

    /// Get the function type
    pub fn ty(&self) -> FunctionType {
        (self.ty)()
    }

    /// Call the function with matched arguments.
    pub fn call(&self, args: Arguments, ctx: &mut BuiltinEvalContext) -> BuiltinResult<Value> {
        (self.f)(args, ctx)
    }
}

/// A primitive is a function that can produce a single model node.
#[derive(Debug, Clone)]
#[debug("{}", info)]
pub struct BuiltinPrimitive {
    pub info: BuiltinInfo,
    pub ty: BuiltinFn<FunctionType>,
    pub f: BuiltinEvalFn<Model>,
}

impl BuiltinPrimitive {
    pub const fn new(
        info: BuiltinInfo,
        ty: BuiltinFn<FunctionType>,
        f: BuiltinEvalFn<Model>,
    ) -> Self {
        Self { info, ty, f }
    }

    /// Get the function type
    pub fn ty(&self) -> FunctionType {
        (self.ty)()
    }
}

#[derive(Debug, Clone)]
#[debug("{}", info)]
pub struct BuiltinOperation {
    pub info: BuiltinInfo,
    pub ty: BuiltinFn<FunctionType>,
    pub f: BuiltinEvalFn<ModelTree>,
}

impl BuiltinOperation {
    pub const fn new(
        info: BuiltinInfo,
        ty: BuiltinFn<FunctionType>,
        f: BuiltinEvalFn<ModelTree>,
    ) -> Self {
        Self { info, ty, f }
    }
}

#[derive(Debug, Clone)]
pub struct BuiltinModule {
    pub info: BuiltinInfo,
    pub items: &'static [&'static BuiltinItem],
}

/// Builder methods
impl BuiltinModule {
    pub const fn new(info: BuiltinInfo, items: &'static [&'static BuiltinItem]) -> Self {
        Self { info, items }
    }
}

/// Item methods
impl BuiltinModule {
    pub fn items(&self) -> impl Iterator<Item = &'static BuiltinItem> {
        self.items.iter().copied()
    }

    pub fn constants(&self) -> impl Iterator<Item = &'static BuiltinConstant> {
        self.items().filter_map(|item| match item {
            BuiltinItem::Constant(builtin_constant) => Some(builtin_constant),
            _ => None,
        })
    }

    pub fn functions(&self) -> impl Iterator<Item = &'static BuiltinFunction> {
        self.items().filter_map(|item| match item {
            BuiltinItem::Function(f) => Some(f),
            _ => None,
        })
    }

    pub fn operations(&self) -> impl Iterator<Item = &'static BuiltinOperation> {
        self.items().filter_map(|item| match item {
            BuiltinItem::Operation(op) => Some(op),
            _ => None,
        })
    }

    pub fn primitives(&self) -> impl Iterator<Item = &'static BuiltinPrimitive> {
        self.items().filter_map(|item| match item {
            BuiltinItem::Primitive(op) => Some(op),
            _ => None,
        })
    }
}

#[derive(Debug, Clone, From)]
pub enum BuiltinItem {
    /// Produces a constant Value
    Constant(BuiltinConstant),
    /// A function computing a Value
    Function(BuiltinFunction),
    /// A function computing a single model node
    Primitive(BuiltinPrimitive),
    /// A function computing a model tree from an existing one
    Operation(BuiltinOperation),
    /// A builtin module
    Module(BuiltinModule),
}

impl BuiltinItem {
    pub const fn info(&self) -> &BuiltinInfo {
        match self {
            BuiltinItem::Constant(c) => &c.info,
            BuiltinItem::Function(f) => &f.info,
            BuiltinItem::Primitive(p) => &p.info,
            BuiltinItem::Operation(o) => &o.info,
            BuiltinItem::Module(o) => &o.info,
        }
    }

    pub const fn id(&self) -> BuiltinId {
        self.info().id()
    }

    pub const fn name(&self) -> &'static str {
        self.info().name
    }

    pub fn value(&self) -> Value {
        match self {
            BuiltinItem::Constant(builtin_constant) => builtin_constant.value(),
            _ => Value::default(),
        }
    }

    pub fn call_fn(&self, args: Arguments, ctx: &mut BuiltinEvalContext) -> BuiltinResult<Value> {
        match self {
            BuiltinItem::Function(f) => (f.f)(args, ctx),
            _ => unreachable!("Only functions can be called."),
        }
    }

    pub fn call_primitive(
        &self,
        args: Arguments,
        ctx: &mut BuiltinEvalContext,
    ) -> BuiltinResult<Model> {
        match self {
            BuiltinItem::Primitive(p) => (p.f)(args, ctx),
            _ => unreachable!("Only functions can be called."),
        }
    }

    pub fn call_op(
        &self,
        args: Arguments,
        ctx: &mut BuiltinEvalContext,
    ) -> BuiltinResult<ModelTree> {
        match self {
            BuiltinItem::Operation(o) => (o.f)(args, ctx),
            _ => unreachable!("Only functions can be called."),
        }
    }
}

/// A macro to declare built-in items.
#[macro_export]
macro_rules! builtin_item {
    // Syntax: builtin_item!(
    //     Module
    //     "Doc string"
    //     mod_name [ item1, item2, ... ]
    // )
    (
        Module
        $doc:literal
        $mod_name:ident [ $($builtin:expr),* $(,)? ]
    ) => {
        $crate::BuiltinItem::Module($crate::BuiltinModule::new(
            $crate::builtin_info!($doc $mod_name),
            &[ $($builtin),* ],
        ))
    };

    // Syntax: builtin_item!(
    //      Function
    //      "Doc string"
    //      mod::func(param1: type1, param2: type2, ...) -> return_type
    //      defaults(param2 = value2)
    // )
    (
        Function
        $doc:literal
        $mod_name:ident::$fn_name:ident ( $func_ty:expr ) defaults ( $( $param:ident = $value:expr ),* )
    ) => {
        $crate::BuiltinItem::Function(
            $crate::BuiltinFunction::new(
                $crate::builtin_info!($doc $mod_name::$fn_name),
                || $func_ty,
                $fn_name,
            )
            .with_defaults(|| microcad_lang_types::tuple!($( $param = $value ),*)),
        )
    };
    // Syntax: builtin_item!(
    //      Function
    //      "Doc string"
    //      mod::func(param1: type1, param2: type2, ...) -> return_type
    // )
    (
        Function
        $doc:literal
        $mod_name:ident::$fn_name:ident ( $func_ty:expr )
    ) => {
        $crate::BuiltinItem::Function($crate::BuiltinFunction::new(
            $crate::builtin_info!($doc $mod_name::$fn_name),
            || $func_ty,
            $fn_name,
        ))
    };
    // Syntax: builtin_item!(Constant "A constant" math::PI = std::f64::consts::PI)
    (
        Constant
        $doc:literal
        $mod_name:ident::$fn_name:ident = $value:expr
    ) => {
        $crate::BuiltinItem::Constant($crate::BuiltinConstant::new(
            $crate::builtin_info!($doc $mod_name::$fn_name),
            || $crate::Value::from($value),
        ))
    };

    // Syntax: builtin_item!(Operation "An operation" ops::op(param1: type1, ...) -> return_type))
    (
        Operation
        $doc:literal
        $mod_name:ident::$fn_name:ident ( $func_ty:expr )
    ) => {
        $crate::BuiltinItem::Operation($crate::BuiltinOperation::new(
            $crate::builtin_info!($doc $mod_name::$fn_name),
            || $func_ty,
            $fn_name,
        ))
    };

    // Syntax: builtin_item!(Operation "An operation" ops::op(param1: type1, ...) -> return_type))
    (
        Primitive
        $doc:literal
        $mod_name:ident::$struct_name:ident ( $func_ty:expr )
    ) => {
        $crate::BuiltinItem::Primitive($crate::BuiltinPrimitive::new(
            $crate::builtin_info!($doc $mod_name::$struct_name),
            || $func_ty,
            $struct_name::call,
        ))
    };
}
