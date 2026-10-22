// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Types of expressions before evaluation.

use mucad_builtin::BuiltinLibrary;
use mucad_lang_base::Source;
use mucad_lang_lower::ir;
use mucad_lang_resolve::{
    DeriveType, Library, ResolveContext, SymbolDef, SymbolNodeExt, TypeContext,
    library::symbol::SourceFile,
};
use mucad_lang_types::{ModelType, QuantityType, Type};

/// A source with constants, a function, workbenches and statements of each.
const CODE: &str = r#"
use globlib::geo::*;

const A = 1mm;
const B = A;
const EXTERNAL = R;

fn area(width: Length, height: Length) -> Area {
    width * height
}

fn tail(x: Length) {
    if x > 1mm { 2mm } else { 3mm }
}

fn ret() {
    return B;
}

sketch S() {}
part P() {}

s = S();
a = area(width = 1mm, height = 2mm);
p = P();
gt = __mu::core::gt(lhs = 1, rhs = 2);
pi = __mu::math::PI;
circle = __mu::geo2d::Circle(radius = 1mm);
shape = Shape();
local = s;
"#;

/// The resolved library of [`CODE`] and its resolve context.
fn resolve() -> (Library, ResolveContext) {
    let mut ctx = ResolveContext::new();
    let (lib_id, issues) = ctx
        .load_external_library("tests/test_cases/glob_lib")
        .expect("A library");
    assert!(!issues.has_errors(), "{issues:?}");
    ctx.add_implicit_dependency("globlib", lib_id);

    let (lib, issues) =
        mucad_lang_resolve::resolve_source(Source::from(CODE), &mut ctx).expect("A library");
    assert!(!issues.has_errors(), "{issues:?}");
    (lib, ctx)
}

/// The definition of the symbol `name`.
fn def<'a>(lib: &'a Library, name: &str) -> &'a SymbolDef {
    lib.root()
        .descendants()
        .find(|node| node.name().is_some_and(|n| n.as_str() == name))
        .map(|node| &node.get().def)
        .unwrap_or_else(|| panic!("A symbol {name}"))
}

/// The expression of the source statement `name`.
fn statement<'a>(lib: &'a Library, name: &str) -> &'a ir::WorkbenchExpression {
    lib.root()
        .descendants()
        .find_map(|node| match &node.get().def {
            SymbolDef::SourceFile(SourceFile::Loaded { source, .. }) => source
                .statements
                .iter()
                .find(|statement| statement.name.as_ref().is_some_and(|n| n.as_str() == name)),
            _ => None,
        })
        .map(|statement| &statement.expression)
        .unwrap_or_else(|| panic!("A statement {name}"))
}

/// The expression of the constant `name`.
fn constant<'a>(lib: &'a Library, name: &str) -> &'a ir::ConstantExpression {
    match def(lib, name) {
        SymbolDef::Constant(constant) => &constant.expr,
        _ => panic!("A constant {name}"),
    }
}

fn length() -> Type {
    Type::Quantity(QuantityType::Length)
}

/// Constants have the type of their value, also of other constants and libraries.
#[test]
fn constant_types() {
    let (lib, ctx) = resolve();
    let builtins = BuiltinLibrary::new();
    let lib_cache = ctx.lib_cache();
    let lib_cache = lib_cache.read_unwrap();
    let types = TypeContext::new(&lib, &builtins).with_lib_cache(&lib_cache);

    assert_eq!(constant(&lib, "A").derive_type(&types), length());
    assert_eq!(constant(&lib, "B").derive_type(&types), length());
    assert_eq!(constant(&lib, "EXTERNAL").derive_type(&types), length());

    // Without the other libraries, their symbols have no known type.
    let types = TypeContext::new(&lib, &builtins);
    assert_eq!(constant(&lib, "EXTERNAL").derive_type(&types), Type::Any);
}

/// Calls of workbenches are models, of functions and built-ins their return type.
#[test]
fn workbench_expression_types() {
    let (lib, ctx) = resolve();
    let builtins = BuiltinLibrary::new();
    let lib_cache = ctx.lib_cache();
    let lib_cache = lib_cache.read_unwrap();
    let types = TypeContext::new(&lib, &builtins).with_lib_cache(&lib_cache);
    let ty = |name: &str| statement(&lib, name).derive_type(&types);

    assert_eq!(ty("s"), Type::Model(ModelType::Geometry2D));
    assert_eq!(ty("p"), Type::Model(ModelType::Geometry3D));
    // The return type of a function.
    assert_eq!(ty("a"), Type::Quantity(QuantityType::Area));
    assert_eq!(ty("gt"), Type::Bool);
    assert_eq!(ty("pi"), Type::scalar());
    assert_eq!(ty("circle"), Type::Model(ModelType::Geometry2D));
    // A sketch of another library.
    assert_eq!(ty("shape"), Type::Model(ModelType::Geometry2D));
    // Locals are known when evaluating.
    assert_eq!(ty("local"), Type::Any);
}

/// The value of a function has the type of its last statement, e.g. of both branches of an `if`.
#[test]
fn function_expression_types() {
    let (lib, _) = resolve();
    let builtins = BuiltinLibrary::new();
    let types = TypeContext::new(&lib, &builtins);

    // The type of the value of a function: its last statement.
    let ty = |name: &str| match def(&lib, name) {
        SymbolDef::Function(function) => function
            .statements
            .last()
            .expect("A statement")
            .derive_type(&types),
        _ => panic!("A function {name}"),
    };

    // Both branches of `if x > 1mm { 2mm } else { 3mm }` are lengths.
    assert_eq!(ty("tail"), length());
    // A calculation with a built-in of unknown type, e.g. `width * height`.
    assert_eq!(ty("area"), Type::Any);
    assert_eq!(ty("ret"), length());
}
