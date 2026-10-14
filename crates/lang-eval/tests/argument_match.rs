// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

use microcad_lang_base::Identifier;
use microcad_lang_eval::ArgumentMatch;
use microcad_lang_types::{
    ArgumentValue, ArgumentValueList, CallSignature, Length, Tuple, Type, argument_value,
    arguments, call_signature, list, tuple,
};

struct ArgumentMatchDummy {
    pub signature: CallSignature,
    pub default: Tuple,
}

impl ArgumentMatch for ArgumentMatchDummy {
    fn call_signature(&self) -> CallSignature {
        self.signature.clone()
    }

    fn default_values(&self) -> Tuple {
        self.default.clone()
    }
}

#[test]
fn argument_matching() {
    let dummy = ArgumentMatchDummy {
        signature: call_signature!(a: Type::scalar(), b: Type::length(), c: Type::scalar(), d: Type::length()),
        default: tuple!(d = Length::mm(4.0)),
    };

    let arguments: ArgumentValueList = [
        argument_value!(a = 1.0),
        argument_value!(b = Length::mm(2.0)),
        argument_value!(3.0),
    ]
    .into_iter()
    .collect();

    let result = dummy
        .argument_match(&arguments)
        .expect("expect valid arguments");

    assert_eq!(
        result,
        arguments!(a = 1.0, b = Length::mm(2.0), c = 3.0, d = Length::mm(4.0))
    );
}

#[test]
fn argument_match_fail() {
    let dummy = ArgumentMatchDummy {
        signature: call_signature!(x: Type::scalar(), y: Type::scalar(), z: Type::scalar()),
        default: tuple!(),
    };

    let arguments: ArgumentValueList = [argument_value!(x = 1.0), argument_value!(Length::mm(1.0))]
        .into_iter()
        .collect();
    assert!(dummy.argument_match(&arguments).is_err());
}

#[test]
fn multi_match_cartesian_product() {
    let dummy = ArgumentMatchDummy {
        signature: call_signature!(a: Type::length(), b: Type::length()),
        default: tuple!(),
    };

    // f(a = [1.0, 2.0], b = [10.0, 20.0, 30.0])
    let args: ArgumentValueList = [
        ArgumentValue::new(list![1.0, 2.0], Some(Identifier::no_ref("a"))),
        ArgumentValue::new(list![10.0, 20.0, 30.0], Some(Identifier::no_ref("b"))),
    ]
    .into_iter()
    .collect();

    let result = dummy.argument_multi_match(&args).unwrap();

    // 2 x 3 = 6 combinations expected
    assert_eq!(result.len(), 6);

    assert_eq!(result[0], arguments!(a = 1.0, b = 10.0));
    assert_eq!(result[1], arguments!(a = 1.0, b = 20.0));
    assert_eq!(result[2], arguments!(a = 1.0, b = 30.0));
    assert_eq!(result[3], arguments!(a = 2.0, b = 10.0));
    assert_eq!(result[4], arguments!(a = 2.0, b = 20.0));
    assert_eq!(result[5], arguments!(a = 2.0, b = 30.0));
}

/// A positional argument matches a parameter of type `Any`,
/// but a parameter of exactly its type is preferred.
#[test]
fn argument_match_any() {
    let dummy = ArgumentMatchDummy {
        signature: call_signature!(x: Type::Any),
        default: tuple!(),
    };
    let arguments: ArgumentValueList = [argument_value!(Length::mm(2.0))].into_iter().collect();
    assert_eq!(
        dummy.argument_match(&arguments).expect("A match"),
        arguments!(x = Length::mm(2.0))
    );

    let dummy = ArgumentMatchDummy {
        signature: call_signature!(any: Type::Any, length: Type::length()),
        default: tuple!(),
    };
    let arguments: ArgumentValueList = [argument_value!(Length::mm(2.0)), argument_value!(1.0)]
        .into_iter()
        .collect();
    assert_eq!(
        dummy.argument_match(&arguments).expect("A match"),
        arguments!(any = 1.0, length = Length::mm(2.0))
    );
}

/// A default value is used if the argument is missing, a positional argument overrides it.
#[test]
fn argument_match_default_positional() {
    let dummy = ArgumentMatchDummy {
        signature: call_signature!(cond: Type::Bool, message: Type::String),
        default: tuple!(message = "default"),
    };

    let arguments: ArgumentValueList = [argument_value!(true)].into_iter().collect();
    assert_eq!(
        dummy.argument_match(&arguments).expect("A match"),
        arguments!(cond = true, message = "default")
    );

    let arguments: ArgumentValueList = [argument_value!(true), argument_value!("given")]
        .into_iter()
        .collect();
    assert_eq!(
        dummy.argument_match(&arguments).expect("A match"),
        arguments!(cond = true, message = "given")
    );
}
