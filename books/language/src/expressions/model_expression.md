# Model Expressions

Things change when an expression consists of *models* instead of just *values*.
We call this a *model expression*:

[![test](.test/model_expression.svg)](.test/model_expression.log)

```mu,model_expression
Rect(1cm) - Circle(radius = 3mm);
```

In this expression which consists of a subtraction operation of the results of
two [calls](../../flow/calls/workbench_calls.md) to `Rect` and `Circle`.

Output
  :![output](.test/model_expression-out.svg)

Building a *group* (using curly braces) of both operands and applying the
operation `subtract` to it is equivalent to the above code:

[![test](.test/model_expression_builtin.svg)](.test/model_expression_builtin.log)

```mu,model_expression_subtract

{
    Rect(1cm);
    Circle(radius = 3mm);
}.std::ops::subtract();
```

Output
  :![output](.test/model_expression_builtin-out.svg)

The following operations can be applied to 2D or 3D models:

| Operator | Builtin Operation           | Description              |
| :------: | --------------------------- | ------------------------ |
|   `-`    | `__builtin::ops::subtract`  | Geometrical difference   |
|   `\|`   | `__builtin::ops::union`     | Geometrical union        |
|   `&`    | `__builtin::ops::intersect` | Geometrical intersection |
