# Calls

[*Workbenches*](../../structure/workbenches/) and [*functions*](../../structure/functions/)
 can get called, which just means there inner code gets executed.
There are several types of calls which have some different effects or usage.

| Call...                           | Example                               | Input(s)                           | Output       |
| --------------------------------- | ------------------------------------- | ---------------------------------- | ------------ |
| [function](#calling-functions)    | `value = my_function(..);`            | parameter list                     | Value        |
| [workbench](#calling-workbenches) | `model = MySketch(..);`               | parameter list                     | Model[^prop] |
| [operation](#calling-operations)  | `new_model = model.my_operation(..);` | Model[^prop] &<br/> parameter list | Model[^prop] |

[^prop]: including properties.

## Calling Functions

A call of a [function](../../structure/functions/)  consists of just the *identifier* and an [argument list](arguments.md).
and the result is a *value*:

[![test](.test/call_function.svg)](.test/call_function.log)

```mu,call_function
// call function -2 and store result in x
x = std::math::abs(-2);

// check value
std::assert(x == 2);
```

## Calling Workbenches

[Workbenches](workbench.md) can be called in the same way as functions
except that the result is a *model*.

[![test](.test/call_workbench.svg)](.test/call_workbench.log)

```mu,call_workbench
// call sketch Circle with a size and store object node in s
s = std::geo2d::Circle(diameter = 2cm);

std::assert(s.radius == 1cm);
```

## Calling Operations

*Operations* are called in a different way because they are always attached to
a [model](../../types/models/) which come out of [workbenches](../../structure/workbenches/).

[![test](.test/call_model.svg)](.test/call_model.log)

```mu,call_model
// call square with a size and store object node in s
s = std::geo2d::Circle(radius = 2cm);

// translate object s
s.std::ops::translate(x = 1cm);
```

Surely this looks better when using thw `use statement`.

[![test](.test/call_model_use.svg)](.test/call_model_use.log)

```mu,call_model_use
// use translate
use std::ops::translate;

// call square with a size and store object node in s
s = std::geo2d::Circle(radius = 2cm);

// translate object s
s.translate(x = 1cm);
```
