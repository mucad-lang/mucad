// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Tests for evaluating workbenches.

use microcad_builtin::__mu;
use microcad_lang_base::{DisplayWithCtx, boxed};
use microcad_lang_eval::{Callable, Eval, EvalContext};
use microcad_lang_resolve::{
    SymbolId, call_builtin, expr,
    library::symbol::{
        self, Parameter, Path, SourceStatement, WorkbenchExpression, WorkbenchStatement, workbench,
    },
};

use microcad_lang_types::{
    ArgumentValue, ArgumentValueList, Length, ModelTree, Value, argument_value, list, tuple,
};
use microcad_macros::parameter_list;

/// Expressions used for testing
pub mod helper {
    use microcad_lang_resolve::{call_builtin, expr};

    use super::*;

    /// __mu::geo2d::Circle(radius = expr)
    pub fn call_circle(radius: impl Into<WorkbenchExpression>) -> WorkbenchExpression {
        call_builtin!(geo2d::Circle(radius = radius)).into()
    }

    /// Input placeholder: @input
    /// TODO: Might be replaced with `self` in the future
    pub fn input() -> WorkbenchExpression {
        symbol::workbench::Marker::input().into()
    }

    /// [1..n] / n * 360°
    pub fn polar_expr() -> symbol::ConstantExpression {
        // [1..n]
        let range = call_builtin!(core::range(start = expr!(1), end = expr!(n)));
        // / n
        let div = call_builtin!(core::div(lhs = range, rhs = expr!(n)));
        // * 360°
        let mul = call_builtin!(core::mul(lhs = div, rhs = expr!(360.0 deg)));
        mul.into()
    }
}

/// Evaluate something into a model tree and test snapshot
pub fn eval_to_model_test<T: Eval>(name: &str, t: T) -> ModelTree {
    let mut context = EvalContext::new();
    let value: Value = t.eval(&mut context).expect("No error");
    let model: ModelTree = value.into();
    insta::assert_snapshot!(name, model.to_string_with_ctx(&context));
    model
}

/// Call a workbench with arguments to produce a model tree and test snapshot
pub fn call_workbench<T: Callable<ModelTree>>(
    name: &str,
    workbench: &T,
    arg_iter: impl IntoIterator<Item = ArgumentValue>,
) -> ModelTree {
    let mut context = EvalContext::new();
    let model: ModelTree = workbench
        .call(&ArgumentValueList::from_iter(arg_iter), &mut context)
        .expect("No error");
    insta::assert_snapshot!(name, model.to_string_with_ctx(&context));
    model
}

/// Call an operation with `self` and arguments to produce a model tree and test snapshot.
pub fn call_op<T: Callable<ModelTree>>(
    name: &str,
    op: &T,
    self_: impl Eval,
    arg_iter: impl IntoIterator<Item = ArgumentValue>,
) -> ModelTree {
    let mut context = EvalContext::new();
    let input: Value = self_.eval(&mut context).expect("No error");
    let args = std::iter::once(ArgumentValue::new(input, Some("self".into()))).chain(arg_iter);
    let model: ModelTree = op
        .call(&ArgumentValueList::from_iter(args), &mut context)
        .expect("No error");
    insta::assert_snapshot!(name, model.to_string_with_ctx(&context));
    model
}

/// { __mu::geo2d::Circle(radius = 4.0mm); }
#[test]
fn group() {
    eval_to_model_test(
        "group",
        symbol::workbench::Group::new([WorkbenchStatement::expr(call_builtin!(geo2d::Circle(
            radius = expr!(4.0 mm)
        )))]),
    );
}

/// {
///     prop a = 4.0mm; // Add property to current model tree
///     __mu::geo2d::Circle(a); // Add circle as new Model Tree
/// }
#[test]
fn group_with_property() {
    let model = eval_to_model_test(
        "group_with_property",
        symbol::workbench::Group::new([
            WorkbenchStatement::prop("a", Value::mm(4.0)),
            WorkbenchStatement::expr(call_builtin!(geo2d::Circle(radius = expr!(a)))),
        ]),
    );

    let prop = model.get_property_value("a");
    assert_eq!(prop, expr!(4.0 mm));
}

/// __mu::geo2d::Circle(radius = 4.0mm).translate(x = 1.0mm, y = 2.0mm, z = 0.0mm)
#[test]
fn translate_circle() {
    let model = eval_to_model_test(
        "translate_circle",
        call_builtin!(ops::translate(
            self = call_builtin!(geo2d::Circle(radius = expr!(4.0 mm))),
            x = expr!(1.0 mm),
            y = expr!(2.0 mm),
            z = expr!(0.0 mm),
        )),
    );

    let prop = model.get_property_value("radius"); // We should be able to access the property.
    assert_eq!(prop, Value::mm(4.0));
}

/// sketch Circle() { __mu::geo2d::Circle(radius = 4.0mm); }
#[test]
fn circle_without_parameter() {
    let workbench =
        symbol::Workbench::sketch(parameter_list!()).with_statements([WorkbenchStatement::expr(
            call_builtin!(geo2d::Circle(radius = expr!(4.0 mm))),
        )]);

    call_workbench("circle_without_parameter", &workbench, []);
}

/// sketch Circle(radius: Length) { __mu::geo2d::Circle(radius); }
///
/// Call single: Circle(4.0mm)
/// Call multi: Circle([1.0mm, 2.0mm, 3.0mm]);
#[test]
fn circle_parameter() {
    let workbench = symbol::Workbench::sketch(parameter_list!(radius: Length)).with_statements([
        WorkbenchStatement::expr(call_builtin!(geo2d::Circle(radius = expr!(radius)))),
    ]);

    {
        let radius = expr!(4.0 mm);
        let model = call_workbench(
            "circle_parameter_single",
            &workbench,
            [argument_value!(radius = radius.clone())],
        );

        let prop = model.get_property_value("radius");
        assert_eq!(prop, radius);
    }

    {
        let radius = list![Length::mm(1.0), Length::mm(2.0), Length::mm(3.0)];
        let model = call_workbench(
            "circle_parameter_multi",
            &workbench,
            [argument_value!(radius = radius.clone())],
        );

        let prop = model.get_property_value("radius");
        assert_eq!(prop, Value::from(radius));
    }
}

/// sketch Circle(radius: Length) {
///    init(diameter: Length) {
///        radius = diameter / 2;
///    }
///    __mu::geo2d::Circle(radius);
/// }
#[test]
fn circle_init() {
    use helper::*;

    let workbench = symbol::Workbench::sketch(parameter_list!(radius: Length))
        .with_inits([
            workbench::Init::new(parameter_list!(diameter: Length)).with_statements([
                workbench::InitStatement::new(
                    "radius",
                    call_builtin!(core::div(lhs = expr!(diameter), rhs = expr!(2.0))),
                ),
            ]),
        ])
        .with_statements([WorkbenchStatement::expr(call_circle(Path::Resolved(
            SymbolId::Local("radius".into()),
        )))]);

    let diameter = Length::mm(8.0);
    let model = call_workbench(
        "circle_init",
        &workbench,
        [argument_value!(diameter = diameter)],
    );

    let prop = model.get_property_value("radius");
    assert_eq!(prop, expr!(4.0 mm));
}

/// a = 32mm;
/// __mu::geo2d::Circle(radius = a);
#[test]
fn circle_source() {
    use helper::*;

    let source = symbol::Source {
        statements: boxed([
            SourceStatement::assignment("a", Value::mm(32.0)),
            SourceStatement::expr(call_circle(expr!(a))),
        ]),
    };

    eval_to_model_test("source", source);
}

/// Evaluate [1..n]
#[test]
fn polar_expr_test() {
    use helper::*;

    let source = symbol::Source {
        statements: boxed([
            SourceStatement::assignment("n", Value::from(4)),
            SourceStatement::assignment("a", polar_expr()),
            SourceStatement::expr(expr!(a)),
        ]),
    };

    eval_to_model_test("polar_expr_test", source);
}

/// An operation that rotates a part.
///
/// op rotate(matrix: Matrix3) {
///    init(angle: Angle, axis = __mu::math::Z) {
///        matrix = __mu::math::rotate_around_axis(angle, x = axis.x, y = axis.y, z = axis.z);
///    }
///    init(x = 0°, y = 0°, z = 0°) {
///        matrix = __mu::math::rotate_xyz(x, y, z);
///    init(roll = 0°, pitch = 0°, yaw = 0°) {
///        matrix = __mu::math::rotate_xyz(roll, pitch, yaw);
///    }
///    @input.__mu::ops::rotate(matrix)
/// }
#[test]
fn op_rotate() {
    use helper::*;

    let workbench = symbol::Workbench::op(parameter_list!(matrix: Mat3))
        .with_inits([
            // init(angle: Angle, axis = __mu::math::Z)
            workbench::Init::new(
                parameter_list!(angle: Angle, axis: Any = microcad_builtin::mu::math::Z.value()),
            )
            .with_statements([workbench::InitStatement::new(
                "matrix",
                call_builtin!(math::rotate_around_axis(
                    angle = expr!(angle),
                    x = expr!(axis.x),
                    y = expr!(axis.y),
                    z = expr!(axis.z),
                )),
            )]),
            // init(x = 0°, y = 0°, z = 0°)
            workbench::Init::new(parameter_list!(
                x: Angle = Value::deg(0.0),
                y: Angle = Value::deg(0.0),
                z: Angle = Value::deg(0.0),
            ))
            .with_statements([workbench::InitStatement::new(
                "matrix",
                call_builtin!(math::rotate_xyz(x = expr!(x), y = expr!(y), z = expr!(z))),
            )]),
            // init(roll = 0°, pitch = 0°, yaw = 0°)
            workbench::Init::new(parameter_list!(
                roll: Angle = Value::deg(0.0),
                pitch: Angle = Value::deg(0.0),
                yaw: Angle = Value::deg(0.0),
            ))
            .with_statements([workbench::InitStatement::new(
                "matrix",
                call_builtin!(math::rotate_xyz(
                    x = expr!(roll),
                    y = expr!(pitch),
                    z = expr!(yaw)
                )),
            )]),
        ])
        .with_statements([WorkbenchStatement::expr(call_builtin!(ops::rotate(
            self = input(),
            matrix = expr!(matrix)
        )))]);

    // Test calling (angle = 90.0, axis = (x = 0, y = 0, z = 1)) via init(angle, axis)
    call_op(
        "rotate_angle_axis",
        &workbench,
        call_circle(Value::mm(4.0)),
        [
            argument_value!(angle = Value::deg(90.0)),
            argument_value!(axis = tuple!(x = 0.0, y = 0.0, z = 1.0)),
        ],
    );

    // Test calling (angle = [0.0, 45.0, 90.0], axis = (x = 0, y = 0, z = 1)) via init(angle, axis)
    call_op(
        "rotate_angle_axis_multi",
        &workbench,
        call_circle(Value::mm(4.0)),
        [
            argument_value!(
                angle = Value::list([Value::deg(0.0), Value::deg(45.0), Value::deg(90.0)])
            ),
            argument_value!(axis = tuple!(x = 0.0, y = 0.0, z = 1.0)),
        ],
    );

    // Test calling (angle = 90.0) via init(angle, axis)
    call_op(
        "rotate_angle_axis_default_arg",
        &workbench,
        call_circle(Value::mm(4.0)),
        [argument_value!(angle = Value::deg(90.0))],
    );
}

/// An operation that arranges copies of a part on a circle.
///
/// Today's µcad source would take `n` and compute the angles itself:
///
/// ```mu
/// op polar_array(n: Integer, r: Length) {
///     @input.__mu::ops::translate(x = r, y = 0.0mm, z = 0.0mm)
///         .rotate([1..n] / n * 360°)
/// }
/// ```
///
/// The evaluator cannot expand a computed list inside a workbench yet, so this test takes the
/// angles as parameter and lets the parameter multiplicity create the copies:
///
/// ```mu
/// op polar_array(angle: Angle, r: Length) {
///     __mu::ops::rotate(
///         self = __mu::ops::translate(self = @input, x = r, y = 0.0mm, z = 0.0mm),
///         matrix = __mu::math::rotate_around_axis(angle, x = 0.0, y = 0.0, z = 1.0),
///     )
/// }
///
/// __mu::geo2d::Circle(radius = 4.0mm).polar_array(angle = [1..8] / 8 * 360°, r = 10.0mm)
/// ```
#[test]
fn op_polar_array() {
    use helper::*;

    let workbench = symbol::Workbench::op(parameter_list!(angle: Angle, r: Length))
        .with_statements([WorkbenchStatement::expr(call_builtin!(ops::rotate(
            self = call_builtin!(ops::translate(
                self = input(),
                x = expr!(r),
                y = expr!(0.0 mm),
                z = expr!(0.0 mm),
            )),
            matrix = call_builtin!(math::rotate_around_axis(
                angle = expr!(angle),
                x = expr!(0.0),
                y = expr!(0.0),
                z = expr!(1.0),
            )),
        )))]);

    // [1..n] / n * 360°
    let n = 3;
    let angles = Value::list((1..=n).map(|i| Value::deg(i as f64 / n as f64 * 360.0)));

    call_op(
        "polar_array",
        &workbench,
        call_circle(Value::mm(4.0)),
        [
            argument_value!(angle = angles),
            argument_value!(r = Value::mm(10.0)),
        ],
    );
}
