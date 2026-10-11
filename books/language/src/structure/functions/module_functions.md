# Module Functions

A [module](../modules/) can contain functions that are accessible within
the module.
By declaring a function as *public* using the keyword `pub`, it becomes
available for use outside the module.

[![test](.test/mod.svg)](.test/mod.log)

```mu,mod#fail
// module math
mod math {
    // pow cannot be called from outside math
    fn pow(x: Scalar, n: Integer) -> Scalar {
        if n == 1 {
            x // return x
        } else {
            x * pow(x, n - 1) // return recursive product
        }
    }

    // square is callable from outside math
    pub fn square(x: Scalar) -> Scalar {
        // call internal pow
        pow(x, 2)
    }
}

// call square in math
math::square(2.0);
math::pow(2.0, 5); // error: pow is private
```
