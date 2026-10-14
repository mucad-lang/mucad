# Source

[![test](.test/source_workbench.svg)](.test/source_workbench.log)

```mu,source_workbench
sketch F() {} F();
```

[![test](.test/source_module.svg)](.test/source_module.log)

```mu,source_module
mod m {}
```

[![test](.test/source_function.svg)](.test/source_function.log)

```mu,source_function
fn f() {} f();
```

[![test](.test/source_init.svg)](.test/source_init.log)

```mu,source_init?result=fail
init() {} // error
```

[![test](.test/source_use.svg)](.test/source_use.log)

```mu,source_use
use std;
```

[![test](.test/source_pub_use.svg)](.test/source_pub_use.log)

```mu,source_pub_use
pub use std;
```

[![test](.test/source_return.svg)](.test/source_return.log)

```mu,source_return?result=fail
return 1;  // error
```

[![test](.test/source_if.svg)](.test/source_if.log)

```mu,source_if
if std::math::PI == 3 { __builtin::geo2d::Circle(radius=1mm); }
```

[![test](.test/source_assignment_const.svg)](.test/source_assignment_const.log)

```mu,source_assignment_const
const B = 1;
```

[![test](.test/source_assignment_pub.svg)](.test/source_assignment_pub.log)

```mu,source_assignment_pub
pub p = 1;
```

[![test](.test/source_assignment_var.svg)](.test/source_assignment_var.log)

```mu,source_assignment_var
a = 1;
```

[![test](.test/source_assignment_prop.svg)](.test/source_assignment_prop.log)

```mu,source_assignment_prop?result=fail
prop a = 1;  // error
```

[![test](.test/source_expression.svg)](.test/source_expression.log)

```mu,source_expression
1 + 2;
```

[![test](.test/source_expression_model.svg)](.test/source_expression_model.log)

```mu,source_expression_model
__builtin::geo2d::Circle(radius=1mm);
```
