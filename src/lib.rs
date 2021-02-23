use io_uring::opcode::types::Fd;
use io_uring::opcode::{Fallocate, Fsync, Write, WriteFixed};
use io_uring::IoUring;
use libc::off_t;
use libc::{iovec, mmap, MAP_ANONYMOUS, MAP_PRIVATE, PROT_READ, PROT_WRITE};
use std::ffi::c_void;
use std::fs::File;
use std::fs::OpenOptions;
use std::io;
use std::io::prelude::Write as _;
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::io::AsRawFd;
use std::path::Path;
use std::ptr::null_mut;
use std::time::{Duration, Instant};

pub const TOTAL: usize = 5 * 1024 * 1024 * 1024;
pub const DATA_LEN: usize = 512 * 1024;
pub static DATA: [u8; DATA_LEN] = [0x61; DATA_LEN];
const NPAGES: usize = TOTAL / DATA_LEN;
pub const BATCH_SIZE: usize = 64;

#[derive(Debug)]
pub struct Timer(Instant);

impl Timer {
    pub fn start() -> Self {
        Timer(Instant::now())
    }

    pub fn stop(self) -> Duration {
        let now = Instant::now();
        now - self.0
    }
}

#[derive(Default, Clone, Copy)]
pub struct Setup {
    pub fsync: bool,
    pub direct: bool,
    pub fallocate: bool,
}

fn open_file(path: impl AsRef<Path>, setup: &Setup) -> io::Result<File> {
    let mut opt = OpenOptions::new();
    if setup.direct {
        opt.custom_flags(libc::O_DIRECT | libc::O_SYNC);
    }
    opt.write(true).create(true).open(path)
}

pub fn iouring_write(path: impl AsRef<Path>, setup: Setup) -> io::Result<()> {
    let mut uring = IoUring::new(BATCH_SIZE as u32 * 2)?;

    let file = open_file(path, &setup)?;

    let (submitter, sq, cq) = uring.split();

    let mut completed = 0;
    let mut waits = NPAGES;

    if setup.fallocate {
        unsafe {
            let entry = Fallocate::new(Fd(file.as_raw_fd()), TOTAL as i64).build();
            sq.available()
                .push(entry)
                .map_err(|_| io::Error::new(io::ErrorKind::Other, "failed to push entry to sq"))?;
            submitter.submit()?;
            waits += 1;
        }
    }

    debug_assert!(NPAGES % BATCH_SIZE == 0);
    let outer = NPAGES / BATCH_SIZE;
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

    if setup.fsync {
        unsafe {
            let entry = Fsync::new(Fd(file.as_raw_fd())).build();
            sq.available()
                .push(entry)
                .map_err(|_| io::Error::new(io::ErrorKind::Other, "failed to push entry to sq"))?;
        }
        submitter.submit()?;
        waits += 1;
    }

    while completed != waits {
        let rest = waits - completed;
        submitter.submit_and_wait(rest)?;
        let count = cq.available().count();
        // println!("count: {}", count);
        completed += count
    }
    Ok(())
}

pub fn iouring_write_tuned(path: impl AsRef<Path>, setup: Setup) -> io::Result<()> {
    let mut uring = IoUring::new(BATCH_SIZE as u32 * 2)?;

    let file = open_file(path, &setup)?;

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

    let mut completed = 0;
    let mut waits = NPAGES;

    if setup.fallocate {
        unsafe {
            let entry = Fallocate::new(Fd(file.as_raw_fd()), TOTAL as i64).build();
            sq.available()
                .push(entry)
                .map_err(|_| io::Error::new(io::ErrorKind::Other, "failed to push entry to sq"))?;
            submitter.submit()?;
            waits += 1;
        }
    }

    debug_assert!(NPAGES % BATCH_SIZE == 0);
    let outer = NPAGES / BATCH_SIZE;
    for i in 0..outer {
        for j in 0..BATCH_SIZE {
            let n = i * BATCH_SIZE + j;
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

    if setup.fsync {
        unsafe {
            let entry = Fsync::new(Fd(file.as_raw_fd())).build();
            sq.available()
                .push(entry)
                .map_err(|_| io::Error::new(io::ErrorKind::Other, "failed to push entry to sq"))?;
        }
        submitter.submit()?;
        waits += 1;
    }

    while completed != waits {
        let rest = waits - completed;
        submitter.submit_and_wait(rest)?;
        let count = cq.available().count();
        completed += count
    }
    Ok(())
}

pub fn write_std(path: impl AsRef<Path>, setup: Setup) -> io::Result<()> {
    let mut file = open_file(path, &setup)?;

    if setup.fallocate {
        file.set_len(TOTAL as u64)?;
    }

    for _ in 0..NPAGES {
        file.write_all(&DATA)?;
    }

    if setup.fsync {
        file.sync_all()?;
    }

    Ok(())
}
