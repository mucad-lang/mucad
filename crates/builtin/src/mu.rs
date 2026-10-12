// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! µcad compiler built-in library.
//!
//! Every compiler built-in starts with `__mu` prefix.
//!
//! The built-ins are grouped into several submodules.

use microcad_lang_types::{
    Arguments, BinaryOperator, Identifier, Integer, List, Ty, TypeError, Value,
};
use microcad_macros::{builtin_constant, builtin_fn, builtin_mod, include_inner_docs};

use crate::{BuiltinError, BuiltinEvalContext, BuiltinItem, BuiltinResult, builtin_item};

use serde::{Deserialize, Serialize};

/// The built-in core functions.
#[builtin_mod]
pub mod core {
    use super::*;

    /// Calculate the sum of two values
    #[builtin_fn(core::add(lhs: Any, rhs: Any) -> Any)]
    pub fn add(args: Arguments, _ctx: &mut BuiltinEvalContext) -> BuiltinResult {
        let (lhs, rhs) = args.get_binary();
        Ok((lhs + rhs)?)
    }

    /// Calculate the difference of two values.
    #[builtin_fn(core::sub(lhs: Any, rhs: Any) -> Any)]
    pub fn sub(args: Arguments, _ctx: &mut BuiltinEvalContext) -> BuiltinResult {
        let (lhs, rhs) = args.get_binary();
        Ok((lhs - rhs)?)
    }

    /// Calculate the product of two values.
    #[builtin_fn(core::mul(lhs: Any, rhs: Any) -> Any)]
    pub fn mul(args: Arguments, _ctx: &mut BuiltinEvalContext) -> BuiltinResult {
        let (lhs, rhs) = args.get_binary();
        Ok((lhs * rhs)?)
    }

    /// Division of two values.
    #[builtin_fn(core::div(lhs: Any, rhs: Any) -> Any)]
    pub fn div(args: Arguments, _ctx: &mut BuiltinEvalContext) -> BuiltinResult {
        let (lhs, rhs) = args.get_binary();
        Ok((lhs / rhs)?)
    }

    /// Union of two values.
    #[builtin_fn(core::union(lhs: Any, rhs: Any) -> Any)]
    pub fn union(args: Arguments, _ctx: &mut BuiltinEvalContext) -> BuiltinResult {
        let (lhs, rhs) = args.get_binary();
        Ok((lhs | rhs)?)
    }

    /// Intersection of two values.
    #[builtin_fn(core::intersect(lhs: Any, rhs: Any) -> Any)]
    pub fn intersect(args: Arguments, _ctx: &mut BuiltinEvalContext) -> BuiltinResult {
        let (lhs, rhs) = args.get_binary();
        Ok((lhs & rhs)?)
    }

    /// Returns `true` if `lhs` is strictly greater than `rhs`.
    #[builtin_fn(core::gt(lhs: Any, rhs: Any) -> Bool)]
    pub fn gt(args: Arguments, _ctx: &mut BuiltinEvalContext) -> BuiltinResult {
        let (lhs, rhs) = args.get_binary();
        Ok(lhs.cmp(BinaryOperator::GreaterThan, &rhs)?)
    }

    /// Returns `true` if `lhs` is strictly less than `rhs`.
    #[builtin_fn(core::lt(lhs: Any, rhs: Any) -> Bool)]
    pub fn lt(args: Arguments, _ctx: &mut BuiltinEvalContext) -> BuiltinResult {
        let (lhs, rhs) = args.get_binary();
        Ok(lhs.cmp(BinaryOperator::LessThan, &rhs)?)
    }

    /// Returns `true` if `lhs` is greater than or equal to `rhs`.
    #[builtin_fn(core::ge(lhs: Any, rhs: Any) -> Bool)]
    pub fn ge(args: Arguments, _ctx: &mut BuiltinEvalContext) -> BuiltinResult {
        let (lhs, rhs) = args.get_binary();
        Ok(lhs.cmp(BinaryOperator::GreaterEqual, &rhs)?)
    }

    /// Returns `true` if `lhs` is less than or equal to `rhs`.
    #[builtin_fn(core::le(lhs: Any, rhs: Any) -> Bool)]
    pub fn le(args: Arguments, _ctx: &mut BuiltinEvalContext) -> BuiltinResult {
        let (lhs, rhs) = args.get_binary();
        Ok(lhs.cmp(BinaryOperator::LessEqual, &rhs)?)
    }

    /// Returns `true` if `lhs` and `rhs` are strictly equal.
    #[builtin_fn(core::eq(lhs: Any, rhs: Any) -> Bool)]
    pub fn eq(args: Arguments, _ctx: &mut BuiltinEvalContext) -> BuiltinResult {
        let (lhs, rhs) = args.get_binary();
        Ok(lhs.cmp(BinaryOperator::Equal, &rhs)?)
    }

    /// Returns `true` if `lhs` and `rhs` are equal within a floating-point tolerance threshold.
    #[builtin_fn(core::near(lhs: Any, rhs: Any) -> Bool)]
    pub fn near(args: Arguments, _ctx: &mut BuiltinEvalContext) -> BuiltinResult {
        let (lhs, rhs) = args.get_binary();
        Ok(lhs.cmp(BinaryOperator::Near, &rhs)?)
    }

    /// Returns `true` if `lhs` and `rhs` are not equal.
    #[builtin_fn(core::not_equal(lhs: Any, rhs: Any) -> Bool)]
    pub fn not_equal(args: Arguments, _ctx: &mut BuiltinEvalContext) -> BuiltinResult {
        let (lhs, rhs) = args.get_binary();
        Ok(lhs.cmp(BinaryOperator::NotEqual, &rhs)?)
    }

    /// Logical AND
    #[builtin_fn(core::and(lhs: Bool, rhs: Bool) -> Bool)]
    pub fn and(args: Arguments, _ctx: &mut BuiltinEvalContext) -> BuiltinResult {
        let (lhs, rhs) = args.get_binary();
        Ok((lhs & rhs)?)
    }

    /// Logical OR
    #[builtin_fn(core::or(lhs: Bool, rhs: Bool) -> Bool)]
    pub fn or(args: Arguments, _ctx: &mut BuiltinEvalContext) -> BuiltinResult {
        let (lhs, rhs) = args.get_binary();
        Ok((lhs | rhs)?)
    }

    /// Compare to values if they are greater_than
    #[builtin_fn(core::xor(lhs: Any, rhs: Any) -> Bool)]
    pub fn xor(args: Arguments, _ctx: &mut BuiltinEvalContext) -> BuiltinResult {
        let (lhs, rhs) = args.get_binary();
        Ok(lhs.pow(&rhs)?)
    }

    /// Negative value.
    #[builtin_fn(core::neg(rhs: Any) -> Any)]
    pub fn neg(args: Arguments, _ctx: &mut BuiltinEvalContext) -> BuiltinResult {
        let rhs = args.get_unary();
        Ok((-rhs)?)
    }

    /// Positive value.
    #[builtin_fn(core::plus(rhs: Any) -> Any)]
    pub fn plus(args: Arguments, _ctx: &mut BuiltinEvalContext) -> BuiltinResult {
        let rhs = args.get_unary();
        Ok(rhs)
    }

    /// Logical NOT.
    #[builtin_fn(core::not(rhs: Any) -> Any)]
    pub fn not(args: Arguments, _ctx: &mut BuiltinEvalContext) -> BuiltinResult {
        let rhs = args.get_unary();
        Ok((!rhs)?)
    }

    #[builtin_fn(core::list_access(lhs: List, index: Integer) -> Any)]
    pub fn list_access(args: Arguments, _ctx: &mut BuiltinEvalContext) -> BuiltinResult {
        let lhs: std::rc::Rc<List> = args.try_get("lhs")?;
        let index: Integer = args.try_get("index")?;
        let index = index.to_num::<usize>();

        match lhs.get(index) {
            Some(value) => Ok(value.clone()),
            None => Err(BuiltinError::BadListIndex {
                index,
                len: lhs.len(),
            }),
        }
    }

    /// Access a field in a tuple or a property of a model.
    #[builtin_fn(core::member_access(lhs: Any, name: String) -> Any)]
    pub fn member_access(args: Arguments, _ctx: &mut BuiltinEvalContext) -> BuiltinResult {
        let lhs = args.get("lhs");
        let name: String = args.try_get("name")?;

        match lhs {
            // Get field of a Tuple
            Value::Tuple(tuple) => match tuple.get_field(&Identifier::from(name.as_str())) {
                Some(value) => Ok(value.clone()),
                None => Err(TypeError::TupleHasNoField {
                    field: name,
                    ty: tuple.ty(),
                }
                .into()),
            },
            // Get an input or output property of a model tree.
            Value::Model(model) => {
                let value = model.get_property_value(&name);
                if value.is_none() {
                    Err(BuiltinError::PropertyNotFound { name: name.clone() })
                } else {
                    Ok(value)
                }
            }
            value => Err(BuiltinError::TypeError(TypeError::NoListType(value.ty()))),
        }
    }

    /// Format a string.
    #[builtin_fn(core::format(*) -> String)]
    pub fn format(args: Arguments, _ctx: &mut BuiltinEvalContext) -> BuiltinResult {
        Ok(Value::from(
            args.positional_iter()
                .map(|arg| arg.to_string())
                .collect::<Vec<_>>()
                .join(""),
        ))
    }

    /// Formats an expression as a `String` using optional field width and decimal precision specifiers.
    ///
    /// Accepts a `width` (minimum character output width, right-aligned) and a `precision`
    /// (number of decimal places for floating-point values/lengths or maximum characters for strings).
    /// Passing a negative integer for `width` or `precision` disables that specifier.
    ///
    /// ## Parameters
    /// - `expr`: The value to format into a string.
    /// - `width`: Minimum total character width (padded with spaces on the left if positive).
    /// - `precision`: Maximum decimal places or string character limit.
    #[builtin_fn(core::format_spec(expr: Any, width: Integer, precision: Integer) -> String)]
    pub fn format_spec(args: Arguments, _ctx: &mut BuiltinEvalContext) -> BuiltinResult {
        use std::fmt::Write;

        let expr = args.get("expr");
        let width: i64 = args.try_get("width")?;
        let precision: i64 = args.try_get("precision")?;

        let mut formatted = String::new();

        // Dynamically apply precision and width using standard format specifiers
        match (width, precision) {
            (w, p) if w >= 0 && p >= 0 => {
                write!(
                    formatted,
                    "{:width$.precision$}",
                    expr,
                    width = w as usize,
                    precision = p as usize
                )?;
            }
            (w, _) if w >= 0 => {
                write!(formatted, "{:width$}", expr, width = w as usize)?;
            }
            (_, p) if p >= 0 => {
                write!(formatted, "{:.precision$}", expr, precision = p as usize)?;
            }
            _ => {
                write!(formatted, "{}", expr)?;
            }
        }

        Ok(formatted.into())
    }

    /// Generates a list containing a sequence of integers from `start` to `end` (inclusive).
    ///
    /// If `start` is greater than `end`, an empty array is returned.
    ///
    /// ## Parameters
    /// - `start`: The beginning integer value of the sequence.
    /// - `end`: The ending integer value of the sequence (included in output).
    ///
    /// ## Examples
    /// ```text
    /// range(1, 5)  // -> [1, 2, 3, 4, 5]
    /// range(0, 0)  // -> [0]
    /// range(5, 1)  // -> []
    /// ```
    #[builtin_fn(core::range(start: Integer, end: Integer) -> Any)] // TODO: Return [Integer]
    pub fn range(args: Arguments, _ctx: &mut BuiltinEvalContext) -> BuiltinResult {
        let start: i64 = args.try_get("start")?;
        let end: i64 = args.try_get("end")?;
        let list = List::from_iter((start..=end).map(Value::from));
        Ok(list.into())
    }

    /// Construct a list.
    #[builtin_fn(core::list(*) -> Any)]
    pub fn list(args: Arguments, _ctx: &mut BuiltinEvalContext) -> BuiltinResult {
        Ok(List::from_iter(args.positional_iter().cloned()).into())
    }

    /// Construct a tuple.
    #[builtin_fn(core::tuple(*) -> Any)]
    pub fn tuple(args: Arguments, _ctx: &mut BuiltinEvalContext) -> BuiltinResult {
        Ok(args.0.into())
    }

    /// Access the attributes in a model tree.
    #[builtin_fn(core::attribute_access(lhs: Any, name: String) -> Any)]
    pub fn attribute_access(args: Arguments, _ctx: &mut BuiltinEvalContext) -> BuiltinResult {
        use microcad_lang_types::model::AttributeAccess;

        let lhs = args.get("lhs");
        let name: String = args.try_get("name")?;

        match lhs {
            // Get an input or output property of a model tree.
            Value::Model(model) => match model.get_attribute(&name) {
                Some(attr) => Ok(attr.clone()),
                None => Err(BuiltinError::PropertyNotFound { name: name.clone() }),
            },
            value => Err(BuiltinError::TypeError(TypeError::NoListType(value.ty()))),
        }
    }
}

#[builtin_mod]
pub mod debug {
    use microcad_lang_base::PushIssue;

    use crate::{BuiltinAdvice, diag::BuiltinWarning};

    use super::*;

    #[builtin_fn(debug::assert(cond: Bool, message: String))]
    pub fn assert(args: Arguments, _ctx: &mut BuiltinEvalContext) -> BuiltinResult {
        let cond = args.get_cond()?;
        let message: String = args.try_get("message")?;

        if cond {
            // assertion ok: return None.
            Ok(Value::None)
        } else {
            // assertion failed: stop eval and return Err.
            Err(BuiltinError::AssertionFailed(message))
        }
    }

    #[builtin_fn(debug::expect(cond: Bool, message: String))]
    pub fn expect(args: Arguments, ctx: &mut BuiltinEvalContext) -> BuiltinResult {
        let cond = args.get_cond()?;
        let message: String = args.try_get("message")?;
        if !cond {
            ctx.push_err(BuiltinError::Expected(message));
        }
        Ok(Value::None)
    }

    #[builtin_fn(debug::panic(message: String))]
    pub fn panic(args: Arguments, _ctx: &mut BuiltinEvalContext) -> BuiltinResult {
        let message: String = args.try_get("message")?;
        Err(BuiltinError::Panic(message))
    }

    #[builtin_fn(debug::error(message: String))]
    pub fn error(args: Arguments, ctx: &mut BuiltinEvalContext) -> BuiltinResult {
        let message: String = args.try_get("message")?;
        ctx.push_err(BuiltinError::Custom(message));
        Ok(Value::None)
    }

    #[builtin_fn(debug::warning(message: String))]
    pub fn warning(args: Arguments, ctx: &mut BuiltinEvalContext) -> BuiltinResult {
        let message: String = args.try_get("message")?;
        ctx.push_warn(BuiltinWarning::Custom(message));
        Ok(Value::None)
    }

    #[builtin_fn(debug::info(message: String))]
    pub fn info(args: Arguments, ctx: &mut BuiltinEvalContext) -> BuiltinResult {
        let message: String = args.try_get("message")?;
        ctx.push_info(BuiltinAdvice::Custom(message));
        Ok(Value::None)
    }

    /// Print all variables
    #[builtin_fn(debug::print(*))]
    pub fn print(args: Arguments, _ctx: &mut BuiltinEvalContext) -> BuiltinResult {
        println!(
            "{}",
            args.positional_iter()
                .map(|arg| arg.to_string())
                .chain(args.named_iter().map(|(id, arg)| format!("{id} = {arg}")))
                .collect::<Vec<_>>()
                .join(", ")
        );
        Ok(Value::None)
    }
}

/// Built-in math functions.
#[builtin_mod]
pub mod math {
    use super::*;
    use microcad_lang_types::{Angle, MathOps, Scalar, tuple};

    /// Pi
    #[builtin_constant(math::PI)]
    pub static PI: BuiltinItem = std::f64::consts::PI;

    #[builtin_constant(math::X)]
    pub static X: BuiltinItem = tuple!(x = 1.0, y = 0.0, z = 0.0);

    #[builtin_constant(math::Y)]
    pub static Y: BuiltinItem = tuple!(x = 0.0, y = 1.0, z = 0.0);

    #[builtin_constant(math::Z)]
    pub static Z: BuiltinItem = tuple!(x = 0.0, y = 0.0, z = 1.0);

    #[builtin_fn(math::abs(x: Any) -> Any)]
    pub fn abs(args: Arguments, _ctx: &mut BuiltinEvalContext) -> BuiltinResult {
        let x = args.get("x");
        Ok(x.abs()?)
    }

    #[builtin_fn(math::sqrt(x: Any) -> Any)]
    pub fn sqrt(args: Arguments, _ctx: &mut BuiltinEvalContext) -> BuiltinResult {
        let x = args.get("x");
        Ok(x.sqrt()?)
    }

    #[builtin_fn(math::sin(x: Any) -> Any)]
    pub fn sin(args: Arguments, _ctx: &mut BuiltinEvalContext) -> BuiltinResult {
        let x = args.get("x");
        Ok(x.sin()?)
    }

    #[builtin_fn(math::cos(x: Any) -> Any)]
    pub fn cos(args: Arguments, _ctx: &mut BuiltinEvalContext) -> BuiltinResult {
        let x = args.get("x");
        Ok(x.cos()?)
    }

    #[builtin_fn(math::int(x: Any) -> Any)]
    pub fn int(args: Arguments, _ctx: &mut BuiltinEvalContext) -> BuiltinResult {
        let x = args.get("x");
        Ok(x.int()?)
    }

    #[builtin_fn(math::floor(x: Any) -> Any)]
    pub fn floor(args: Arguments, _ctx: &mut BuiltinEvalContext) -> BuiltinResult {
        let x = args.get("x");
        Ok(x.floor()?)
    }

    #[builtin_fn(math::ceil(x: Any) -> Any)]
    pub fn ceil(args: Arguments, _ctx: &mut BuiltinEvalContext) -> BuiltinResult {
        let x = args.get("x");
        Ok(x.ceil()?)
    }

    #[builtin_fn(math::round(x: Any) -> Any)]
    pub fn round(args: Arguments, _ctx: &mut BuiltinEvalContext) -> BuiltinResult {
        let x = args.get("x");
        Ok(x.round()?)
    }

    #[builtin_fn(math::fract(x: Any) -> Any)]
    pub fn fract(args: Arguments, _ctx: &mut BuiltinEvalContext) -> BuiltinResult {
        let x = args.get("x");
        Ok(x.fract()?)
    }

    #[builtin_fn(math::rotate_around_axis(angle: Angle, x: Scalar, y: Scalar, z: Scalar) -> Mat3)]
    pub fn rotate_around_axis(args: Arguments, _ctx: &mut BuiltinEvalContext) -> BuiltinResult {
        let angle: Angle = args.try_get("angle")?;
        let x: Scalar = args.try_get("x")?;
        let y: Scalar = args.try_get("y")?;
        let z: Scalar = args.try_get("z")?;
        Ok(microcad_lang_types::math::rotate_around_axis(angle, x, y, z).into())
    }

    #[builtin_fn(math::rotate_xyz(x: Angle, y: Angle, z: Angle) -> Mat3)]
    pub fn rotate_xyz(args: Arguments, _ctx: &mut BuiltinEvalContext) -> BuiltinResult {
        let x: Angle = args.try_get("x")?;
        let y: Angle = args.try_get("y")?;
        let z: Angle = args.try_get("z")?;
        Ok(microcad_lang_types::math::rotate_xyz(x, y, z).into())
    }

    #[builtin_fn(math::rotate_zyx(x: Angle, y: Angle, z: Angle) -> Mat3)]
    pub fn rotate_zyx(args: Arguments, _ctx: &mut BuiltinEvalContext) -> BuiltinResult {
        let x: Angle = args.try_get("x")?;
        let y: Angle = args.try_get("y")?;
        let z: Angle = args.try_get("z")?;
        Ok(microcad_lang_types::math::rotate_zyx(z, y, x).into())
    }
}

/// Built-in string functions
#[builtin_mod]
pub mod string {
    use super::*;

    #[builtin_fn(string::len(s: String) -> Integer)]
    pub fn len(args: Arguments, _ctx: &mut BuiltinEvalContext) -> BuiltinResult {
        let s: String = args.try_get("s")?;
        Ok(s.len().into())
    }
}

/// Built-in list functions.
#[builtin_mod]
pub mod list {
    use std::rc::Rc;

    use super::*;

    #[builtin_fn(list::count(a: List) -> Integer)]
    pub fn count(args: Arguments, _ctx: &mut BuiltinEvalContext) -> BuiltinResult {
        let a: Rc<List> = args.try_get("a")?;
        Ok(a.len().into())
    }

    #[builtin_fn(list::first(a: List) -> Any)]
    pub fn first(args: Arguments, _ctx: &mut BuiltinEvalContext) -> BuiltinResult {
        let a: Rc<List> = args.try_get("a")?;
        Ok(a.first())
    }

    #[builtin_fn(list::last(a: List) -> Any)]
    pub fn last(args: Arguments, _ctx: &mut BuiltinEvalContext) -> BuiltinResult {
        let a: Rc<List> = args.try_get("a")?;
        Ok(a.last())
    }

    #[builtin_fn(list::rev(a: List) -> List)]
    pub fn rev(args: Arguments, _ctx: &mut BuiltinEvalContext) -> BuiltinResult {
        let a: Rc<List> = args.try_get("a")?;
        Ok(a.rev().into())
    }

    #[builtin_fn(list::sorted(a: List) -> List)]
    pub fn sorted(args: Arguments, _ctx: &mut BuiltinEvalContext) -> BuiltinResult {
        let a: Rc<List> = args.try_get("a")?;
        Ok(a.sorted().into())
    }

    #[builtin_fn(list::head(a: List, n: Integer) -> List)]
    pub fn head(args: Arguments, _ctx: &mut BuiltinEvalContext) -> BuiltinResult {
        let a: Rc<List> = args.try_get("a")?;
        let n: Integer = args.try_get("n")?;
        Ok(a.head(n).into())
    }

    #[builtin_fn(list::tail(a: List, n: Integer) -> List)]
    pub fn tail(args: Arguments, _ctx: &mut BuiltinEvalContext) -> BuiltinResult {
        let a: Rc<List> = args.try_get("a")?;
        let n: Integer = args.try_get("n")?;
        Ok(a.tail(n).into())
    }

    #[builtin_fn(list::contains(a: List, v: Any) -> Bool)]
    pub fn contains(args: Arguments, _ctx: &mut BuiltinEvalContext) -> BuiltinResult {
        let a: Rc<List> = args.try_get("a")?;
        let v = args.get("v");
        Ok(a.contains(v).into())
    }

    #[builtin_fn(list::all_equal(a: List) -> Bool)]
    pub fn all_equal(args: Arguments, _ctx: &mut BuiltinEvalContext) -> BuiltinResult {
        let a: Rc<List> = args.try_get("a")?;
        Ok(a.all_equal().into())
    }

    #[builtin_fn(list::is_ascending(a: List) -> Bool)]
    pub fn is_ascending(args: Arguments, _ctx: &mut BuiltinEvalContext) -> BuiltinResult {
        let a: Rc<List> = args.try_get("a")?;
        Ok(a.is_ascending().into())
    }

    #[builtin_fn(list::is_descending(a: List) -> Bool)]
    pub fn is_descending(args: Arguments, _ctx: &mut BuiltinEvalContext) -> BuiltinResult {
        let a: Rc<List> = args.try_get("a")?;
        Ok(a.is_descending().into())
    }
}

/// Built-in color functions.
#[builtin_mod]
pub mod color {
    use super::*;

    use microcad_lang_types::{Scalar, tuple};

    #[builtin_fn(color::rgb(r: Scalar, g: Scalar, b: Scalar) -> Color)]
    pub fn rgb(args: Arguments, _ctx: &mut BuiltinEvalContext) -> BuiltinResult {
        let r: Scalar = args.try_get("r")?;
        let g: Scalar = args.try_get("g")?;
        let b: Scalar = args.try_get("b")?;
        Ok(tuple!(r = r, g = g, b = b).into())
    }
}

/// Built-in 2D primitives
#[builtin_mod]
pub mod geo2d {
    use microcad_lang_types::Length;
    use microcad_macros::builtin_primitive;

    use super::*;

    /// A circle with a radius.
    #[derive(Serialize, Deserialize, Debug)]
    #[builtin_primitive(Geometry2D)]
    pub struct Circle {
        /// Radius in mm.
        pub radius: Length,
    }

    /// A rectangle.
    #[derive(Serialize, Deserialize, Debug)]
    #[builtin_primitive(Geometry2D)]
    pub struct Rect {
        /// Width of the rectangle.
        pub width: Length,
        /// Height of the rectangle.
        pub height: Length,
        /// X position (left side) of the rectangle.
        pub x: Length,
        /// Y position (bottom side) of the rectangle.
        pub y: Length,
    }
}

/// Built-in Operations
#[builtin_mod]
pub mod ops {
    use microcad_lang_types::{
        Angle, Integer, Length, Mat3, ModelTree, Scalar, Vec3, math::AffineTransform,
    };
    use microcad_macros::{builtin_op, builtin_transform};

    use super::*;

    /// Translate a model.
    #[builtin_transform]
    pub struct Translate {
        /// Translation in X direction.
        pub x: Length,
        /// Translation in Y direction.
        pub y: Length,
        /// Translation in Z direction.
        pub z: Length,
    }

    impl From<Translate> for AffineTransform {
        fn from(Translate { x, y, z }: Translate) -> Self {
            AffineTransform::Translation { x, y, z }
        }
    }

    /// Translate a model by `x`, `y` and `z`.
    #[builtin_transform]
    pub fn translate(
        args: Arguments,
        ctx: &mut BuiltinEvalContext,
    ) -> Result<ModelTree, BuiltinError>;

    /// Rotate a model by a rotation matrix.
    #[builtin_transform]
    pub struct Rotate {
        /// Rotation matrix.
        pub matrix: Mat3,
    }

    impl From<Rotate> for AffineTransform {
        fn from(Rotate { matrix }: Rotate) -> Self {
            AffineTransform::RotateMatrix { m: matrix }
        }
    }

    /// Rotate a model by a rotation matrix.
    #[builtin_transform]
    pub fn rotate(args: Arguments, ctx: &mut BuiltinEvalContext)
    -> Result<ModelTree, BuiltinError>;

    /// Subtract all following child models from the first one.
    #[builtin_op(Any -> Any)]
    pub struct Difference;

    /// Subtract all following child models from the first one.
    #[builtin_op]
    pub fn difference(
        args: Arguments,
        ctx: &mut BuiltinEvalContext,
    ) -> Result<ModelTree, BuiltinError>;

    /// Linear extrusion of a 2D model.
    #[builtin_op(Geometry2D -> Geometry3D)]
    pub struct Extrude {
        /// Height in mm.
        pub height: Length,
        /// Scale of the top in X direction.
        pub scale_x: Scalar,
        /// Scale of the top in Y direction.
        pub scale_y: Scalar,
        /// Twist angle of the top.
        pub twist: Angle,
    }

    /// Linear extrusion of a 2D model.
    #[builtin_op]
    pub fn extrude(
        args: Arguments,
        ctx: &mut BuiltinEvalContext,
    ) -> Result<ModelTree, BuiltinError>;

    /// Scale a model.
    #[builtin_transform]
    pub struct Scale {
        /// Scale factor in X direction.
        pub x: Scalar,
        /// Scale factor in Y direction.
        pub y: Scalar,
        /// Scale factor in Z direction.
        pub z: Scalar,
    }

    impl From<Scale> for AffineTransform {
        fn from(Scale { x, y, z }: Scale) -> Self {
            AffineTransform::Scale(Vec3::new(x, y, z))
        }
    }

    /// Scale a model by `x`, `y` and `z`.
    #[builtin_transform]
    pub fn scale(args: Arguments, ctx: &mut BuiltinEvalContext) -> Result<ModelTree, BuiltinError>;

    /// Orient a model, so that its Z axis points towards a vector.
    #[builtin_transform]
    pub struct Orient {
        /// X component of the vector.
        pub x: Scalar,
        /// Y component of the vector.
        pub y: Scalar,
        /// Z component of the vector.
        pub z: Scalar,
    }

    impl From<Orient> for AffineTransform {
        fn from(Orient { x, y, z }: Orient) -> Self {
            AffineTransform::RotateMatrix {
                m: microcad_lang_types::math::orient_to_z(Vec3::new(x, y, z)),
            }
        }
    }

    /// Orient a model, so that its Z axis points towards the vector `(x, y, z)`.
    #[builtin_transform]
    pub fn orient(args: Arguments, ctx: &mut BuiltinEvalContext)
    -> Result<ModelTree, BuiltinError>;

    /// Union of all child models.
    #[builtin_op(Any -> Any)]
    pub struct Union;

    /// Union of all child models.
    #[builtin_op]
    pub fn union(args: Arguments, ctx: &mut BuiltinEvalContext) -> Result<ModelTree, BuiltinError>;

    /// Subtract all following child models from the first one.
    #[builtin_op(Any -> Any)]
    pub struct Subtract;

    /// Subtract all following child models from the first one.
    #[builtin_op]
    pub fn subtract(
        args: Arguments,
        ctx: &mut BuiltinEvalContext,
    ) -> Result<ModelTree, BuiltinError>;

    /// Intersection of all child models.
    #[builtin_op(Any -> Any)]
    pub struct Intersect;

    /// Intersection of all child models.
    #[builtin_op]
    pub fn intersect(
        args: Arguments,
        ctx: &mut BuiltinEvalContext,
    ) -> Result<ModelTree, BuiltinError>;

    /// Combine each child model with each other child model.
    #[builtin_op(Any -> Any)]
    pub struct Multiply;

    /// Combine each child model with each other child model.
    #[builtin_op]
    pub fn multiply(
        args: Arguments,
        ctx: &mut BuiltinEvalContext,
    ) -> Result<ModelTree, BuiltinError>;

    /// Mirror a model at a plane (3D) or line (2D), keeping the original.
    #[builtin_op(Any -> Any)]
    pub struct Mirror {
        /// X position of a point on the mirror plane.
        pub x: Length,
        /// Y position of a point on the mirror plane.
        pub y: Length,
        /// Z position of a point on the mirror plane.
        pub z: Length,
        /// X component of the plane's normal.
        pub nx: Scalar,
        /// Y component of the plane's normal.
        pub ny: Scalar,
        /// Z component of the plane's normal.
        pub nz: Scalar,
    }

    /// Mirror a model at a plane (3D) or line (2D), keeping the original.
    #[builtin_op]
    pub fn mirror(args: Arguments, ctx: &mut BuiltinEvalContext)
    -> Result<ModelTree, BuiltinError>;

    /// Reflect a model at a plane (3D) or line (2D).
    #[builtin_op(Any -> Any)]
    pub struct Reflect {
        /// X position of a point on the reflection plane.
        pub x: Length,
        /// Y position of a point on the reflection plane.
        pub y: Length,
        /// Z position of a point on the reflection plane.
        pub z: Length,
        /// X component of the plane's normal.
        pub nx: Scalar,
        /// Y component of the plane's normal.
        pub ny: Scalar,
        /// Z component of the plane's normal.
        pub nz: Scalar,
    }

    /// Reflect a model at a plane (3D) or line (2D).
    #[builtin_op]
    pub fn reflect(
        args: Arguments,
        ctx: &mut BuiltinEvalContext,
    ) -> Result<ModelTree, BuiltinError>;

    /// Revolve a 2D model around the Y axis.
    #[builtin_op(Geometry2D -> Geometry3D)]
    pub struct Revolve {
        /// Revolve angle.
        pub angle: Angle,
    }

    /// Revolve a 2D model around the Y axis.
    #[builtin_op]
    pub fn revolve(
        args: Arguments,
        ctx: &mut BuiltinEvalContext,
    ) -> Result<ModelTree, BuiltinError>;

    /// Create a spiral from a 2D model.
    #[builtin_op(Geometry2D -> Geometry3D)]
    pub struct Spiralize {
        /// Height of the spiral.
        pub height: Length,
        /// Inner radius of the spiral.
        pub inner_radius: Length,
        /// Outer radius of the spiral.
        pub outer_radius: Length,
        /// Angle of the spiral.
        pub angle: Angle,
    }

    /// Create a spiral from a 2D model.
    #[builtin_op]
    pub fn spiralize(
        args: Arguments,
        ctx: &mut BuiltinEvalContext,
    ) -> Result<ModelTree, BuiltinError>;

    /// Move a model to the origin.
    #[builtin_op(Any -> Any)]
    pub struct Center;

    /// Move a model to the origin.
    #[builtin_op]
    pub fn center(args: Arguments, ctx: &mut BuiltinEvalContext)
    -> Result<ModelTree, BuiltinError>;

    /// Align a collection of models along a direction with spacing.
    #[builtin_op(Any -> Any)]
    pub struct Align {
        /// X component of the direction.
        pub x: Scalar,
        /// Y component of the direction.
        pub y: Scalar,
        /// Z component of the direction.
        pub z: Scalar,
        /// Spacing between the models.
        pub spacing: Length,
    }

    /// Align a collection of models along a direction with spacing.
    #[builtin_op]
    pub fn align(args: Arguments, ctx: &mut BuiltinEvalContext) -> Result<ModelTree, BuiltinError>;

    /// Distribute models in a rectangular grid.
    #[builtin_op(Any -> Any)]
    pub struct DistributeGrid {
        /// X position of the grid.
        pub x: Length,
        /// Y position of the grid.
        pub y: Length,
        /// Width of the grid.
        pub width: Length,
        /// Height of the grid.
        pub height: Length,
        /// Number of rows.
        pub rows: Integer,
        /// Number of columns.
        pub columns: Integer,
    }

    /// Distribute models in a rectangular grid.
    #[builtin_op]
    pub fn distribute_grid(
        args: Arguments,
        ctx: &mut BuiltinEvalContext,
    ) -> Result<ModelTree, BuiltinError>;

    /// Convex hull of a model.
    #[builtin_op(Any -> Any)]
    pub struct Hull;

    /// Convex hull of a model.
    #[builtin_op]
    pub fn hull(args: Arguments, ctx: &mut BuiltinEvalContext) -> Result<ModelTree, BuiltinError>;

    /// Offset the boundary of a 2D model by a distance.
    #[builtin_op(Geometry2D -> Geometry2D)]
    pub struct Buffer {
        /// Offset distance, negative values shrink the model.
        pub distance: Length,
    }

    /// Offset the boundary of a 2D model by a distance.
    #[builtin_op]
    pub fn buffer(args: Arguments, ctx: &mut BuiltinEvalContext)
    -> Result<ModelTree, BuiltinError>;
}

pub static __MU: &str = include_inner_docs!("src/mu.rs");
