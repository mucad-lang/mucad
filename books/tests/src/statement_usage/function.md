# Function

[![test](.test/function_workbench.svg)](.test/function_workbench.log)

```mu,function_workbench#fail
fn f() {
  sketch S() {} // error
} f();
```

[![test](.test/function_module.svg)](.test/function_module.log)

```mu,function_module#fail
fn f() {
  mod m {} // error
} f();
```

[![test](.test/function_function.svg)](.test/function_function.log)

```mu,function_function#fail
fn f() {
  fn f() {} // error
} f();
```

[![test](.test/function_init.svg)](.test/function_init.log)

```mu,function_init#fail
fn f() {
  init() {} // error
} f();
```

[![test](.test/function_use.svg)](.test/function_use.log)

```mu,function_use
fn f() {
  use std;
} f();
```

[![test](.test/function_pub_use.svg)](.test/function_pub_use.log)

```mu,function_pub_use#fail
fn f() {
  pub use std; // error
} f();
```

[![test](.test/function_return.svg)](.test/function_return.log)

```mu,function_return
fn f() {
  return 1;
} f();
```

[![test](.test/function_if.svg)](.test/function_if.log)

```mu,function_if
fn f() {
  if std::math::PI == 3 { __builtin::geo2d::Circle(radius=1); }
} f();
```

[![test](.test/function_assignment_const.svg)](.test/function_assignment_const.log)

```mu,function_assignment_const#fail
fn f() {
  const B = 1; // error
} f();
```

[![test](.test/function_assignment_pub.svg)](.test/function_assignment_pub.log)

```mu,function_assignment_pub#fail
fn f() {
  pub p = 1; // error
} f();
```

[![test](.test/function_assignment_var.svg)](.test/function_assignment_var.log)

```mu,function_assignment_var
fn f() {
  a = 1;
} f();
```

[![test](.test/function_assignment_prop.svg)](.test/function_assignment_prop.log)

```mu,function_assignment_prop#fail
fn f() {
  prop a = 1; // error
} f();
```

[![test](.test/function_expression.svg)](.test/function_expression.log)

```mu,function_expression
fn f() {
  1 + 2;
} f();
```

[![test](.test/function_expression_model.svg)](.test/function_expression_model.log)

```mu,function_expression_model
fn f() {
  __builtin::geo2d::Circle(radius=1mm);
} f();
```
