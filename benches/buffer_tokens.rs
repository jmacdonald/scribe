#![feature(test)]

extern crate test;
extern crate scribe;

use test::Bencher;
use scribe::Workspace;
use std::path::PathBuf;

#[bench]
fn bench_tokens(b: &mut Bencher) {
    // Create a workspace with this benchmark test as a buffer.
    let mut workspace = Workspace::new(PathBuf::from("."));
    let path = PathBuf::from("benches/buffer_tokens.rs");
    workspace.open_buffer(path);

    // Benchmark the buffer's tokens method.
    let buffer = workspace.current_buffer().unwrap();
    b.iter(|| {
        let mut tokens = buffer.tokens().unwrap();

        // Exhaust the token iterator.
        for _ in tokens.iter() {}
    });
}
