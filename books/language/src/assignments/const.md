# Constant Assignments

Unlike *values*, constants are not stored on the *stack* but in
the *symbol table*.
For example this allows them to be accessed from within functions or workbenches
in the same module where the constant is defined.
Constants can be placed in
[source files](../structure/source_file.md),
[modules](../structure/modules/) or
[initialization code](../structure/workbenches/elements/init_code.md).

[![test](.test/const_assignment_mod.svg)](.test/const_assignment_mod.log)

```mu,const_assignment_mod
const TEXT = "Hello";

mod my_module {
    // constant assignment
    const TEXT = "Hello my_module";

    // public function
    pub fn f() -> String {
        TEXT
    }

    // public workbench
    pub sketch MySketch(text: String) {
        assert_eq(TEXT, text);
    }
}

my_module::MySketch("Hello my_module");
assert_eq(my_module::f(), "Hello my_module");
assert_eq(TEXT, "Hello");
```

Additionally, constant assignments are permitted in the *init code* of a
*workbench*, where value assignments are likewise prohibited.

[![test](.test/const_assignment_workbench.svg)](.test/const_assignment_workbench.log)

```mu,const_assignment_workbench
sketch MySketch(text: String) {
    // constant assignment in initialization code
    const TEXT = "Hello";

    init() {
        text = TEXT;
    }

    print(text);
}

MySketch();
```

## Restrictions

### Uppercase naming

Constants are always written in `UPPER_CASE`.

[![test](.test/const_assignment_uppercase.svg)](.test/const_assignment_uppercase.log)

```mu,const_assignment_uppercase
const A = 1; // ok
const a = 1; // warning
const MyValue = 1; // warning
const MY_VALUE = 1; // ok
```

### Ambiguous Names

A constant cannot be defined within the same module or workbench twice.

[![test](.test/const_assignment_shadow.svg)](.test/const_assignment_shadow.log)

```mu,const_assignment_shadow?result=fail
mod module {
    const A = 5;
    const A = 1; // error: ambiguous identifier in this module

    pub mod another_module {
        const A = 5; // ok

        pub fn a() -> Integer { A }
    }

    pub sketch Sketch() {
        const A = 5;
        const A = 5; // error: ambiguous identifier in this workbench
    }
}

assert_eq(module::another_module::a(), 5);
module::Sketch();
```

### Not in building code

Constant assignments cannot be used in building code (the code below any initializers).

[![test](.test/const_assignment_building_code.svg)](.test/const_assignment_building_code.log)

```mu,const_assignment_building_code
sketch MySketch() {
    init(_: Integer) {}
    const MY_CONST = 1;
}
MySketch();
```

Constant assignments must be on top of the workbench code if initializers are not used.

[![test](.test/const_assignment_workbench_code.svg)](.test/const_assignment_workbench_code.log)

```mu,const_assignment_workbench_code
sketch MySketch() {
    const MY_CONST = 1; // allowed if no initializers
}
MySketch();
```

They cannot be placed below non constant assignments within in a workbench.

[![test](.test/const_assignment_workbench_code_wrong.svg)](.test/const_assignment_workbench_code_wrong.log)

```mu,const_assignment_workbench_code_wrong
sketch MySketch() {
    _i = 5; // any non const code
    const MY_CONST = 1;
}
MySketch();
```

### Not in function

Constants can be used in functions.

[![test](.test/const_assignment_fn.svg)](.test/const_assignment_fn.log)

```mu,const_assignment_fn
fn f() {
    const MY_CONST = 1;
}
f();

sketch MySketch() {
    fn f() {
        const MY_CONST = 1;
    }
    f();
}
MySketch();
```

### Not in initializers

Constant assignments cannot be used in initializers.

[![test](.test/const_assignment_init.svg)](.test/const_assignment_init.log)

```mu,const_assignment_init?result=fail
sketch MySketch() {
    init(_: Integer) {
        const MY_CONST = 1; // error: not allowed in initializers
    }
}
MySketch();
```
