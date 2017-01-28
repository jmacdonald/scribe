#![feature(test)]

extern crate test;
extern crate scribe;

use test::Bencher;
use scribe::Workspace;
use std::path::Path;

#[bench]
fn bench_tokens(b: &mut Bencher) {
    // Create a workspace with this benchmark test as a buffer.
    let mut workspace = Workspace::new(Path::new(".")).unwrap();
    let path = Path::new("benches/buffer_tokens.rs");
    workspace.open_buffer(path);

    // Benchmark the buffer's tokens method.
    let buffer = workspace.current_buffer().unwrap();
    b.iter(|| {
        let tokens = buffer.tokens().unwrap();

        // Exhaust the token iterator.
        for _ in tokens.iter() {}
    });
}
