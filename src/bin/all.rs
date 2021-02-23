use io_uring_write_file::{iouring_write, iouring_write_tuned, write_std, Setup, Timer};
use std::io;

fn run_set(name: &str, setup: Setup) -> io::Result<()> {
    println!("std {}", name);
    let timer = Timer::start();
    write_std(format!("../std_{}.text", name), setup)?;
    let elapsed = timer.stop();
    println!("{} ms", elapsed.as_millis());

    println!("iouring {}", name);
    let timer = Timer::start();
    iouring_write(format!("../iouring_{}.text", name), setup)?;
    let elapsed = timer.stop();
    println!("{} ms", elapsed.as_millis());

    println!("iouring_tuned_{}", name);
    let timer = Timer::start();
    iouring_write_tuned(format!("../iouring_tuned_{}.text", name), setup)?;
    let elapsed = timer.stop();
    println!("{} ms", elapsed.as_millis());
    Ok(())
}

fn main() -> io::Result<()> {
    run_set(
        "default",
        Setup {
            sync: false,
            direct: false,
        },
    )?;

    run_set(
        "sync",
        Setup {
            sync: true,
            direct: false,
        },
    )?;

    run_set(
        "direct",
        Setup {
            sync: false,
            direct: true,
        },
    )?;

    Ok(())
}
