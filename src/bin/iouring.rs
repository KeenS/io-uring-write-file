use io_uring_write_file::{iouring_write, Setup, Timer};
use std::io;

fn main() -> io::Result<()> {
    let timer = Timer::start();

    iouring_write("../iouring.text", Setup::default())?;

    let elapsed = timer.stop();
    println!("{} ms", elapsed.as_millis());

    Ok(())
}
