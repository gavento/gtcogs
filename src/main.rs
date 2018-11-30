use std::fmt::Debug;

#[derive(Debug)]
struct X<T: Debug> {
    a: T,
}

fn main() {
    let d: &dyn Debug = (&42u32) as &Debug;
    let x1: X<&dyn Debug> = X { a: d };
    let x2: X<i64> = X { a: -42 };
    let x3: X<&i64> = X { a: &-42 };
    println!("Hello, world {:?} {:?} {:?}!", x1, x2, x3);
}
