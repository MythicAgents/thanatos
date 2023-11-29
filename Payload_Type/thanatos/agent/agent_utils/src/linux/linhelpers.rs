//! Helper functions for Linux

use std::marker::PhantomData;

use crate::errors::ThanatosError;

/// RAII wrapper around libc getifaddrs
pub struct IfAddrs<'a> {
    /// Head of the interfaces linked list
    if_head: *mut libc::ifaddrs,
    /// Pointer to the current element of the interface linked list
    if_curr: *mut libc::ifaddrs,

    /// Marker to signify that the data is owned. Needed for the iterator
    _marker: PhantomData<&'a libc::ifaddrs>,
}

impl IfAddrs<'_> {
    /// Constructs a new `IfAddrs` structure
    pub fn new() -> Result<Self, ThanatosError> {
        let mut interfaces: *mut libc::ifaddrs = std::ptr::null_mut();

        if unsafe { libc::getifaddrs(&mut interfaces) } != 0 {
            return Err(ThanatosError::os_error());
        }

        Ok(Self {
            if_head: interfaces,
            if_curr: interfaces,
            _marker: PhantomData,
        })
    }
}

impl Drop for IfAddrs<'_> {
    fn drop(&mut self) {
        unsafe {
            libc::freeifaddrs(self.if_head);
        }
    }
}

impl<'a> Iterator for IfAddrs<'a> {
    type Item = &'a libc::ifaddrs;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.if_curr.is_null() {
            let if_curr_cpy = self.if_curr;
            self.if_curr = unsafe { *if_curr_cpy }.ifa_next;

            Some(unsafe { &*if_curr_cpy })
        } else {
            None
        }
    }
}

/// Wrapper for Linux unnamed (anonyous) pipes.
#[allow(non_snake_case)]
pub mod UnnamedPipe {
    use std::{
        io::{Read, Write},
        os::fd::{AsRawFd, FromRawFd, OwnedFd},
    };

    use crate::errors::ThanatosError;

    /// Creates a new pair of unnamed pipes.
    pub fn create(
        flags: Option<i32>,
    ) -> Result<(UnnamedPipeSender, UnnamedPipeReceiver), ThanatosError> {
        let mut pipefds = [0i32; 2];

        if unsafe { libc::pipe2(pipefds.as_mut_ptr().cast(), flags.unwrap_or(0)) } != 0 {
            return Err(ThanatosError::os_error());
        }

        Ok((
            unsafe { UnnamedPipeSender::from_raw_fd(pipefds[1]) },
            unsafe { UnnamedPipeReceiver::from_raw_fd(pipefds[0]) },
        ))
    }

    /// Sender side of an unnamed pipe.
    pub struct UnnamedPipeSender {
        /// Underlying file descriptor for the pipe.
        fd: OwnedFd,
    }

    impl FromRawFd for UnnamedPipeSender {
        unsafe fn from_raw_fd(fd: std::os::fd::RawFd) -> Self {
            Self {
                fd: OwnedFd::from_raw_fd(fd),
            }
        }
    }

    impl Drop for UnnamedPipeSender {
        fn drop(&mut self) {
            unsafe { libc::close(self.fd.as_raw_fd()) };
        }
    }

    impl Write for UnnamedPipeSender {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            let ret = unsafe { libc::write(self.fd.as_raw_fd(), buf.as_ptr().cast(), buf.len()) };
            if ret == -1 {
                Err(std::io::Error::last_os_error())
            } else {
                Ok(ret as usize)
            }
        }

        /// Pipes don't support flushing
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    impl ToString for UnnamedPipeSender {
        fn to_string(&self) -> String {
            self.fd.as_raw_fd().to_string()
        }
    }

    /// Receiver side of an unnamed pipe.
    pub struct UnnamedPipeReceiver {
        /// Underlying file descriptor for the pipe.
        fd: OwnedFd,
    }

    impl FromRawFd for UnnamedPipeReceiver {
        unsafe fn from_raw_fd(fd: std::os::fd::RawFd) -> Self {
            Self {
                fd: OwnedFd::from_raw_fd(fd),
            }
        }
    }

    impl Drop for UnnamedPipeReceiver {
        fn drop(&mut self) {
            unsafe { libc::close(self.fd.as_raw_fd()) };
        }
    }

    impl Read for UnnamedPipeReceiver {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            let ret =
                unsafe { libc::read(self.fd.as_raw_fd(), buf.as_mut_ptr().cast(), buf.len()) };
            if ret == -1 {
                Err(std::io::Error::last_os_error())
            } else {
                Ok(ret as usize)
            }
        }
    }
}
