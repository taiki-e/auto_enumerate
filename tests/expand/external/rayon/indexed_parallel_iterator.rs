use auto_enums::enum_derive;

#[enum_derive(rayon::IndexedParallelIterator)]
enum Enum<A, B> {
    A(A),
    B(B),
}

fn main() {}
