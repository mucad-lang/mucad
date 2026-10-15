# Public Assignments

Public assignments provide a value to the inner **and** the outer of a module.

[![test](.test/pub_assignment.svg)](.test/pub_assignment.log)

```mu,pub_assignment
mod my_module {
    pub mod sub_module {
        pub const TEXT = "Hello";
    }
}

std::print(my_module::sub_module::TEXT);
```

## Restrictions

### Not in workbenches

Using `pub` is not allowed within workbenches:

[![test](.test/pub_assignment_workbench.svg)](.test/pub_assignment_workbench.log)

```mu,pub_assignment_workbench
sketch MySketch() {
    pub const TEXT = "Hello";
    std::print(TEXT);
}

MySketch();
```

### Not in functions

[![test](.test/pub_assignment_fn.svg)](.test/pub_assignment_fn.log)

```mu,pub_assignment_fn
fn f() {
    const MY_CONST = 1;
}
f();
```
