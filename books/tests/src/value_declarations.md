# Value Declarations

| Keyword(s)           | Context                                   | Description                                      | Example                      |
| -------------------- | ----------------------------------------- | ------------------------------------------------ | ---------------------------- |
| -                    | source, workbench, function               | local value                                      | `a=1`                        |
| -                    | init                                      | property (if in building plan), else local value | `a=1`                        |
| `prop`               | workbench                                 | model property                                   | `prop a=1`                   |
| `const`              | module                                    | private module value                             | `const a=1;`                 |
| `pub`, (`pub const`) | module                                    | public module value                              | `pub a=1;`, `pub const a=1;` |
| `use`                | module, source, workbench, function, init | private usage                                    | `use std::geo3d;`            |
| `pub use`            | module, source                            | public usage                                     | `pub use std::geo3d;`        |
| `fn`                 | module, source, workbench                 | private function                                 | `fn() {}`                    |
| `pub fn`             | module, source                            | public function                                  | `pub fn() {}`                |

[![test](.test/value_declarations.svg)](.test/value_declarations.log)

```mu,value_declarations
use std::debug::*;

mod module {
    
    use std::debug::*;

    // private module variable
    const value = 1;
    // public module variable
    pub pub_value = 2;

    pub mod pub_sub_module {
        // pre-init code
        use std::debug::*;

        // private module variable
        const value = 3;
        // public module variable
        pub pub_value = 4;

        // private workbench
        sketch PrivateWorkbench() {}

        // public workbench
        pub sketch Workbench(param = 5) {
            const sketch_local = 6;

            init(alt_param = 7) {
                init_local = 8;

                std::assert_eq(value, 3);
                std::assert_eq(pub_value, 4);
                assert_invalid("param");
                std::assert_eq(sketch_local, 6);
                std::assert_eq(alt_param, 7);
                std::assert_eq(init_local, 8);
            }

            // property of sketch
            prop property = 9;

            // post init code
            std::assert_eq(value, 3);
            std::assert_eq(pub_value, 4);
            std::assert_eq(param, 5);
            std::assert_eq(sketch_local, 6);
            assert_invalid("alt_param");
            assert_invalid("init_local");
            std::assert_eq(property, 9);

            function();
        }

        fn function(fn_param = 10) {
            std::assert_eq(value, 3);
            std::assert_eq(pub_value, 4);
            assert_invalid("param");
            // assert_invalid(Workbench);
            std::assert_eq(fn_param, 10);

            return 0;
        }
    }

    pub fn function(fn_param = 11) {
        std::assert_eq(value, 1);
        std::assert_eq(pub_value, 2);
        assert_invalid("pub_sub_module::value");
        std::assert_eq(pub_sub_module::pub_value, 4);
        assert_invalid("Workbench");
        assert_invalid("PrivateWorkbench");
        std::assert_eq(fn_param, 11);
        
        return 0;
    }
}

// source file code 
assert_invalid("module::value");
std::assert_eq(module::pub_value, 2);
assert_invalid("module::pub_sub_module::value");
std::assert_eq(module::pub_sub_module::pub_value, 4);
std::assert_eq(module::pub_sub_module::Workbench(p=5).property, 9);
assert_invalid("module::pub_sub_module::PrivateWorkbench");
std::assert_eq(module::function(), 0);
```
