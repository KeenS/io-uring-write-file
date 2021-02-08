use io_uring::opcode::types::Fd;
use io_uring::opcode::Write;
use io_uring::IoUring;
use io_uring_write_file::{Timer, DATA, TOTAL};
use libc::off_t;
use std::fs::File;
use std::io;
use std::os::unix::io::AsRawFd;

const BATCH_SIZE: usize = 1024;

fn main() -> io::Result<()> {
    let file = File::create("iouring.text")?;

    let mut uring = IoUring::new(BATCH_SIZE as u32 * 2)?;

    let (submitter, sq, cq) = uring.split();

    let npages = TOTAL / DATA.len();

    let timer = Timer::start();
    let mut completed = 0;

    debug_assert!(npages % 1024 == 0);
    let outer = npages / BATCH_SIZE;
    for i in 0..outer {
        for j in 0..BATCH_SIZE {
            let n = i * BATCH_SIZE + j;
            unsafe {
                let entry = Write::new(Fd(file.as_raw_fd()), &DATA as *const u8, DATA.len() as u32)
                    .offset((n * DATA.len()) as off_t)
                    .build();
                sq.available().push(entry).map_err(|_| {
                    io::Error::new(io::ErrorKind::Other, "failed to push entry to sq")
                })?;
            }
        }
        submitter.submit()?;
        if i % 4 == 0 {
            completed += cq.available().count();
        }
    }

    while completed != npages {
        let rest = npages - completed;
        submitter.submit_and_wait(rest)?;
        let count = cq.available().count();
        // println!("count: {}", count);
        completed += count
    }

    let elapsed = timer.stop();
    println!("{} ms", elapsed.as_millis());

    Ok(())
}
