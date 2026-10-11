# Body

[![test](.test/body_workbench.svg)](.test/body_workbench.log)

```mu,body_workbench#fail
{
  sketch F() {} // error
}
```

[![test](.test/body_module.svg)](.test/body_module.log)

```mu,body_module#fail
{
  mod m {} // error
}
```

[![test](.test/body_function.svg)](.test/body_function.log)

```mu,body_function#fail
{
  fn f() {} f(); // error
}
```

[![test](.test/body_init.svg)](.test/body_init.log)

```mu,body_init#fail
{
  init() {} // error
}
```

[![test](.test/body_use.svg)](.test/body_use.log)

```mu,body_use
{
  use std;
}
```

[![test](.test/body_pub_use.svg)](.test/body_pub_use.log)

```mu,body_pub_use#fail
{
  pub use std; // error
}
```

[![test](.test/body_return.svg)](.test/body_return.log)

```mu,body_return#fail
{
  return 1; // error
}
```

[![test](.test/body_if.svg)](.test/body_if.log)

```mu,body_if
{
  if std::math::PI == 3 { }
}
```

[![test](.test/body_marker.svg)](.test/body_marker.log)

```mu,body_marker
{
  @input
}
```

[![test](.test/body_assignment_const.svg)](.test/body_assignment_const.log)

```mu,body_assignment_const#fail
{
  const B = 1; // error
}
```

[![test](.test/body_assignment_pub.svg)](.test/body_assignment_pub.log)

```mu,body_assignment_pub#fail
{
  pub p = 1; // error
}
```

[![test](.test/body_assignment_var.svg)](.test/body_assignment_var.log)

```mu,body_assignment_var
{
  a = 1;
}
```

[![test](.test/body_assignment_prop.svg)](.test/body_assignment_prop.log)

```mu,body_assignment_prop#fail
{
  prop a = 1; // error
}
```

[![test](.test/body_expression.svg)](.test/body_expression.log)

```mu,body_expression
{
  1 + 2;
}
```

[![test](.test/body_expression_model.svg)](.test/body_expression_model.log)

```mu,body_expression_model
{
  __builtin::geo2d::Circle(radius=1mm);
}
```
