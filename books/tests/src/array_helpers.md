# Array Helpers

There are several helpers in the standard library which might worth mentioning.

## `len`

You can count the number of elements in an array using `std::array::len()`.

[![test](.test/array_helper_len.svg)](.test/array_helper_len.log)

```mu,array_helper_len
assert_eq(std::array::len([1,2,3]), 3);
```

## `first`

You can get the first element of an array using `std::array::first()`.

[![test](.test/array_helper_first.svg)](.test/array_helper_first.log)

```mu,array_helper_first
assert_eq(std::array::first([1,2,3]), 1);
```

## `last`

You can get the first element of an array using `std::array::last()`.

[![test](.test/array_helper_last.svg)](.test/array_helper_last.log)

```mu,array_helper_last
assert_eq(std::array::last([1,2,3]), 3);
```

## `head`

Get the first element of an array.

> [!NOTE] this function is supposed to be deprected. Use `first` instead.

[![test](.test/array_helper_head.svg)](.test/array_helper_head.log)

```mu,array_helper_head
assert_eq(std::array::head([1,2,3]), 1);
```

## `tail`

You can get the tail of an array using `std::array::first()` and `std::array::tail()`.

[![test](.test/array_helper_tail.svg)](.test/array_helper_tail.log)

```mu,array_helper_tail
assert_eq(std::array::tail([1,2,3]), [2,3]);
```

## `rev`

You can reverse the order the elements in an array using `std::array::rev()`.

[![test](.test/array_helper_rev.svg)](.test/array_helper_rev.log)

```mu,array_helper_rev
assert_eq(std::array::rev([1,2,3]), [3,2,1]);
```

## `sorted`

You can sort the elements in an array using `std::array::sorted()`.

[![test](.test/array_helper_sorted.svg)](.test/array_helper_sorted.log)

```mu,array_helper_sorted
assert_eq(std::array::sorted([3,1,2]), [1,2,3]);
```

## `contains`

You can sort the elements in an array using `std::array::contains(x)`.

[![test](.test/array_helper_contains.svg)](.test/array_helper_contains.log)

```mu,array_helper_contains
assert_eq(std::array::contains([3,1,2], 1), true);
assert_eq(std::array::contains([3,1,2], 4), false);
```

## `is_ascending`

You can sort the elements in an array using `std::array::is_ascensing(x)`.

[![test](.test/array_helper_is_ascending.svg)](.test/array_helper_is_ascending.log)

```mu,array_helper_is_ascending
assert_eq(std::array::is_ascending([1,2,3]), true);
assert_eq(std::array::is_ascending([3,2,1]), false);
```

## `is_descending`

You can sort the elements in an array using `std::array::is_descending(x)`.

[![test](.test/array_helper_is_descending.svg)](.test/array_helper_is_descending.log)

```mu,array_helper_is_descending
assert_eq(std::array::is_descending([1,2,3]), false);
assert_eq(std::array::is_descending([3,2,1]), true);
```
