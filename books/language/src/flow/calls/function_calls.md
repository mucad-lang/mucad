# Function Calls

A function is simply called by appending a parameter list to it's name:

[![test](.test/function_call.svg)](.test/function_call.log)

```mu,function_call
// function definition
fn f() {}

// function call
f();
```

[![test](.test/function_param_return.svg)](.test/function_param_return.log)

```mu,function_param_return
// function definition with parameter
fn f(n: Integer) {
    return n * 2;
}

// function calls with parameter (and result checks)
std::assert(f(1) == 2);
std::assert(f(2) == 4);
```
