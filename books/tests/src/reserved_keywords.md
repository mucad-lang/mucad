# Using reserved keywords

[![test](.test/reserved_keywords.svg)](.test/reserved_keywords.log)

```mu,reserved_keywords#fail
const assembly = 1; // error

a = 1; // recovered 

unit = 1; // error

fn match() { // error

}

somefun(enum); // error

struct asd { // error

}

a = 2; // recovered 

unit foo; // error

a = 3; // recovered

a = material; // error

type A = Integer; // error
```
