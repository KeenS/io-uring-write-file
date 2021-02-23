use io_uring_write_file::{iouring_write, iouring_write_tuned, write_std, Setup, Timer};
use libc::sync;
use std::fs::remove_file;
use std::io;

fn run_set(name: &str, setup: Setup) -> io::Result<()> {
    println!("{}", name);

    unsafe {
        sync();
    }
    print!("{:>15}:", "std");
    let filename = format!("../std_{}.text", name);
    let timer = Timer::start();
    write_std(&filename, setup)?;
    let elapsed = timer.stop();
    println!("{} ms", elapsed.as_millis());
    remove_file(filename)?;

    unsafe {
        sync();
    }
    print!("{:>15}:", "iouring");
    let filename = format!("../iouring_{}.text", name);
    let timer = Timer::start();
    iouring_write(&filename, setup)?;
    let elapsed = timer.stop();
    println!("{} ms", elapsed.as_millis());
    remove_file(filename)?;

    unsafe {
        sync();
    }
    print!("{:>15}:", "iouring_tuned");
    let filename = format!("../iouring_tuned_{}.text", name);
    let timer = Timer::start();
    iouring_write_tuned(&filename, setup)?;
    let elapsed = timer.stop();
    println!("{} ms", elapsed.as_millis());
    remove_file(filename)?;

    Ok(())
}

fn main() -> io::Result<()> {
    run_set(
        "default",
        Setup {
            fsync: false,
            direct: false,
            fallocate: false,
        },
    )?;

    run_set(
        "fsync",
        Setup {
            fsync: true,
            direct: false,
            fallocate: false,
        },
    )?;

    run_set(
        "direct",
        Setup {
            fsync: false,
            direct: true,
            fallocate: false,
        },
    )?;

    run_set(
        "fallocate",
        Setup {
            fsync: false,
            direct: false,
            fallocate: true,
        },
    )?;

    run_set(
        "fsync_fallocate",
        Setup {
            fsync: true,
            direct: false,
            fallocate: true,
        },
    )?;

    run_set(
        "direct_fallocate",
        Setup {
            fsync: false,
            direct: true,
            fallocate: true,
        },
    )?;

    Ok(())
}
