/// MSVC `__get_entropy()` function
#[cfg(all(windows, target_arch = "x86_64"))]
#[inline(always)]
pub fn get_entropy() -> usize {
    use windows::Win32::System::{
        Performance::QueryPerformanceCounter,
        SystemInformation::GetSystemTimeAsFileTime,
        Threading::{GetCurrentProcessId, GetCurrentThreadId},
    };

    let file_time = unsafe { GetSystemTimeAsFileTime() };
    let timestamp =
        ((file_time.dwHighDateTime as usize) << 0x20) | file_time.dwLowDateTime as usize;

    let mut seed = timestamp;
    seed ^= unsafe { GetCurrentThreadId() } as usize;
    seed ^= unsafe { GetCurrentProcessId() } as usize;

    let mut tsc = 0i64;
    let _ = unsafe { QueryPerformanceCounter(&mut tsc) };
    let tsc = tsc as usize;

    seed ^ (tsc & !0xffffffff) ^ tsc ^ (&seed as *const _ as usize)
}

/// MSVC `___get_entropy()` function
#[cfg(all(windows, target_arch = "x86"))]
#[inline(always)]
pub fn get_entropy() -> usize {
    use windows::Win32::System::{
        Performance::QueryPerformanceCounter,
        SystemInformation::GetSystemTimeAsFileTime,
        Threading::{GetCurrentProcessId, GetCurrentThreadId},
    };

    let file_time = unsafe { GetSystemTimeAsFileTime() };
    let mut seed = (file_time.dwLowDateTime as usize) ^ (file_time.dwHighDateTime as usize);
    seed ^= unsafe { GetCurrentThreadId() } as usize;
    seed ^= unsafe { GetCurrentProcessId() } as usize;

    let mut tsc = 0i64;
    let _ = unsafe { QueryPerformanceCounter(&mut tsc) };
    let tsc = tsc as usize;
    seed ^ (tsc & 0xffff) ^ (tsc ^ !0xffff) ^ (&seed as *const _ as usize)
}
