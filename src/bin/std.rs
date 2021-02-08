use io_uring_write_file::{Timer, DATA, TOTAL};
use std::fs::File;
use std::io;
use std::io::prelude::*;

fn main() -> io::Result<()> {
    let mut file = File::create("std.text")?;
    let timer = Timer::start();
    let npages = TOTAL / DATA.len();
    for _ in 0..npages {
        file.write_all(&DATA)?;
    }

    let elapsed = timer.stop();
    println!("{} ms", elapsed.as_millis());
    Ok(())
}
