#![feature(unsize)] //DUMMY
#![feature(coerce_unsized)] //DUMMY

mod box_test;
fn main() {
    println!("Hello, world!");
    box_test::test1();
}
