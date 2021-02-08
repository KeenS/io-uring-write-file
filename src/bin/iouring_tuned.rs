use io_uring::opcode::types::Fd;
use io_uring::opcode::WriteFixed;
use io_uring::IoUring;
use io_uring_write_file::{Timer, DATA, TOTAL};
use libc::off_t;
use libc::{iovec, mmap, MAP_ANONYMOUS, MAP_PRIVATE, PROT_READ, PROT_WRITE};
use std::ffi::c_void;
use std::fs::File;
use std::io;
use std::os::unix::io::AsRawFd;
use std::ptr::null_mut;

fn main() -> io::Result<()> {
    let mut uring = IoUring::new(2048)?;
    let file = File::create("iouring_tuned.text")?;

    let buf;
    unsafe {
        buf = mmap(
            null_mut::<c_void>(),
            DATA.len(),
            PROT_READ | PROT_WRITE,
            MAP_PRIVATE | MAP_ANONYMOUS,
            -1,
            0,
        );
        if (buf as isize) == -1 {
            return Err(io::Error::last_os_error());
        }

        for i in 0..DATA.len() {
            *((buf as *mut u8).offset(i as isize)) = DATA[i];
        }
    }

    let (submitter, sq, cq) = uring.split();

    submitter.register_buffers(&[iovec {
        iov_base: buf,
        iov_len: DATA.len(),
    }])?;

    submitter.register_files(&[file.as_raw_fd()])?;

    let npages = TOTAL / DATA.len();

    let timer = Timer::start();

    let mut completed = 0;

    debug_assert!(npages % 1024 == 0);
    let outer = npages / 1024;
    for i in 0..outer {
        for j in 0..1024 {
            let n = i * 1024 + j;
            unsafe {
                let entry =
                    WriteFixed::new(Fd(file.as_raw_fd()), buf as *const u8, DATA.len() as u32, 0)
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
        completed += count
    }

    let elapsed = timer.stop();
    println!("{} ms", elapsed.as_millis());

    Ok(())
}
