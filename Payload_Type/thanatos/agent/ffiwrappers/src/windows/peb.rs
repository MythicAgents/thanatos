use super::cffiheaders::ntdll;

pub struct Peb(&'static ntdll::PEB);

#[allow(unused_assignments)]
fn current_peb() -> *const ntdll::PEB {
    let mut ppeb: *const ntdll::PEB = std::ptr::null();
    #[cfg(target_arch = "x86_64")]
    unsafe {
        std::arch::asm!("mov {x}, gs:[0x60]", x = out(reg) ppeb)
    };

    ppeb
}

impl Peb {
    pub fn new() -> Peb {
        let ppeb = current_peb();
        Peb(unsafe { &*ppeb })
    }

    pub const fn os_major_version(&self) -> u32 {
        self.0.OSMajorVersion
    }

    pub const fn os_minor_version(&self) -> u32 {
        self.0.OSMinorVersion
    }

    pub const fn os_build_number(&self) -> u16 {
        self.0.OSBuildNumber
    }

    pub const fn os_platform_id(&self) -> u32 {
        self.0.OSPlatformId
    }
}

impl Default for Peb {
    fn default() -> Self {
        Self::new()
    }
}
