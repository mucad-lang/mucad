# Conditions

The *`if` statement* controls the program flow by the result of boolean conditions.

## Structure of an if statement

In general, an `if` statement consists of the following elements in a fixed order:

1. an initial `if` followed by the condition in form of an [*boolean expression*](../expressions/#boolean-results)
2. a block of code (in `{ .. }`) which is processed if the condition is true
3. maybe one or more `else if` statements with alternative conditions and code blocks
4. and last maybe an `else` statement followed by a code block which got executed when no other condition was `true`.

Conditions lead to different executions paths for different cases.

Here is a simple example:

[![test](.test/if_statement.svg)](.test/if_statement.log)

```mu,if_statement
use std::geo2d::*;

x = 0; // output changes if you change that value

if x > 0 {
    Circle(radius = 5mm);
} else if x < 0 {
    Rect(10mm);
} else {
    Hexagon(5mm);
}
```

Output
  :![test](.test/if_statement-out.svg)

## `if` in workbenches

Inside a workbench block, an `if` statement can be used to select different shapes
or constructions depending on input parameters.
So in the following example all possible geometries are generated with [*parameter multiplicity*](calls/multiplicity.md)
and put side by side with the operation `std::ops::align`.

[![test](.test/if_statement_sketch.svg)](.test/if_statement_sketch.log)

```mu,if_statement_sketch
use std::ops::*;
use std::math::*;
use std::geo2d::*;

sketch MySketch(x: Integer) {
    if x > 0 {
        Circle(radius = 5mm)
    } else if x < 0 {
        Rect(10mm)
    } else {
        Hexagon(5mm)
    }
}

MySketch([-1, 0, 2]).align(X, 5mm);
```

Output
  :![output](.test/if_statement_sketch-out.svg)

## `if` in expressions

If statements can also be used as an expression, evaluating to the value from
the chosen branch.

[![test](.test/if_expression.svg)](.test/if_expression.log)

```mu,if_expression
use std::ops::*;
use std::math::*;
use std::geo2d::*;

sketch MySketch(x: Integer) {
    outer = if x > 0 {
        Circle(radius = 5mm)
    } else if x < 0 {
        Rect(10mm)
    } else {
        Hexagon(5mm)
    };

    outer - Circle(radius = 3mm)
}

MySketch([-1, 0, 1]).align(X, 5mm);
```

Output
  :![output](.test/if_expression-out.svg)
