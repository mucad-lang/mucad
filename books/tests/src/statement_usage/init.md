# Init

[![test](.test/init_workbench.svg)](.test/init_workbench.log)

```mu,init_workbench?result=fail
sketch K() { init(l:Length) {
  sketch F() {} // error
} } K(1cm);
```

[![test](.test/init_module.svg)](.test/init_module.log)

```mu,init_module?result=fail
sketch K() { init(l:Length) {
  mod m {} // error
} } K(1cm);
```

[![test](.test/init_function.svg)](.test/init_function.log)

```mu,init_function?result=fail
sketch K() { init(l:Length) {
  fn f() {} // error
} } K(1cm);
```

[![test](.test/init_init.svg)](.test/init_init.log)

```mu,init_init?result=fail
sketch K() { init(l:Length) {
  init() {} // error
} } K(1cm);
```

[![test](.test/init_use.svg)](.test/init_use.log)

```mu,init_use
sketch K() { init(l:Length) {
  use std;
} } K(1cm);
```

[![test](.test/init_pub_use.svg)](.test/init_pub_use.log)

```mu,init_pub_use?result=fail
sketch K() { init(l:Length) {
  pub use std; // error
} } K(1cm);
```

[![test](.test/init_return.svg)](.test/init_return.log)

```mu,init_return?result=fail
sketch K() { init(l:Length) {
  return l; // error
} } K(1cm);
```

[![test](.test/init_if.svg)](.test/init_if.log)

```mu,init_if?result=fail
sketch K() { init(l:Length) {
  if std::math::PI == l { } // error
} } K(1cm);
```

[![test](.test/init_assignment_const.svg)](.test/init_assignment_const.log)

```mu,init_assignment_const?result=fail
sketch K() { init(l:Length) {
  const B = l; // error
} } K(1cm);
```

[![test](.test/init_assignment_pub.svg)](.test/init_assignment_pub.log)

```mu,init_assignment_pub?result=fail
sketch K() { init(l:Length) {
  pub a = l; // error
} } K(1cm);
```

[![test](.test/init_assignment_var.svg)](.test/init_assignment_var.log)

```mu,init_assignment_var
sketch K() { init(l:Length) {
  a = l;
} } K(1cm);
```

[![test](.test/init_assignment_prop.svg)](.test/init_assignment_prop.log)

```mu,init_assignment_prop?result=fail
sketch K() { init(l:Length) {
  prop a = l; // error
} } K(1cm);
```

[![test](.test/init_expression.svg)](.test/init_expression.log)

```mu,init_expression?result=fail
sketch K() { init(l:Length) {
  l + 2; // error
} } K(1cm);
```

[![test](.test/init_expression_model.svg)](.test/init_expression_model.log)

```mu,init_expression_model?mode=todo
sketch K() { init(l:Length) {
  __builtin::geo2d::Circle(radius=l);
} } K(1cm);
```
