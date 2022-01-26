extern crate cc;

fn main() {
    cc::Build::new()
        .file("src/untrusted.c")
        .compile("untrusted")
}
