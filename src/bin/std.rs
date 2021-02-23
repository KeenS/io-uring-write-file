use io_uring_write_file::{write_std, Setup, Timer};
use std::io;

fn main() -> io::Result<()> {
    let timer = Timer::start();

    write_std("../std.text", Setup::default())?;

    let elapsed = timer.stop();
    println!("{} ms", elapsed.as_millis());
    Ok(())
}
