// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Argument match tests

use microcad_lang_base::Identifier;
use microcad_lang_eval::{ArgumentMatch, MatchPriority};
use microcad_lang_types::{
    ArgumentValue, ArgumentValueList, Arguments, CallSignature, Length, Tuple, Type, Value,
    argument_value, arguments, call_signature, list, tuple,
};
use test_case::test_case;

/// A call signature with default values.
struct Signature {
    signature: CallSignature,
    default: Tuple,
}

impl ArgumentMatch for Signature {
    fn call_signature(&self) -> CallSignature {
        self.signature.clone()
    }

    fn default_values(&self) -> Tuple {
        self.default.clone()
    }
}

/// `(radius: Length)`
fn circle() -> Signature {
    Signature {
        signature: call_signature!(radius: Type::length()),
        default: tuple!(),
    }
}

/// `(r: Length, radius: Length)`
fn r_and_radius() -> Signature {
    Signature {
        signature: call_signature!(r: Type::length(), radius: Type::length()),
        default: tuple!(),
    }
}

/// `(foo: Integer, foz: Integer)`
fn foo_foz() -> Signature {
    Signature {
        signature: call_signature!(foo: Type::Integer, foz: Type::Integer),
        default: tuple!(),
    }
}

/// `(a: Scalar, b: Length)`
fn scalar_length() -> Signature {
    Signature {
        signature: call_signature!(a: Type::scalar(), b: Type::length()),
        default: tuple!(),
    }
}

/// `(any: Any, length: Length)`
fn any_length() -> Signature {
    Signature {
        signature: call_signature!(any: Type::Any, length: Type::length()),
        default: tuple!(),
    }
}

/// `(x: Scalar)`
fn scalar() -> Signature {
    Signature {
        signature: call_signature!(x: Type::scalar()),
        default: tuple!(),
    }
}

/// `(cond: Bool, message: String = "default")`
fn assert() -> Signature {
    Signature {
        signature: call_signature!(cond: Type::Bool, message: Type::String),
        default: tuple!(message = "default"),
    }
}

/// `(a: Any, b: Any, message: String = "default")`
fn assert_eq() -> Signature {
    Signature {
        signature: call_signature!(a: Type::Any, b: Type::Any, message: Type::String),
        default: tuple!(message = "default"),
    }
}

/// `(x: Integer, y: Integer)`
fn x_y() -> Signature {
    Signature {
        signature: call_signature!(x: Type::Integer, y: Type::Integer),
        default: tuple!(),
    }
}

/// `(x: Integer, y: Integer = 0)`
fn x_y_default() -> Signature {
    Signature {
        signature: call_signature!(x: Type::Integer, y: Type::Integer),
        default: tuple!(y = 0),
    }
}

/// `(*)`
fn variadic() -> Signature {
    Signature {
        signature: call_signature!(*),
        default: tuple!(),
    }
}

fn mm(value: f64) -> Value {
    Length::mm(value).into()
}

/// Match `arguments`, errors are compared by their message.
fn argument_match(
    signature: Signature,
    arguments: Vec<ArgumentValue>,
) -> Result<Arguments, String> {
    signature
        .argument_match(&arguments.into_iter().collect())
        .map_err(|err| err.to_string())
}

// Named arguments
#[test_case(circle(), vec![argument_value!(radius = mm(42.0))]
    => Ok(arguments!(radius = mm(42.0))); "named")]
#[test_case(scalar_length(), vec![argument_value!(b = mm(2.0)), argument_value!(a = 1.0)]
    => Ok(arguments!(a = 1.0, b = mm(2.0))); "named in any order")]
#[test_case(circle(), vec![argument_value!(diameter = mm(42.0))]
    => Err("Unexpected arguments: diameter".into()); "named unexpected")]
// Abbreviations
#[test_case(circle(), vec![argument_value!(r = mm(42.0))]
    => Ok(arguments!(radius = mm(42.0))); "abbreviation")]
#[test_case(r_and_radius(), vec![argument_value!(r = mm(1.0)), argument_value!(radius = mm(2.0))]
    => Ok(arguments!(r = mm(1.0), radius = mm(2.0))); "abbreviation is no exact name")]
#[test_case(foo_foz(), vec![argument_value!(f = 10), argument_value!(foz = 1)]
    => Err("Argument `f` is ambiguous: foo,foz".into()); "abbreviation ambiguous")]
#[test_case(circle(), vec![argument_value!(d = mm(42.0))]
    => Err("Unexpected arguments: d".into()); "abbreviation unknown")]
#[test_case(circle(), vec![argument_value!(ra = mm(42.0))]
    => Err("Unexpected arguments: ra".into()); "abbreviation only a letter")]
#[test_case(circle(), vec![argument_value!(r = mm(1.0)), argument_value!(radius = mm(2.0))]
    => Err("Argument `radius` is given more than once".into()); "abbreviation and name")]
// Positional arguments
#[test_case(circle(), vec![argument_value!(mm(42.0))]
    => Ok(arguments!(radius = mm(42.0))); "positional")]
#[test_case(scalar_length(), vec![argument_value!(mm(2.0)), argument_value!(1.0)]
    => Ok(arguments!(a = 1.0, b = mm(2.0))); "positional by type")]
#[test_case(scalar_length(), vec![argument_value!(b = mm(2.0)), argument_value!(1.0)]
    => Ok(arguments!(a = 1.0, b = mm(2.0))); "positional and named")]
#[test_case(any_length(), vec![argument_value!(mm(2.0)), argument_value!(1.0)]
    => Ok(arguments!(any = 1.0, length = mm(2.0))); "positional exact type before any")]
#[test_case(any_length(), vec![argument_value!(length = mm(2.0)), argument_value!(mm(3.0))]
    => Ok(arguments!(any = mm(3.0), length = mm(2.0))); "positional any")]
#[test_case(scalar(), vec![argument_value!(1)]
    => Ok(arguments!(x = 1)); "positional integer for scalar")]
// Default values
#[test_case(assert(), vec![argument_value!(true)]
    => Ok(arguments!(cond = true, message = "default")); "default")]
#[test_case(assert(), vec![argument_value!(true), argument_value!(message = "given")]
    => Ok(arguments!(cond = true, message = "given")); "default overridden by name")]
#[test_case(assert(), vec![argument_value!(true), argument_value!(m = "given")]
    => Ok(arguments!(cond = true, message = "given")); "default overridden by abbreviation")]
#[test_case(assert(), vec![argument_value!(true), argument_value!("given")]
    => Ok(arguments!(cond = true, message = "given")); "default overridden positionally")]
#[test_case(assert_eq(), vec![argument_value!("x"), argument_value!("y")]
    => Ok(arguments!(a = "x", b = "y", message = "default")); "default after required")]
#[test_case(assert_eq(), vec![argument_value!("x"), argument_value!("y"), argument_value!("z")]
    => Ok(arguments!(a = "x", b = "y", message = "z")); "default given positionally last")]
// Missing arguments
#[test_case(circle(), vec![]
    => Err("Missing arguments: radius".into()); "missing")]
#[test_case(assert(), vec![argument_value!(message = "given")]
    => Err("Missing arguments: cond".into()); "missing without default")]
// Inline identifiers, e.g. `r` in `f(r * 2)`
#[test_case(x_y(), vec![ArgumentValue::inline("y", 2), argument_value!(1)]
    => Ok(arguments!(x = 1, y = 2)); "inline identifier before type")]
#[test_case(assert_eq(), vec![ArgumentValue::inline("v", mm(20.0)), argument_value!(mm(20.0))]
    => Ok(arguments!(a = mm(20.0), b = mm(20.0), message = "default")); "inline identifier without parameter")]
#[test_case(circle(), vec![ArgumentValue::inline("r", mm(1.0))]
    => Ok(arguments!(radius = mm(1.0))); "inline short name")]
#[test_case(foo_foz(), vec![ArgumentValue::inline("f", 10), argument_value!(foz = 1)]
    => Ok(arguments!(foo = 10, foz = 1)); "ambiguous inline short name by type")]
#[test_case(circle(), vec![ArgumentValue::inline("radius", 1.0)]
    => Ok(arguments!(radius = 1.0)); "inline identifier type checked later")]
#[test_case(x_y(), vec![ArgumentValue::inline("x", 1), ArgumentValue::inline("x", 2)]
    => Ok(arguments!(x = 1, y = 2)); "same inline identifier twice")]
// Positional arguments without matching parameter
#[test_case(circle(), vec![argument_value!(1.0)]
    => Err("Unexpected positional arguments: 1 (Scalar)".into()); "unexpected positional of other type")]
#[test_case(circle(), vec![argument_value!(mm(1.0)), argument_value!(mm(2.0))]
    => Err("Unexpected positional arguments: 2mm (Length)".into()); "unexpected positional surplus")]
#[test_case(circle(), vec![argument_value!(radius = mm(1.0)), argument_value!(mm(2.0))]
    => Err("Unexpected positional arguments: 2mm (Length)".into()); "unexpected positional for named")]
fn match_arguments(
    signature: Signature,
    arguments: Vec<ArgumentValue>,
) -> Result<Arguments, String> {
    argument_match(signature, arguments)
}

/// A variadic signature returns all arguments.
#[test]
fn match_variadic() {
    let arguments = vec![argument_value!(1.0), argument_value!(a = 2)];
    assert_eq!(
        argument_match(variadic(), arguments),
        Ok(Arguments::from(Tuple {
            positional: vec![1.0.into()],
            named: vec![(Identifier::no_ref("a"), 2.into())],
        }))
    );
}

/// A variadic signature names arguments with inline identifiers.
#[test]
fn match_variadic_inline() {
    let arguments = vec![ArgumentValue::inline("x", 1.0)];
    assert_eq!(
        argument_match(variadic(), arguments),
        Ok(Arguments::from(Tuple {
            positional: vec![],
            named: vec![(Identifier::no_ref("x"), 1.0.into())],
        }))
    );
}

/// A list argument for a non-list parameter results in one match per element.
#[test]
fn multi_match_cartesian_product() {
    let signature = Signature {
        signature: call_signature!(a: Type::length(), b: Type::length()),
        default: tuple!(),
    };

    // f(a = [1mm, 2mm], b = [10mm, 20mm, 30mm])
    let args: ArgumentValueList = [
        ArgumentValue::named("a", list![mm(1.0), mm(2.0)]),
        ArgumentValue::named("b", list![mm(10.0), mm(20.0), mm(30.0)]),
    ]
    .into_iter()
    .collect();

    let result = signature
        .argument_multi_match(&args)
        .expect("A multi match");

    // 2 x 3 = 6 combinations expected
    assert_eq!(
        result,
        [
            arguments!(a = mm(1.0), b = mm(10.0)),
            arguments!(a = mm(1.0), b = mm(20.0)),
            arguments!(a = mm(1.0), b = mm(30.0)),
            arguments!(a = mm(2.0), b = mm(10.0)),
            arguments!(a = mm(2.0), b = mm(20.0)),
            arguments!(a = mm(2.0), b = mm(30.0)),
        ]
    );
}

/// A positional list argument for a parameter of its element type results in one match per element.
#[test]
fn multi_match_positional() {
    let args: ArgumentValueList = [argument_value!(list![-1, 0, 2])].into_iter().collect();

    assert_eq!(
        x_y_default()
            .argument_multi_match(&args)
            .expect("A multi match"),
        [
            arguments!(x = -1, y = 0),
            arguments!(x = 0, y = 0),
            arguments!(x = 2, y = 0),
        ]
    );
}

/// A list argument for a list parameter is no multiplicity.
#[test]
fn multi_match_list_parameter() {
    let signature = Signature {
        signature: call_signature!(x: Type::List(Box::new(Type::Integer))),
        default: tuple!(),
    };
    let args: ArgumentValueList = [argument_value!(list![1, 2])].into_iter().collect();

    assert_eq!(
        signature
            .argument_multi_match(&args)
            .expect("A multi match"),
        [arguments!(x = list![1, 2])]
    );
}

/// A multi match with a variadic signature is an error.
#[test]
fn multi_match_variadic() {
    let args: ArgumentValueList = [argument_value!(1.0)].into_iter().collect();
    assert!(variadic().argument_multi_match(&args).is_err());
}

/// Inline identifiers matching a parameter by name have priority over matching by type,
/// e.g. to choose between overloaded initializers.
#[test]
fn match_priority() {
    let diameter = Signature {
        signature: call_signature!(diameter: Type::length()),
        default: tuple!(),
    };
    let arguments: ArgumentValueList = [ArgumentValue::inline("radius", mm(1.0))]
        .into_iter()
        .collect();

    let by_name = circle().match_priority(&arguments);
    let by_type = diameter.match_priority(&arguments);
    assert_eq!(
        by_name,
        Some(MatchPriority {
            exact_names: 1,
            short_names: 0
        })
    );
    assert_eq!(by_type, Some(MatchPriority::default()));
    assert!(by_name > by_type);
    assert_eq!(foo_foz().match_priority(&arguments), None);
}

/// A short name matches before a type, e.g. `r` in `Circle(r)` is the radius, not the diameter.
#[test]
fn match_priority_short_name() {
    let diameter = Signature {
        signature: call_signature!(diameter: Type::length()),
        default: tuple!(),
    };
    let arguments: ArgumentValueList = [ArgumentValue::inline("r", mm(1.0))].into_iter().collect();

    assert!(circle().match_priority(&arguments) > diameter.match_priority(&arguments));
}

/// An exact name matches before a short name, e.g. `x = 1°` is `init(x, y, z)`, not `init(xyz)`.
#[test]
fn match_priority_exact_name() {
    let x_y_z = Signature {
        signature: call_signature!(x: Type::angle(), y: Type::angle(), z: Type::angle()),
        default: tuple!(
            x = Value::deg(0.0),
            y = Value::deg(0.0),
            z = Value::deg(0.0)
        ),
    };
    let xyz = Signature {
        signature: call_signature!(xyz: Type::Any),
        default: tuple!(xyz = Value::deg(0.0)),
    };
    let arguments: ArgumentValueList = [argument_value!(x = Value::deg(1.0))].into_iter().collect();

    assert!(x_y_z.match_priority(&arguments) > xyz.match_priority(&arguments));
}

/// Positional arguments give no priority, the definition order of initializers decides,
/// e.g. `rotate(45°)` matches `init(angle: Angle)` and `init(x = 0°, y = 0°, z = 0°)` equally.
#[test]
fn match_priority_positional() {
    let x_y_z = Signature {
        signature: call_signature!(x: Type::length(), y: Type::length(), z: Type::length()),
        default: tuple!(x = mm(0.0), y = mm(0.0), z = mm(0.0)),
    };
    let arguments: ArgumentValueList = [argument_value!(mm(1.0))].into_iter().collect();

    assert_eq!(
        circle().match_priority(&arguments),
        x_y_z.match_priority(&arguments)
    );
}
