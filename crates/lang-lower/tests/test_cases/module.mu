// Extracted from: assignments/const.md
// [![test](.test/const_assignment_mod.svg)](.test/const_assignment_mod.log)

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
        std::debug::assert_eq([TEXT, text]);
    }
}

my_module::MySketch("Hello my_module");
std::debug::assert_eq([my_module::f(), "Hello my_module"]);
std::debug::assert_eq([TEXT, "Hello"]);
