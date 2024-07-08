pub enum ForkProcess {
    Parent(i32),
    Child,
}

pub fn fork() -> Result<ForkProcess, std::io::Error> {
    let pid = unsafe { libc::fork() };
    match pid {
        -1 => Err(std::io::Error::last_os_error()),
        0 => Ok(ForkProcess::Child),
        o => Ok(ForkProcess::Parent(o)),
    }
}
