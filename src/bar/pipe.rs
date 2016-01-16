use std::os::unix::io::{
    RawFd,
    AsRawFd,
    FromRawFd,
    IntoRawFd,
};
use std::io::{
    Write,
    Read,
    Result,
    Error,
};
use std::process::Stdio;
use std::mem;
use libc;

#[cfg(any(target_os = "macos",
          target_os = "ios",
          target_os = "freebsd",
          target_os = "dragonfly",
          target_os = "bitrig",
          target_os = "netbsd",
          target_os = "openbsd"))]
const FIOCLEX: libc::c_ulong = 0x20006601;

#[cfg(any(all(target_os = "linux",
              any(target_arch = "x86",
                  target_arch = "x86_64",
                  target_arch = "arm",
                  target_arch = "aarch64")),
          target_os = "android"))]
const FIOCLEX: libc::c_ulong = 0x5451;

#[cfg(all(target_os = "linux",
          any(target_arch = "mips",
              target_arch = "mipsel",
              target_arch = "powerpc")))]
const FIOCLEX: libc::c_ulong = 0x6601;

#[cfg(all(target_os = "linux",
          any(target_arch = "powerpc64",
              target_arch = "powerpc64le")))]
const FIOCLEX: libc::c_ulong = 0x20006601;

#[derive(Debug)]
pub struct PipeWriter(FileDesc);

#[derive(Debug)]
pub struct PipeReader(FileDesc);

pub fn pipe() -> Result<(PipeReader, PipeWriter)> {
    let mut fds = [0; 2];
    unsafe {
        if libc::pipe(fds.as_mut_ptr()) == 0 {
            Ok((PipeReader::from_raw_fd(fds[0]), PipeWriter::from_raw_fd(fds[1])))
        } else {
            Err(Error::last_os_error())
        }
    }
}

impl PipeWriter {
    /// Duplicates the underlying file descriptor, returning a new handle to it.
    pub fn dup(&self) -> Result<PipeWriter> {
        let new = try!(self.0.dup());
        new.set_cloexec();
        Ok(PipeWriter(new))
    }

    /// Creates an `Stdio` handle to the underlying pipe, duplicating the file descriptor.
    pub fn stdio(&self) -> Result<Stdio> {
        let new = try!(self.0.dup());
        new.set_cloexec();
        unsafe {
            Ok(Stdio::from_raw_fd(new.into_raw_fd()))
        }
    }
}

impl PipeReader {
    /// Duplicates the underlying file descriptor, returning a new handle to it.
    pub fn dup(&self) -> Result<PipeReader> {
        let new = try!(self.0.dup());
        new.set_cloexec();
        Ok(PipeReader(new))
    }

    /// Creates an `Stdio` handle to the underlying pipe, duplicating the file descriptor.
    pub fn stdio(&self) -> Result<Stdio> {
        let new = try!(self.0.dup());
        new.set_cloexec();
        unsafe {
            Ok(Stdio::from_raw_fd(new.into_raw_fd()))
        }
    }
}

impl AsRawFd for PipeReader {
    fn as_raw_fd(&self) -> RawFd {
        self.0.as_raw_fd()
    }
}

impl AsRawFd for PipeWriter {
    fn as_raw_fd(&self) -> RawFd {
        self.0.as_raw_fd()
    }
}

impl FromRawFd for PipeReader {
    unsafe fn from_raw_fd(fd: RawFd) -> PipeReader {
        let fd = FileDesc::from_raw_fd(fd);
        fd.set_cloexec();
        PipeReader(fd)
    }
}

impl FromRawFd for PipeWriter {
    unsafe fn from_raw_fd(fd: RawFd) -> PipeWriter {
        let fd = FileDesc::from_raw_fd(fd);
        fd.set_cloexec();
        PipeWriter(fd)
    }
}

impl IntoRawFd for PipeReader {
    fn into_raw_fd(self) -> RawFd {
        self.0.into_raw_fd()
    }
}

impl IntoRawFd for PipeWriter {
    fn into_raw_fd(self) -> RawFd {
        self.0.into_raw_fd()
    }
}

impl Read for PipeReader {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.0.read(buf)
    }
}

impl Write for PipeWriter {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.0.write(buf)
    }

    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}

#[derive(Debug)]
pub struct FileDesc {
    fd: RawFd,
}

impl AsRawFd for FileDesc {
    fn as_raw_fd(&self) -> RawFd {
        self.fd
    }
}

impl FromRawFd for FileDesc {
    unsafe fn from_raw_fd(fd: RawFd) -> FileDesc {
        FileDesc { fd: fd }
    }
}

impl IntoRawFd for FileDesc {
    fn into_raw_fd(self) -> RawFd {
        let fd = self.fd;
        mem::forget(self);
        fd
    }
}

impl FileDesc {
    pub fn read(&self, buf: &mut [u8]) -> Result<usize> {
        let ret = unsafe {
            libc::read(self.fd,
                       buf.as_mut_ptr() as *mut libc::c_void,
                       buf.len() as libc::size_t)
        };
        if ret == -1 {
            Err(Error::last_os_error())
        } else {
            Ok(ret as usize)
        }
    }

    pub fn write(&self, buf: &[u8]) -> Result<usize> {
        let ret = unsafe {
            libc::write(self.fd,
                        buf.as_ptr() as *const libc::c_void,
                        buf.len() as libc::size_t)
        };
        if ret == -1 {
            Err(Error::last_os_error())
        } else {
            Ok(ret as usize)
        }
    }

    pub fn set_cloexec(&self) {
        extern {
            fn ioctl(fd: libc::c_int, req: libc::c_ulong, ...) -> libc::c_int;
        }

        unsafe {
            let ret = ioctl(self.fd, FIOCLEX);
            debug_assert_eq!(ret, 0);
        }
    }

    pub fn dup(&self) -> Result<FileDesc> {
        let res = unsafe {
            libc::dup(self.fd)
        };
        if res == -1 {
            Err(Error::last_os_error())
        } else {
            unsafe {
                Ok(FileDesc::from_raw_fd(res))
            }
        }
    }
}

impl Drop for FileDesc {
    fn drop(&mut self) {
        let _ = unsafe {
            libc::close(self.fd)
        };
    }
}
