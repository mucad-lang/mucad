// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

use derive_more::{Display, From};
use microcad_lang_base::{Identifier, SrcRef};
use microcad_macros::SrcReferrer;
use serde::{Deserialize, Serialize};

use crate::{Tuple, Ty, Type, Value, ValueError, ValueResult};

#[derive(Clone, Default, Display, From, Hash, Debug, PartialEq, Serialize, Deserialize)]
pub struct Arguments(pub Tuple);

impl Arguments {
    pub fn positional_iter(&self) -> impl Iterator<Item = &Value> {
        self.0.positional.iter()
    }

    pub fn named_iter(&self) -> impl Iterator<Item = (&Identifier, &Value)> {
        self.0.named.iter().map(|(k, v)| (k, v))
    }

    /// Retrieves a named [`Value`] by key.
    pub fn get(&self, name: impl AsRef<str>) -> &Value {
        self.named_iter()
            .find_map(|(id, v)| {
                if id == &Identifier::from(name.as_ref()) {
                    Some(v)
                } else {
                    None
                }
            })
            .unwrap() // TODO: This is not cool.
    }

    /// Get `lhs` and `rhs` from binary arguments.
    pub fn get_binary(self) -> (Value, Value) {
        (self.get("lhs").clone(), self.get("rhs").clone())
    }

    pub fn get_unary(self) -> Value {
        self.get("rhs").clone()
    }

    pub fn get_cond(&self) -> ValueResult<bool> {
        self.try_get("cond")
    }

    pub fn try_get<T: TryFrom<Value, Error = ValueError>>(
        &self,
        name: impl AsRef<str>,
    ) -> ValueResult<T> {
        T::try_from(self.get(name).clone())
    }

    /// Consumes `self`, removes the named argument if present, and returns the modified `Arguments`.
    pub fn with_field_removed(mut self, name: impl AsRef<str>) -> Self {
        let key = Identifier::from(name.as_ref());
        self.0 = self.0.with_field_removed(&key);
        self
    }
}

impl FromIterator<(Identifier, Value)> for Arguments {
    fn from_iter<T: IntoIterator<Item = (Identifier, Value)>>(iter: T) -> Self {
        Self::from(Tuple::from_iter(iter))
    }
}

/// Argument value.
#[derive(Clone, Debug)]
pub struct ArgumentValue {
    /// *value* of the argument.
    pub value: Value,
    /// The name of the argument, if given by the caller (e.g. `radius = 10mm`).
    pub id: Option<Identifier>,
    /// The identifier of an argument without name, if its expression is a single identifier
    /// (e.g. `x` in `f(x)`). It matches a parameter with the same name.
    pub inline_id: Option<Identifier>,
}

impl std::fmt::Display for ArgumentValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{val}", val = self.value,)
    }
}

impl Ty for ArgumentValue {
    fn ty(&self) -> Type {
        self.value.ty()
    }
}

impl ArgumentValue {
    /// Create new argument value
    pub fn new(value: impl Into<Value>, id: Option<Identifier>) -> Self {
        Self {
            value: value.into(),
            id,
            inline_id: None,
        }
    }

    /// Create a positional argument (no name)
    pub fn positional(value: impl Into<Value>) -> Self {
        Self::new(value, None)
    }

    /// Create a named argument (e.g., `radius: 10.0`)
    pub fn named(id: impl Into<Identifier>, value: impl Into<Value>) -> Self {
        Self::new(value, Some(id.into()))
    }
}

/// Collection of *argument values* (e.g. `( x=1, y=2 )`).
///
/// Also provides methods to find a matching call
/// between it and a given *parameter list*.
#[derive(Clone, Debug, Default, SrcReferrer)]
pub struct ArgumentValueList {
    pub args: Vec<ArgumentValue>,
    pub src_ref: SrcRef,
}

impl ArgumentValueList {
    pub fn get_named(&self, name: impl AsRef<str>) -> Option<&Value> {
        let this_name = Identifier::from(name.as_ref());
        self.args.iter().find_map(|arg| {
            if let Some(name) = &arg.id
                && name == &this_name
            {
                Some(&arg.value)
            } else {
                None
            }
        })
    }

    /// Remove a named argument and return its value.
    pub fn remove_named(&mut self, name: impl AsRef<str>) -> Option<Value> {
        let this_name = Identifier::from(name.as_ref());
        let index = self
            .args
            .iter()
            .position(|arg| arg.id.as_ref() == Some(&this_name))?;
        Some(self.args.remove(index).value)
    }

    pub fn iter(&self) -> impl Iterator<Item = &ArgumentValue> {
        self.args.iter()
    }
}

impl std::fmt::Display for ArgumentValueList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", {
            self.args
                .iter()
                .map(|arg| arg.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        })
    }
}

impl<Arg> FromIterator<Arg> for ArgumentValueList
where
    Arg: Into<ArgumentValue>,
{
    fn from_iter<T: IntoIterator<Item = Arg>>(iter: T) -> Self {
        let args: Vec<_> = iter.into_iter().map(|a| a.into()).collect();
        Self {
            src_ref: SrcRef::none(),
            args,
        }
    }
}

/// Create arguments via macro
/// arguments!(a = Length::mm(10.0))
#[macro_export]
macro_rules! arguments {
    ($($key:ident = $value:expr),*) => {
            $crate::Arguments::from($crate::Tuple::from([$( (stringify!($key), $crate::Value::from($value)) ),* ]
                .iter()))
    };

    ($($value:expr),*) => {
            $crate::Arguments::from($crate::Tuple::from([$( $crate::Value::from($value)),* ]
                .iter()))
    };
}

/// Shortcut to create an argument value
#[macro_export]
macro_rules! argument_value {
    ($id:ident = $value:expr) => {
        $crate::ArgumentValue::named(stringify!($id), $value)
    };
    ($value:expr) => {
        $crate::ArgumentValue::positional($value)
    };
    () => {};
}

#[macro_export]
macro_rules! parse_args {
    // Single item with custom string key override: `self_: Rc<ModelTree> => "self"`
    ($args:expr, $( $name:ident : $ty:ty $(=> $key:expr)? ),* $(,)?) => {
        $(
            let $name: $ty = $args.try_get(
                parse_args!(@key $name $(=> $key)?)
            )?;
        )*
    };

    // Helper rules to select explicit key or fallback to stringify!
    (@key $name:ident => $key:expr) => { $key };
    (@key $name:ident) => { stringify!($name) };
}
