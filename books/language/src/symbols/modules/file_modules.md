# File Modules

File modules are modules that include µcad source files.

For example, if you put another file `second.µcad` next to your main file, you
can easily import this second file via its name `second`:

[![test](.test/file_modules_main.svg)](.test/file_modules_main.log)

```mu,file_modules_main?mode=todo
mod second;
second::f();
```

[![test](.test/file_modules_second.svg)](.test/file_modules_second.log)

```mu,file_modules_second?mode=todo
// file: second.µcad
pub fn f() {}
```

By writing `mod second;` in the first source file, microcad searches for either a
file called `second.µcad` or `second/mod.µcad` and loads its public symbols into a module.

Please note that according to µcad code convention that says that module name have to be lower snake code, file names have to be written lower snake case, too.

**Hint**: Because file modules are source files, they may contain statements that are not allowed in inline modules.
These statements (such as calls, expressions, or value assignments) will not be processed when including a file module.
