# Pre-Init

[![test](.test/pre_init_workbench.svg)](.test/pre_init_workbench.log)

```mu,pre_init_workbench?result=fail
sketch K() { 
  sketch F() {}  // error
init(l:Length) {} } K();
```

[![test](.test/pre_init_module.svg)](.test/pre_init_module.log)

```mu,pre_init_module?result=fail
sketch K() { 
  mod m {}   // error
init(l:Length) {} } K();
```

[![test](.test/pre_init_function.svg)](.test/pre_init_function.log)

```mu,pre_init_function?result=fail
sketch K() { 
  fn f() {} f();   // error
init(l:Length) {} } K();
```

[![test](.test/pre_init_init.svg)](.test/pre_init_init.log)

```mu,pre_init_init
sketch K(x: Length) { 
  init() { x=1; }
init(l:Length) {} } K();
```

[![test](.test/pre_init_use.svg)](.test/pre_init_use.log)

```mu,pre_init_use
sketch K() { 
  use std;
init(l:Length) {} } K();
```

[![test](.test/pre_init_pub_use.svg)](.test/pre_init_pub_use.log)

```mu,pre_init_pub_use?result=fail
sketch K() { 
  pub use std; // error
init(l:Length) {} } K();
```

[![test](.test/pre_init_return.svg)](.test/pre_init_return.log)

```mu,pre_init_return?result=fail
sketch K() { 
  return 1; // error
init(l:Length) {} } K();
```

[![test](.test/pre_init_if.svg)](.test/pre_init_if.log)

```mu,pre_init_if?result=fail
sketch K() { 
  if std::math::PI == 3 { } // error
init(l:Length) {} } K();
```

[![test](.test/pre_init_assignment_const.svg)](.test/pre_init_assignment_const.log)

```mu,pre_init_assignment_const
sketch K() { 
  const B = 1;
init(l:Length) {} } K();
```

[![test](.test/pre_init_assignment_pub.svg)](.test/pre_init_assignment_pub.log)

```mu,pre_init_assignment_pub?result=fail
sketch K() { 
  pub p = 1;  // error
init(l:Length) {} } K();
```

[![test](.test/pre_init_assignment_var.svg)](.test/pre_init_assignment_var.log)

```mu,pre_init_assignment_var?result=fail
sketch K() { 
  a = 1; // error
init(l:Length) {} } K();
```

[![test](.test/pre_init_assignment_prop.svg)](.test/pre_init_assignment_prop.log)

```mu,pre_init_assignment_prop?result=fail
sketch K() {
  prop a = 1; // error
init(l:Length) {} } K();
```

[![test](.test/pre_init_expression.svg)](.test/pre_init_expression.log)

```mu,pre_init_expression?result=fail
sketch K() { 
  1 + 2; // error
init(l:Length) {} } K();
```

[![test](.test/pre_init_expression_model.svg)](.test/pre_init_expression_model.log)

```mu,pre_init_expression_model?result=fail
sketch K() { 
  __builtin::geo2d::Circle(radius=1mm); // error
init(l:Length) {} }
```
