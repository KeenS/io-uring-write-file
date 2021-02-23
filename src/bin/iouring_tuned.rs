use io_uring_write_file::{iouring_write_tuned, Setup, Timer};
use std::io;
// use std::os::unix::fs::OpenOptionsExt;

fn main() -> io::Result<()> {
    let timer = Timer::start();

    iouring_write_tuned("../iouring_tuned.text", Setup::default)?;

    let elapsed = timer.stop();
    println!("{} ms", elapsed.as_millis());

    Ok(())
}
