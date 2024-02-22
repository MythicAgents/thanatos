use std::{ffi::c_void, mem::MaybeUninit};

use windows::{
    Wdk::System::SystemServices::RtlGetVersion,
    Win32::System::{
        SystemInformation::{
            GetNativeSystemInfo, GetProductInfo, OSVERSIONINFOEXW, OSVERSIONINFOW, OS_PRODUCT_TYPE,
            PROCESSOR_ARCHITECTURE, PROCESSOR_ARCHITECTURE_AMD64, PROCESSOR_ARCHITECTURE_ARM,
            PROCESSOR_ARCHITECTURE_ARM64, PROCESSOR_ARCHITECTURE_IA64,
            PROCESSOR_ARCHITECTURE_INTEL, PRODUCT_CORE, PRODUCT_DATACENTER_EVALUATION_SERVER,
            PRODUCT_DATACENTER_SERVER, PRODUCT_DATACENTER_SERVER_CORE,
            PRODUCT_DATACENTER_SERVER_CORE_V, PRODUCT_DATACENTER_SERVER_V, PRODUCT_EDUCATION,
            PRODUCT_ENTERPRISE, PRODUCT_ENTERPRISE_EVALUATION, PRODUCT_ENTERPRISE_SERVER,
            PRODUCT_ENTERPRISE_SERVER_CORE, PRODUCT_ENTERPRISE_SERVER_CORE_V,
            PRODUCT_ENTERPRISE_SERVER_V, PRODUCT_HOME_BASIC, PRODUCT_PROFESSIONAL,
            PRODUCT_PRO_WORKSTATION, PRODUCT_STANDARD_EVALUATION_SERVER, PRODUCT_STANDARD_SERVER,
            PRODUCT_STANDARD_SERVER_CORE_V, PRODUCT_STANDARD_SERVER_V, SYSTEM_INFO,
        },
        SystemServices::{PRODUCT_PRO_FOR_EDUCATION, PRODUCT_STANDARD_SERVER_CORE},
    },
};

#[repr(u16)]
pub enum ProcessorArchitecture {
    Amd64 = PROCESSOR_ARCHITECTURE_AMD64.0,
    Arm = PROCESSOR_ARCHITECTURE_ARM.0,
    Arm64 = PROCESSOR_ARCHITECTURE_ARM64.0,
    Ia64 = PROCESSOR_ARCHITECTURE_IA64.0,
    Intel = PROCESSOR_ARCHITECTURE_INTEL.0,
    Other(PROCESSOR_ARCHITECTURE),
}

impl ProcessorArchitecture {
    pub const fn from_value(value: u16) -> ProcessorArchitecture {
        match PROCESSOR_ARCHITECTURE(value) {
            PROCESSOR_ARCHITECTURE_AMD64 => Self::Amd64,
            PROCESSOR_ARCHITECTURE_ARM => Self::Arm,
            PROCESSOR_ARCHITECTURE_ARM64 => Self::Arm64,
            PROCESSOR_ARCHITECTURE_IA64 => Self::Ia64,
            PROCESSOR_ARCHITECTURE_INTEL => Self::Intel,
            o => Self::Other(o),
        }
    }
}

#[derive(Debug)]
#[repr(u32)]
pub enum ProductType {
    Other(u32),
    Core = PRODUCT_CORE.0,
    Professional = PRODUCT_PROFESSIONAL.0,
    Education = PRODUCT_EDUCATION.0,
    Enterprise = PRODUCT_ENTERPRISE.0,
    ProWorkstation = PRODUCT_PRO_WORKSTATION.0,
    ProForEducation = PRODUCT_PRO_FOR_EDUCATION,
    EnterpriseEvaluation = PRODUCT_ENTERPRISE_EVALUATION.0,
    DatacenterServer = PRODUCT_DATACENTER_SERVER.0,
    DatacenterEvaluationServer = PRODUCT_DATACENTER_EVALUATION_SERVER.0,
    DatacenterServerCore = PRODUCT_DATACENTER_SERVER_CORE.0,
    DatacenterServerCoreV = PRODUCT_DATACENTER_SERVER_CORE_V.0,
    DatacenterServerV = PRODUCT_DATACENTER_SERVER_V.0,
    EnterpriseServer = PRODUCT_ENTERPRISE_SERVER.0,
    EnterpriseServerCore = PRODUCT_ENTERPRISE_SERVER_CORE.0,
    EnterpriseServerCoreV = PRODUCT_ENTERPRISE_SERVER_CORE_V.0,
    EnterpriseServerV = PRODUCT_ENTERPRISE_SERVER_V.0,
    HomeBasic = PRODUCT_HOME_BASIC.0,
    StandardServer = PRODUCT_STANDARD_SERVER.0,
    StandardEvaluationServer = PRODUCT_STANDARD_EVALUATION_SERVER.0,
    StandardServerCore = PRODUCT_STANDARD_SERVER_CORE,
    StandardServerCoreV = PRODUCT_STANDARD_SERVER_CORE_V.0,
    StandardServerV = PRODUCT_STANDARD_SERVER_V.0,
}

impl ProductType {
    pub const fn from_value(value: OS_PRODUCT_TYPE) -> ProductType {
        match value {
            PRODUCT_CORE => Self::Core,
            PRODUCT_DATACENTER_EVALUATION_SERVER => Self::DatacenterEvaluationServer,
            PRODUCT_DATACENTER_SERVER => Self::DatacenterServer,
            PRODUCT_DATACENTER_SERVER_CORE => Self::DatacenterServerCore,
            PRODUCT_DATACENTER_SERVER_CORE_V => Self::DatacenterServerCoreV,
            PRODUCT_DATACENTER_SERVER_V => Self::DatacenterServerV,
            PRODUCT_EDUCATION => Self::Education,
            PRODUCT_ENTERPRISE => Self::Enterprise,
            PRODUCT_ENTERPRISE_EVALUATION => Self::EnterpriseEvaluation,
            PRODUCT_ENTERPRISE_SERVER => Self::EnterpriseServer,
            PRODUCT_ENTERPRISE_SERVER_CORE => Self::EnterpriseServerCore,
            PRODUCT_ENTERPRISE_SERVER_CORE_V => Self::EnterpriseServerCoreV,
            PRODUCT_ENTERPRISE_SERVER_V => Self::EnterpriseServerV,
            PRODUCT_HOME_BASIC => Self::HomeBasic,
            OS_PRODUCT_TYPE(PRODUCT_PRO_FOR_EDUCATION) => Self::ProForEducation,
            PRODUCT_PRO_WORKSTATION => Self::ProWorkstation,
            PRODUCT_PROFESSIONAL => Self::Professional,
            PRODUCT_STANDARD_SERVER => Self::StandardServer,
            PRODUCT_STANDARD_EVALUATION_SERVER => Self::StandardEvaluationServer,
            OS_PRODUCT_TYPE(PRODUCT_STANDARD_SERVER_CORE) => Self::StandardServerCore,
            PRODUCT_STANDARD_SERVER_CORE_V => Self::StandardServerCoreV,
            PRODUCT_STANDARD_SERVER_V => Self::StandardServerV,
            _ => Self::Other(value.0),
        }
    }
}

#[repr(transparent)]
pub struct SystemInfo(SYSTEM_INFO);

impl SystemInfo {
    pub fn new() -> SystemInfo {
        let mut info: MaybeUninit<SYSTEM_INFO> = MaybeUninit::zeroed();
        unsafe { GetNativeSystemInfo(info.as_mut_ptr()) };
        Self(unsafe { info.assume_init() })
    }

    pub const fn processor_architecture(&self) -> ProcessorArchitecture {
        ProcessorArchitecture::from_value(unsafe {
            self.0.Anonymous.Anonymous.wProcessorArchitecture.0
        })
    }

    pub const fn page_size(&self) -> u32 {
        self.0.dwPageSize
    }

    pub const fn minimum_application_address(&self) -> *mut c_void {
        self.0.lpMinimumApplicationAddress
    }

    pub const fn maximum_application_address(&self) -> *mut c_void {
        self.0.lpMaximumApplicationAddress
    }

    pub const fn active_processor_mask(&self) -> usize {
        self.0.dwActiveProcessorMask
    }

    pub const fn number_of_processors(&self) -> u32 {
        self.0.dwNumberOfProcessors
    }

    pub const fn processor_type(&self) -> u32 {
        self.0.dwProcessorType
    }

    pub const fn allocation_granularity(&self) -> u32 {
        self.0.dwAllocationGranularity
    }

    pub const fn processor_level(&self) -> u32 {
        self.0.dwAllocationGranularity
    }

    pub const fn processor_revision(&self) -> u16 {
        self.0.wProcessorRevision
    }
}

impl Default for SystemInfo {
    fn default() -> Self {
        Self::new()
    }
}

#[repr(transparent)]
pub struct OsVersionInfo(OSVERSIONINFOEXW);

impl OsVersionInfo {
    pub fn new() -> OsVersionInfo {
        let mut version_buffer = OSVERSIONINFOEXW::default();
        unsafe {
            RtlGetVersion(&mut version_buffer as *mut OSVERSIONINFOEXW as *mut OSVERSIONINFOW)
        };
        OsVersionInfo(version_buffer)
    }

    pub const fn major_version(&self) -> u32 {
        self.0.dwMajorVersion
    }

    pub const fn minor_version(&self) -> u32 {
        self.0.dwMinorVersion
    }

    pub const fn build_number(&self) -> u32 {
        self.0.dwBuildNumber
    }

    pub const fn service_pack_major(&self) -> u16 {
        self.0.wServicePackMajor
    }

    pub const fn service_pack_minor(&self) -> u16 {
        self.0.wServicePackMinor
    }

    pub fn product_type(&self) -> ProductType {
        let mut returned_product_type = OS_PRODUCT_TYPE(0u32);

        let _ = unsafe {
            GetProductInfo(
                self.major_version(),
                self.minor_version(),
                self.service_pack_major() as u32,
                self.service_pack_minor() as u32,
                &mut returned_product_type,
            )
        };

        ProductType::from_value(returned_product_type)
    }
}

impl Default for OsVersionInfo {
    fn default() -> Self {
        Self::new()
    }
}
