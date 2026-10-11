# Module

[![test](.test/module_workbench.svg)](.test/module_workbench.log)

```mu,module_workbench
mod k {
  sketch F() {}
}
```

[![test](.test/module_module.svg)](.test/module_module.log)

```mu,module_module
mod k {
  mod m {}
}
```

[![test](.test/module_function.svg)](.test/module_function.log)

```mu,module_function
mod k {
  fn f() {}
}
```

[![test](.test/module_init.svg)](.test/module_init.log)

```mu,module_init#fail
mod k {
  init() { }  // error
}
```

[![test](.test/module_use.svg)](.test/module_use.log)

```mu,module_use
mod k {
  use std::geo2d;
}
```

[![test](.test/module_pub_use.svg)](.test/module_pub_use.log)

```mu,module_pub_use
mod k {
  pub use std::geo2d;
}
```

[![test](.test/module_return.svg)](.test/module_return.log)

```mu,module_return#fail
mod k {
  return 1;  // error
}
```

[![test](.test/module_if.svg)](.test/module_if.log)

```mu,module_if#fail
mod k {
  if std::math::PI == 3 { __builtin::geo2d::Circle(radius=1); }  // error
}
```

[![test](.test/module_assignment_const.svg)](.test/module_assignment_const.log)

```mu,module_assignment_const
mod k {
  const B = 1;
}
```

[![test](.test/module_assignment_pub.svg)](.test/module_assignment_pub.log)

```mu,module_assignment_pub
mod k {
  pub p = 1;
}
```

[![test](.test/module_assignment_var.svg)](.test/module_assignment_var.log)

```mu,module_assignment_var#fail
mod k {
  a = 1; // error
}
```

[![test](.test/module_assignment_prop.svg)](.test/module_assignment_prop.log)

```mu,module_assignment_prop#fail
mod k {
  prop a = 1;  // error
}
```

[![test](.test/module_expression.svg)](.test/module_expression.log)

```mu,module_expression#fail
mod k {
  1 + 2; // error
}
```

[![test](.test/module_expression_model.svg)](.test/module_expression_model.log)

```mu,module_expression_model#fail
mod k {
  __builtin::geo2d::Circle(radius=1mm); // error
}
```
