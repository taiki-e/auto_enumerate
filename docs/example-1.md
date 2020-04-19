```rust
fn foo(x: i32) -> impl Iterator<Item = i32> {
    enum __Enum1<__T1, __T2> {
        __T1(__T1),
        __T2(__T2),
    }

    impl<__T1, __T2> ::core::iter::Iterator for __Enum1<__T1, __T2>
    where
        __T1: ::core::iter::Iterator,
        __T2: ::core::iter::Iterator<Item = <__T1 as ::core::iter::Iterator>::Item>,
    {
        type Item = <__T1 as ::core::iter::Iterator>::Item;
        fn next(&mut self) -> ::core::option::Option<Self::Item> {
            match self {
                __Enum1::__T1(x) => x.next(),
                __Enum1::__T2(x) => x.next(),
            }
        }
        fn size_hint(&self) -> (usize, ::core::option::Option<usize>) {
            match self {
                __Enum1::__T1(x) => x.size_hint(),
                __Enum1::__T2(x) => x.size_hint(),
            }
        }
    }

    match x {
        0 => __Enum1::__T1(1..10),
        _ => __Enum1::__T2(vec![5, 10].into_iter()),
    }
}
```
