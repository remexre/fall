#[cfg(feature = "parser")]
extern crate lalrpop;

#[cfg(feature = "parser")]
fn main() {
    lalrpop::process_root().unwrap();
}

#[cfg(not(feature = "parser"))]
fn main() {}
