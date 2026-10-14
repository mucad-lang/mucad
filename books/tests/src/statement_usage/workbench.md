# Workbench

[![test](.test/workbench_workbench.svg)](.test/workbench_workbench.log)

```mu,workbench_workbench?result=fail
sketch K() {
  sketch F() {} F(); // error
} K();
```

[![test](.test/workbench_module.svg)](.test/workbench_module.log)

```mu,workbench_module?result=fail
sketch K() {
  mod m {} // error
} K();
```

[![test](.test/workbench_function.svg)](.test/workbench_function.log)

```mu,workbench_function
sketch K() {
  fn f() {} f();
} K();
```

[![test](.test/workbench_init.svg)](.test/workbench_init.log)

```mu,workbench_init
sketch K(x: Scalar) {
  init() { x = 1; }
} K();
```

[![test](.test/workbench_use.svg)](.test/workbench_use.log)

```mu,workbench_use
sketch K() {
  use std;
} K();
```

[![test](.test/workbench_pub_use.svg)](.test/workbench_pub_use.log)

```mu,workbench_pub_use?result=fail
sketch K() {
  pub use std; // error
} K();
```

[![test](.test/workbench_return.svg)](.test/workbench_return.log)

```mu,workbench_return?result=fail
sketch K() {
  return 1; // error
} K();
```

[![test](.test/workbench_if.svg)](.test/workbench_if.log)

```mu,workbench_if
sketch K() {
  if std::math::PI == 3 { }
} K();
```

[![test](.test/workbench_marker.svg)](.test/workbench_marker.log)

```mu,workbench_marker?result=fail
sketch K() {
  @input; // error: Input marker is not allowed here.
} K();
```

[![test](.test/workbench_assignment_const.svg)](.test/workbench_assignment_const.log)

```mu,workbench_assignment_const
sketch K() {
  const B = 1;
} K();
```

[![test](.test/workbench_assignment_var.svg)](.test/workbench_assignment_var.log)

```mu,workbench_assignment_var
sketch K() {
  a = 1;
} K();
```

[![test](.test/workbench_assignment_prop.svg)](.test/workbench_assignment_prop.log)

```mu,workbench_assignment_prop
sketch K() {
  prop a = 1;
} K();
```

[![test](.test/workbench_assignment_pub.svg)](.test/workbench_assignment_pub.log)

```mu,workbench_assignment_pub?result=fail
sketch K() {
  pub a = 1; // error
} K();
```

[![test](.test/workbench_expression.svg)](.test/workbench_expression.log)

```mu,workbench_expression
sketch K() {
  1 + 2;
} K();
```

[![test](.test/workbench_expression_model.svg)](.test/workbench_expression_model.log)

```mu,workbench_expression_model
sketch K() {
  __builtin::geo2d::Circle(radius=1mm);
} K();
```
