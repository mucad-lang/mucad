// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Tests for the high-level compiler API.

use mu::base::DisplayWithCtx;
use mucad_driver::{Compiler, prelude as mu};

/// A sketch and an operation, called as method on the sketch.
const DISC: &str = r#"
sketch Disc(radius: Length) {
    __mu::geo2d::Circle(radius);
}

op Raise(height: Length) {
    @input.__mu::ops::extrude(height, scale_x = 1.0, scale_y = 1.0, twist = 0°);
}

Disc(radius = 4mm).Raise(height = 2mm);
"#;

fn display(model: &mu::ModelTree) -> String {
    model.to_string_with_ctx(&mu::builtin::BuiltinLibrary::new())
}

/// A compiler without standard library, independent of any installation.
fn compiler() -> Compiler {
    Compiler::from_config(&mu::DriverConfig {
        no_std: true,
        ..Default::default()
    })
}

#[test]
fn compile_source() {
    let mut compiler = compiler();
    let (model, diags) = compiler.compile(DISC).expect("A model");

    assert!(!diags.has_errors(), "{diags:?}");
    insta::assert_snapshot!("compile_source", display(&model));
}

#[test]
fn compile_unresolved_symbol() {
    let mut compiler = compiler();
    let (_model, diags) = compiler
        .compile("Unknown(radius = 4mm);")
        .expect("A model, despite errors");

    assert!(diags.has_errors(), "{diags:?}");
}

/// Built-in transforms and operations of `__mu::ops`.
#[test]
fn compile_ops() {
    let mut compiler = compiler();
    let (model, diags) = compiler
        .compile(
            r#"
            __mu::geo2d::Circle(radius = 1mm).__mu::ops::scale(x = 2.0, y = 3.0, z = 1.0);
            __mu::geo2d::Circle(radius = 1mm).__mu::ops::orient(x = 0.0, y = 1.0, z = 0.0);
            __mu::geo2d::Circle(radius = 1mm).__mu::ops::hull();
            __mu::geo2d::Circle(radius = 1mm).__mu::ops::buffer(distance = 0.5mm);
            __mu::geo2d::Circle(radius = 1mm).__mu::ops::revolve(angle = 360°);
            "#,
        )
        .expect("A model");

    assert!(!diags.has_errors(), "{diags:?}");
    insta::assert_snapshot!("compile_ops", display(&model));
}

/// Built-in primitives of `__mu::geo2d`.
#[test]
fn compile_geo2d() {
    let mut compiler = compiler();
    let (model, diags) = compiler
        .compile(
            r#"
            __mu::geo2d::Ngon(n = 6, radius = 10mm);
            __mu::geo2d::Sector(radius = 10mm, start = 0°, end = 90°);
            __mu::geo2d::RoundedRect(
                width = 20mm, height = 10mm, x = 0mm, y = 0mm,
                radius_bottom_left = 1mm, radius_bottom_right = 1mm,
                radius_top_left = 2mm, radius_top_right = 2mm,
            );
            __mu::geo2d::Line(x0 = 0.0, y0 = 0.0, x1 = 10.0, y1 = 5.0);
            __mu::geo2d::InvoluteGearProfile(module = 1mm, teeth = 12, pressure_angle = 20°);
            __mu::geo2d::SinusoidalGearProfile(module = 1mm, teeth = 12, roundness = 0.5, stretch = 0.5);
            __mu::geo2d::Text(height = 5mm, text = "µcad", font_file = "");
            "#,
        )
        .expect("A model");

    assert!(!diags.has_errors(), "{diags:?}");
    insta::assert_snapshot!("compile_geo2d", display(&model));
}

/// Output type of the first model in a compiled source.
fn output_type(model: &mu::ModelTree) -> mu::types::ModelType {
    use mu::types::model::NodeExt;
    model.root().deduce_output_type()
}

/// Built-in primitives of `__mu::geo3d`.
#[test]
fn compile_geo3d() {
    let mut compiler = compiler();
    let (model, diags) = compiler
        .compile("__mu::geo3d::Sphere(radius = 10mm);")
        .expect("A model");

    assert!(!diags.has_errors(), "{diags:?}");
    assert_eq!(output_type(&model), mu::types::ModelType::Geometry3D);
    insta::assert_snapshot!("compile_geo3d", display(&model));
}

/// Like `std::geo3d::Torus` and `std::geo3d::Cylinder`: 3D models made of 2D primitives.
#[test]
fn compile_geo3d_from_2d() {
    let mut compiler = compiler();
    for source in [
        // Torus(major_radius = 45mm, minor_radius = 5mm)
        "__mu::geo2d::Circle(radius = 5mm).__mu::ops::translate(x = 45mm, y = 0mm, z = 0mm).__mu::ops::revolve(angle = 360°);",
        // Cylinder(radius = 10mm, height = 20mm)
        "__mu::geo2d::Circle(radius = 10mm).__mu::ops::extrude(height = 20mm, scale_x = 1.0, scale_y = 1.0, twist = 0°).__mu::ops::translate(x = 0mm, y = 0mm, z = -10mm);",
    ] {
        let (model, diags) = compiler.compile(source).expect("A model");

        assert!(!diags.has_errors(), "{diags:?}");
        assert_eq!(
            output_type(&model),
            mu::types::ModelType::Geometry3D,
            "{source}"
        );
    }
}

/// A compiler can be used again, the main library of the previous run is replaced.
#[test]
fn compile_twice() {
    let mut compiler = compiler();
    let (first, _) = compiler.compile(DISC).expect("A model");
    let (second, diags) = compiler.compile(DISC).expect("A model");

    assert!(!diags.has_errors(), "{diags:?}");
    assert_eq!(display(&first), display(&second));
}

/// The colors of the models in a model tree.
fn colors(model: &mu::ModelTree) -> Vec<mu::types::Color> {
    use mu::types::model::Attribute;
    model
        .root()
        .descendants()
        .flat_map(|node| node.attr.iter().cloned().collect::<Vec<_>>())
        .filter_map(|attr| match attr {
            Attribute::Color(color) => Some(color),
            _ => None,
        })
        .collect()
}

/// Model attributes of expression statements and assignments.
#[test]
fn compile_model_attributes() {
    use mu::types::Color;

    let mut compiler = compiler();
    let (model, diags) = compiler
        .compile(
            r##"
            #[color = "#FFFFFF"]
            c = __mu::geo2d::Circle(radius = 1mm);
            c;

            #[color = (r = 1.0, g = 0.0, b = 0.0)]
            #[resolution = 0.5mm]
            #[name = "disc"]
            __mu::geo2d::Circle(radius = 2mm);
            "##,
        )
        .expect("A model");

    assert!(!diags.has_errors(), "{diags:?}");
    assert_eq!(
        colors(&model),
        [Color::rgb(1.0, 1.0, 1.0), Color::rgb(1.0, 0.0, 0.0)]
    );

    let disc = model.root().children().nth(1).expect("A disc");
    assert_eq!(disc.name.as_ref().map(|name| name.as_str()), Some("disc"));
    assert_eq!(
        disc.resolution(),
        Some(&mu::types::model::attribute::ResolutionAttribute::Absolute(
            mu::types::Length::mm(0.5)
        ))
    );
}

/// Invalid attribute values and attributes of values without models are errors.
#[test]
fn compile_invalid_model_attributes() {
    let mut compiler = compiler();
    for source in [
        "#[color = 42]\n__mu::geo2d::Circle(radius = 1mm);",
        "#[color = \"no color\"]\n__mu::geo2d::Circle(radius = 1mm);",
        "#[resolution = 10°]\n__mu::geo2d::Circle(radius = 1mm);",
        "#[color = \"red\"]\nx = 42;",
    ] {
        let (_, diags) = compiler.compile(source).expect("A model, despite errors");
        assert!(diags.has_errors(), "{source}");
    }
}

/*


/// A compiler with the standard library of this repository (relative to `crates/driver`).
fn std_compiler() -> Compiler {
    Compiler::from_config(&mu::DriverConfig {
        search_paths: vec!["../std/lib".into()],
        ..Default::default()
    })
}

/// A source using the standard library.
#[test]
fn compile_with_std() {
    let mut compiler = std_compiler();
    let (model, diags) = compiler
        .compile(mu::Source::from("std::geo2d::Circle(radius = 4mm);"))
        .expect("A model");

    assert!(!diags.has_errors(), "{diags:?}");
    insta::assert_snapshot!("compile_with_std", display(&model));
}
*/

/// A compiler with the minimal standard library in `tests/data/lib/std`.
fn fixture_std_compiler() -> Compiler {
    Compiler::from_config(&mu::DriverConfig {
        search_paths: vec!["tests/data/lib".into()],
        ..Default::default()
    })
}

/// External library symbols: `std::…`, `mu::std::…`, and std referring to itself.
#[test]
fn compile_with_std_fixture() {
    let mut compiler = fixture_std_compiler();
    let (model, diags) = compiler
        .compile(
            r#"
            std::geo2d::Circle(radius = 4mm);
            mu::std::geo2d::Circle(radius = 5mm);
            std::geo3d::Cylinder(radius = 6mm, height = 2mm);
            "#,
        )
        .expect("A model");

    assert!(!diags.has_errors(), "{diags:?}");
    insta::assert_snapshot!("compile_with_std_fixture", display(&model));
}

/// A compiler with the standard library of this repository (relative to `crates/driver`).
fn std_compiler() -> Compiler {
    Compiler::from_config(&mu::DriverConfig {
        search_paths: vec!["../std/lib".into()],
        ..Default::default()
    })
}

/// `std::assert` re-exports the built-in `__mu::debug::assert`: a holding condition.
#[test]
fn compile_std_assert() {
    let mut compiler = std_compiler();
    let (_, diags) = compiler
        .compile(r#"std::assert(1 < 2, "One is less than two");"#)
        .expect("A model");

    assert!(!diags.has_errors(), "{diags:?}");
}

/// `std::assert` with a failing condition reports its message.
#[test]
fn compile_std_assert_fails() {
    let mut compiler = std_compiler();
    let diags = match compiler.compile(r#"std::assert(2 < 1, "Two is not less than one");"#) {
        Ok((_, diags)) | Err(diags) => diags,
    };

    assert!(diags.has_errors(), "{diags:?}");
    assert!(
        format!("{diags:?}").contains("Assertion failed: Two is not less than one"),
        "{diags:?}"
    );
}

/// `std::geo2d::Circle` with its building plan and with its `init`.
#[test]
fn compile_std_circle() {
    let mut compiler = std_compiler();
    let (model, diags) = compiler
        .compile(mu::Source::from(
            r#"
            std::geo2d::Circle(radius = 4mm);
            std::geo2d::Circle(diameter = 8mm, center = (x = 1mm, y = 2mm));
            "#,
        ))
        .expect("A model");

    assert!(!diags.has_errors(), "{diags:?}");
    insta::assert_snapshot!("compile_std_circle", display(&model));
}

/// Built-in calls created while lowering (member access, lists, ranges) with identifiers.
#[test]
fn compile_access() {
    let mut compiler = compiler();
    let (model, diags) = compiler
        .compile(mu::Source::from(
            r#"
            p = (x = 1mm, y = 2mm);
            ps = [p, p];
            r = [1..3]mm;
            __mu::geo2d::Circle(radius = ps[1].y);
            __mu::geo2d::Circle(radius = r[2]);
            "#,
        ))
        .expect("A model");

    assert!(!diags.has_errors(), "{diags:?}");
    insta::assert_snapshot!("compile_access", display(&model));
}

/// Boolean operators on models, a chain of equal operators is flattened.
#[test]
fn compile_boolean_ops() {
    let mut compiler = compiler();
    let (model, diags) = compiler
        .compile(mu::Source::from(
            r#"
            __mu::geo2d::Circle(radius = 3mm) - __mu::geo2d::Circle(radius = 2mm) - __mu::geo2d::Circle(radius = 1mm);
            __mu::geo2d::Circle(radius = 2mm) | __mu::geo2d::Circle(radius = 1mm);
            __mu::geo2d::Circle(radius = 2mm) & __mu::geo2d::Circle(radius = 1mm);
            "#,
        ))
        .expect("A model");

    assert!(!diags.has_errors(), "{diags:?}");
    insta::assert_snapshot!("compile_boolean_ops", display(&model));
}

/// The boolean operations of `__mu::ops` build the same models as the boolean operators.
#[test]
fn compile_boolean_op_calls() {
    let compile = |code: &str| {
        let (model, diags) = compiler().compile(mu::Source::from(code)).expect("A model");
        assert!(!diags.has_errors(), "{diags:?}");
        display(&model)
    };

    let a = "__mu::geo2d::Circle(radius = 3mm)";
    let b = "__mu::geo2d::Circle(radius = 2mm)";
    let c = "__mu::geo2d::Circle(radius = 1mm)";

    assert_eq!(
        compile(&format!("{{ {a}; {b}; {c}; }}.__mu::ops::subtract();")),
        compile(&format!("{a} - {b} - {c};"))
    );
    assert_eq!(
        compile(&format!("{{ {a}; {b}; }}.__mu::ops::union();")),
        compile(&format!("{a} | {b};"))
    );
    assert_eq!(
        compile(&format!("{{ {a}; {b}; }}.__mu::ops::intersect();")),
        compile(&format!("{a} & {b};"))
    );
}

/// The source files of a compilation include those of the standard library.
#[test]
fn compile_source_files() {
    let mut compiler = std_compiler();
    let (_, diags) = compiler
        .compile("std::geo2d::Circle(radius = 1mm);")
        .expect("A model");
    assert!(!diags.has_errors(), "{diags:?}");

    let files = compiler.source_files();
    assert!(
        files.iter().any(|path| path.ends_with("std/geo2d.mu")),
        "{files:?}"
    );
}

/// Aliases (`use`, `pub use`) can be called and evaluated, also from the standard library.
#[test]
fn compile_std_alias() {
    let mut compiler = std_compiler();
    let (_, diags) = compiler
        .compile(mu::Source::from(
            r#"
            use std::math::sqrt;

            a = 2cm;
            b = 5cm;
            c = sqrt(a * a + b * b);
            std::print("{c}");
            x = std::math::PI;
            "#,
        ))
        .expect("A model");

    assert!(!diags.has_errors(), "{diags:?}");
}

/// Calling an alias of something which cannot be called reports its name.
#[test]
fn compile_std_alias_not_callable() {
    let mut compiler = std_compiler();
    let diags = match compiler.compile("use std::math::PI;\nPI();") {
        Ok((_, diags)) | Err(diags) => diags,
    };

    assert!(diags.has_errors(), "{diags:?}");
    let message = diags.iter().next().expect("An error").report.to_string();
    assert_eq!(message, "Symbol `__mu::math::PI` cannot be called.");
}

/// `std::assert` without message, also with `std::list::all_equal` for more than two values.
#[test]
fn compile_std_assert_default_message() {
    let mut compiler = std_compiler();
    let (_, diags) = compiler
        .compile("std::assert(1 + 1 == 2);\nstd::assert(std::list::all_equal([2, 1 + 1, 4 - 2]));")
        .expect("A model");
    assert!(!diags.has_errors(), "{diags:?}");

    let diags = match compiler.compile("std::assert(2 < 1);") {
        Ok((_, diags)) | Err(diags) => diags,
    };
    assert!(
        format!("{diags:?}").contains("Assertion failed: condition is false"),
        "{diags:?}"
    );
}

/// A failing assertion refers to its call in the source code.
#[test]
fn compile_std_assert_location() {
    let mut compiler = std_compiler();
    let diags = match compiler.compile("a = 2cm;\nstd::assert(a == 3cm);\n") {
        Ok((_, diags)) | Err(diags) => diags,
    };

    let diag = diags.iter().next().expect("An assertion error");
    assert!(diag.src_ref.is_some(), "{diag:?}");
    // Lines are counted from zero.
    assert_eq!(diag.src_ref.line(), Some(1), "{diag:?}");
}

/// Issues of built-ins (e.g. a failed `expect`) are reported, evaluation continues.
#[test]
fn compile_builtin_issues() {
    let mut compiler = compiler();
    let (model, diags) = compiler
        .compile(
            "__mu::debug::expect(cond = 1 > 2, message = \"one to be larger\");\n\
            __mu::geo2d::Circle(radius = 1mm);",
        )
        .expect("A model, despite the failed expectation");

    assert_eq!(diags.error_count(), 1, "{diags:?}");
    let diag = diags.iter().next().expect("An error");
    assert_eq!(diag.report.to_string(), "Expected one to be larger");
    assert_eq!(diag.src_ref.line(), Some(0), "{diag:?}");
    assert!(display(&model).contains("Circle"), "{}", display(&model));
}

/// Arguments can be abbreviated by the first letter of a parameter, if it is unique.
#[test]
fn compile_abbreviated_arguments() {
    let mut compiler = std_compiler();
    let (_, diags) = compiler
        .compile("std::geo2d::Circle(r = 42mm);")
        .expect("A model");
    assert!(!diags.has_errors(), "{diags:?}");

    let diags = match compiler
        .compile("fn f(foo: Integer, foz: Integer) { foo + foz }\nx = f(f = 10, foz = 1);")
    {
        Ok((_, diags)) | Err(diags) => diags,
    };
    assert!(
        format!("{diags:?}").contains("Argument `f` is ambiguous: foo,foz"),
        "{diags:?}"
    );
}

/// Errors name the called symbol with its full path.
#[test]
fn compile_symbol_name() {
    let mut compiler = std_compiler();
    let diags = match compiler.compile("std::geo2d::Circle(foo = 1mm);") {
        Ok((_, diags)) | Err(diags) => diags,
    };
    let message = diags.iter().next().expect("An error").report.to_string();
    assert_eq!(
        message,
        "Workbench `std::geo2d::Circle` cannot find initialization for those arguments"
    );
}

/// Quantities of different units are equal, if their values in the base unit are equal.
#[test]
fn compile_quantity_units() {
    let mut compiler = std_compiler();
    let (_, diags) = compiler
        .compile(
            "std::assert_eq(6cm - 2cm, 0.04m);\n\
            std::assert_eq(1in, 25.4mm);\n\
            std::assert_eq(0.1m, 100mm);\n\
            std::assert_eq(1ml, 1cm3);",
        )
        .expect("A model");
    assert!(!diags.has_errors(), "{diags:?}");
}

/// Micro units with prefix `µ`, e.g. `µm` or `µl`.
#[test]
fn compile_micro_units() {
    let mut compiler = std_compiler();
    let (_, diags) = compiler
        .compile(
            "std::assert_eq(1000µm, 1mm);\n\
            std::assert_eq(1000µl, 1ml);\n\
            std::assert_eq(1000000µm2, 1mm2);\n\
            std::assert_eq(1000000000µm³, 1mm³);",
        )
        .expect("A model");
    assert!(!diags.has_errors(), "{diags:?}");
}

/// Local variables of functions can be assigned again, unlike properties.
#[test]
fn compile_function_reassignment() {
    let mut compiler = std_compiler();
    let (_, diags) = compiler
        .compile(
            "fn f() -> Integer {\n    a = 5;\n    a = a * 2;\n    a\n}\nstd::assert_eq(f(), 10);",
        )
        .expect("A model");
    assert!(!diags.has_errors(), "{diags:?}");
}

/// Symbols of the prelude of the standard library can be used by name.
#[test]
fn compile_prelude() {
    let mut compiler = std_compiler();
    let (_, diags) = compiler
        .compile(
            "Circle(radius = 1mm).translate(x = 1mm).hull();\n\
            Rect(10mm).extrude(height = 1mm);\n\
            Circle(radius = 1mm).translate(x = 2mm).revolve(90°);\n\
            Circle(radius = 1mm).rotate(45°).scale(2.0);\n\
            Circle(radius = 1mm).extrude(height = 1mm).orient(v = (x = 1.0, y = 0.0, z = 0.0));\n\
            assert(true);\n\
            assert_eq(1, 1);\n\
            print(\"prelude\");",
        )
        .expect("A model");
    assert!(!diags.has_errors(), "{diags:?}");
}

/// Own symbols have priority over the prelude.
#[test]
fn compile_prelude_shadowed() {
    let mut compiler = std_compiler();
    let (_, diags) = compiler
        .compile("fn assert_eq(a: Integer, b: Integer) {}\nassert_eq(1, 2);")
        .expect("A model");
    assert!(!diags.has_errors(), "{diags:?}");
}

/// Without standard library there is no prelude.
#[test]
fn compile_prelude_no_std() {
    let mut compiler = compiler();
    let (_, diags) = compiler
        .compile("Circle(radius = 1mm);")
        .expect("A model, despite errors");
    assert!(diags.has_errors(), "{diags:?}");
}

/// A list argument for a parameter of its element type calls the workbench once per element.
#[test]
fn compile_positional_list_expansion() {
    let mut compiler = std_compiler();
    let (model, diags) = compiler
        .compile(
            "sketch MySketch(x: Integer) {\n\
                if x > 0 { Circle(radius = 5mm) } else { Rect(10mm) }\n\
            }\n\
            MySketch([-1, 2]);",
        )
        .expect("A model");
    assert!(!diags.has_errors(), "{diags:?}");
    insta::assert_snapshot!("compile_positional_list_expansion", display(&model));
}

/// An initialization error is reported at the call with the possible initializations.
#[test]
fn compile_no_initialization_found() {
    let mut compiler = std_compiler();
    let diags = compiler
        .compile(
            "sketch S(x: Integer) {\n\
                init(a: Length) { x = 1; }\n\
                Circle(radius = 1mm)\n\
            }\n\
            S(\"text\");",
        )
        .expect_err("No model");
    let diag = diags.iter().next().expect("An error");
    // The call in the fifth line (zero-based), not the definition.
    assert_eq!(diag.src_ref.line(), Some(4));
    let help = diag.report.help().expect("A help").to_string();
    assert!(help.contains("init(a: Length)"), "{help}");
    assert!(help.contains("init(x: Integer)"), "{help}");
}

/// A local cannot be called, so it does not shadow the operation of a method call,
/// e.g. `center` in the tutorial, which is also an argument of `Circle`.
#[test]
fn compile_local_does_not_shadow_method() {
    let mut compiler = std_compiler();
    let (_, diags) = compiler
        .compile("use std::ops::*;\ncenter = (x = 1mm, y = 0mm);\nCircle(radius = 1mm, center).center();")
        .expect("A model");
    assert!(!diags.has_errors(), "{diags:?}");
}

/// A local cannot be called, so it does not shadow a called function, but is used as value.
#[test]
fn compile_local_does_not_shadow_function() {
    let mut compiler = std_compiler();
    let (_, diags) = compiler
        .compile("fn double(x: Integer) -> Integer { x * 2 }\ndouble = 3;\nassert_eq(double(double), 6);")
        .expect("A model");
    assert!(!diags.has_errors(), "{diags:?}");
}

/// A compiler with a virtual file system containing the standard library, e.g. in a browser.
fn virtual_compiler(file_system: mu::base::VirtualFileSystem) -> Compiler {
    Compiler::from_config(&mu::DriverConfig {
        search_paths: vec!["/lib".into()],
        ..Default::default()
    })
    .with_file_system(std::sync::Arc::new(file_system))
}

/// The standard library is loaded from a virtual file system.
#[test]
fn compile_virtual_std() {
    let file_system = mu::stdlib::StdLib::virtual_file_system("/lib");
    let mut compiler = virtual_compiler(file_system);
    let (_, diags) = compiler
        .compile("Circle(radius = 1mm).translate(x = 1mm);")
        .expect("A model");
    assert!(!diags.has_errors(), "{diags:?}");
}

/// File modules are loaded from a virtual file system.
#[test]
fn compile_virtual_file_module() {
    use mu::base::FileSystem;

    let file_system = mu::stdlib::StdLib::virtual_file_system("/lib")
        .with_file("/project/main.mu", "mod geo;\ngeo::Shape();")
        .with_file(
            "/project/main/geo.mu",
            "pub sketch Shape() {\n    Circle(radius = 1mm);\n}",
        );
    let source = file_system
        .load_source(std::path::Path::new("/project/main.mu"))
        .expect("A source");
    let mut compiler = virtual_compiler(file_system);
    let (_, diags) = compiler.compile(source).expect("A model");
    assert!(!diags.has_errors(), "{diags:?}");
}

/// The names to complete at `|` in `code`, with the symbols of the most recent compilation.
fn completion_names(compiler: &Compiler, code: &str) -> Vec<String> {
    let offset = code.find('|').expect("A position");
    let source = mu::Source::from(code.replacen('|', "", 1).as_str());
    compiler
        .completions(&source, offset)
        .into_iter()
        .map(|completion| completion.name.to_string())
        .collect()
}

/// Names of the source, of libraries and of the prelude are completed, but no built-ins.
#[test]
fn complete_names() {
    let mut compiler = std_compiler();
    let code = "sketch Disc() {}\nDi|";
    let _ = compiler.compile(code.replace('|', "").as_str());

    let names = completion_names(&compiler, code);
    for name in ["Disc", "Circle", "std"] {
        assert!(names.iter().any(|n| n == name), "{name} in {names:?}");
    }
    assert!(!names.iter().any(|n| n.starts_with("__")), "{names:?}");
}

/// The symbols in a module of a library are completed, also while the code has syntax errors.
#[test]
fn complete_library_module() {
    let mut compiler = std_compiler();
    let code = "std::geo2d::Ci|";
    let _ = compiler.compile(code.replace('|', "").as_str());

    let names = completion_names(&compiler, code);
    for name in ["Circle", "Rect"] {
        assert!(names.iter().any(|n| n == name), "{name} in {names:?}");
    }
    assert!(completion_names(&compiler, "__mu::geo2d::|").is_empty());
}

/// Names are completed with the symbols of a previous version of the source.
#[test]
fn complete_names_of_previous_compilation() {
    let mut compiler = std_compiler();
    let (_, diags) = compiler.compile("sketch Disc() {}").expect("A model");
    assert!(!diags.has_errors(), "{diags:?}");

    let names = completion_names(&compiler, "sketch Disc() {}\nDisc(); Di|");
    assert!(names.iter().any(|n| n == "Disc"), "{names:?}");
}

/// Numbers are not completed.
#[test]
fn complete_no_number() {
    let compiler = std_compiler();
    assert!(completion_names(&compiler, "x = 12|").is_empty());
}

/// The export paths of the root model of a model tree, `None` for `#[export]` without file.
fn exports(model: &mu::ModelTree) -> Vec<Option<String>> {
    use mu::types::model::Attribute;
    model
        .root()
        .attr
        .iter()
        .filter_map(|attr| match attr {
            Attribute::Export(export) => Some(export.path.clone()),
            _ => None,
        })
        .collect()
}

/// Export attributes of source statements are attached to their models.
#[test]
fn compile_export_attributes() {
    let mut compiler = compiler();
    let (model, diags) = compiler
        .compile(
            r#"
            #[export]
            shape = __mu::geo2d::Circle(radius = 1mm);

            #[export = "mylogo.svg"]
            #[export("mylogo.dxf")]
            logo = __mu::geo2d::Circle(radius = 2mm);

            #[export = "logo.stl"]
            logo.__mu::ops::extrude(height = 1mm, scale_x = 1.0, scale_y = 1.0, twist = 0°);
            "#,
        )
        .expect("A model");
    assert!(!diags.has_errors(), "{diags:?}");

    // Properties of source statements are hidden.
    let property = |name: &str| match model.root().get_property_value(name) {
        mu::Value::Model(model) => exports(&model),
        value => panic!("A model: {value}"),
    };
    assert_eq!(property("shape"), [None]);
    assert_eq!(
        property("logo"),
        [Some("mylogo.svg".into()), Some("mylogo.dxf".into())]
    );

    let extruded = model.root().children().next().expect("A child");
    let extruded = mu::ModelTree::new(extruded.get().clone());
    assert_eq!(exports(&extruded), [Some("logo.stl".into())]);
}

/// Exports need a file name without statement name, are only allowed in source files and
/// their file names are strings.
#[test]
fn compile_invalid_export_attributes() {
    let mut compiler = compiler();
    for source in [
        "#[export]\n__mu::geo2d::Circle(radius = 1mm);",
        "sketch S() {\n    #[export = \"s.svg\"]\n    __mu::geo2d::Circle(radius = 1mm);\n}\nS();",
        "#[export = 42]\nc = __mu::geo2d::Circle(radius = 1mm);",
        "#[export = \"x.svg\"]\nx = 42;",
    ] {
        let diags = match compiler.compile(source) {
            Ok((_, diags)) | Err(diags) => diags,
        };
        assert!(diags.has_errors(), "{source}");
    }
}

/// A directory with `mu.toml` and `lib.mu` is a library.
#[test]
fn library_dir() {
    let compiler = compiler();
    assert!(compiler.is_library_dir("tests/data/lib/std"));
    // No `mu.toml`.
    assert!(!compiler.is_library_dir("tests/data/lib"));
    assert!(!compiler.is_library_dir("tests/data/lib/std/lib.mu"));
}

/// A library is compiled from its directory.
#[test]
fn compile_lib() {
    let mut compiler = compiler();
    let (lib, diags) = compiler
        .compile_lib("tests/data/lib/std")
        .expect("A library");
    assert!(!diags.has_errors(), "{diags:?}");
    assert_eq!(lib.name().map(|name| name.as_str()), Some("std"));
}

/// The errors of compiling `source`.
fn errors(compiler: &mut Compiler, source: &str) -> Vec<String> {
    let diags = match compiler.compile(source) {
        Ok((_, diags)) | Err(diags) => diags,
    };
    diags
        .iter()
        .filter(|diag| diag.severity() == Some(mu::base::Severity::Error))
        .map(|diag| diag.to_string())
        .collect()
}

/// Parameters need a type or a default value, which must match the type.
#[test]
fn type_check_parameters() {
    let mut compiler = compiler();
    for (source, error) in [
        (
            "sketch S(x) {}",
            Some("Parameter `x` needs a type or a default value"),
        ),
        ("sketch S(x: Any) {}", None),
        (
            "sketch S(x: Length = 5) {}",
            Some("Type mismatch: Length != Integer"),
        ),
        ("sketch S(x: Scalar = 5) {}", None),
        (
            "const SIZE = 10;\nsketch S(x: Length = SIZE) {}",
            Some("Type mismatch: Length != Integer"),
        ),
        // The return type of a built-in.
        ("sketch S(b: Bool = 1mm > 2mm) {}", None),
        (
            "sketch S(b: Length = 1mm > 2mm) {}",
            Some("Type mismatch: Length != Bool"),
        ),
        (
            "fn f(x: Bool = 1mm) { return x; }",
            Some("Type mismatch: Bool != Length"),
        ),
    ] {
        let errors = errors(&mut compiler, source);
        match error {
            Some(error) => assert!(
                errors.iter().any(|e| e.contains(error)),
                "{source}: {errors:?}"
            ),
            None => assert!(errors.is_empty(), "{source}: {errors:?}"),
        }
    }
}

/// A parameter without type gets the type of its default value.
#[test]
fn type_check_infers_parameter_types() {
    use mu::resolve::{SymbolDef, SymbolNodeExt};

    let mut compiler = compiler();
    let (_, diags) = compiler
        .compile("const SIZE = 10mm;\nsketch S(a = SIZE, b = 1mm > 2mm, c = 1.5, d: Any = 1mm) {}")
        .expect("A model");
    assert!(!diags.has_errors(), "{diags:?}");

    let lib = compiler.library().expect("A library");
    let node = lib
        .root()
        .descendants()
        .find(|node| node.name().is_some_and(|name| name.as_str() == "S"))
        .expect("A sketch");
    let SymbolDef::Workbench(workbench) = node.def() else {
        panic!("A workbench");
    };
    let types: Vec<_> = workbench
        .signature
        .parameters
        .parameters
        .iter()
        .map(|param| param.ty.as_ref().map(|ty| ty.ty.to_string()))
        .collect();
    assert_eq!(
        types,
        [
            Some("Length".into()),
            Some("Bool".into()),
            Some("Scalar".into()),
            // An explicit `Any` stays.
            Some("Any".into()),
        ]
    );
}
