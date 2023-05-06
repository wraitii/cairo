#[derive(Copy, Drop)]
struct SomeStruct {}

impl SomeStruct {
    fn foo(self: SomeStruct) -> felt252 {
        0
    }
}

#[derive(Copy, Drop)]
struct SomeGenericStruct<T> {
    a: T
}

impl SomeGenericStruct<T, impl DN: NumericLiteral<T>> {
    fn foo(self: SomeGenericStruct<T>) -> T {
        self.a
    }

    fn foo2(self: SomeGenericStruct<T>) -> T {
        self.a
    }
}

#[test]
fn main() {
    let a = SomeStruct {};
    let b = SomeGenericStruct::<felt252> { a: 23 };
    assert(a.foo() == 0, 'a.foo() == 0');
    assert(b.foo() == 23, 'b.foo() == 23');
    assert(b.foo2() == 23, 'b.foo2() == 23');
}
