use ffiwrappers::windows::sysinfoapi::OsVersionInfo;

use base_profile::msg::checkin::windows_info::Product;

macro_rules! map_product {
    ($v:ident, $($field:ident),*) => {
        match $v {
            ffiwrappers::windows::sysinfoapi::ProductType::Other(o) => base_profile::msg::checkin::windows_info::Product::Other(o),
            $(ffiwrappers::windows::sysinfoapi::ProductType::$field => base_profile::msg::checkin::windows_info::Product::ProductType(base_profile::msg::checkin::WindowsProductType::$field.into()),)*
        }
    };
}

pub fn build_number() -> u32 {
    let osversion = OsVersionInfo::new();
    osversion.build_number()
}

pub fn product() -> Product {
    let osversion = OsVersionInfo::new();
    let product_type = osversion.product_type();
    map_product!(
        product_type,
        Core,
        Professional,
        Education,
        Enterprise,
        ProWorkstation,
        ProForEducation,
        EnterpriseEvaluation,
        DatacenterServer,
        DatacenterEvaluationServer,
        DatacenterServerCore,
        DatacenterServerCoreV,
        DatacenterServerV,
        EnterpriseServer,
        EnterpriseServerCore,
        EnterpriseServerCoreV,
        EnterpriseServerV,
        HomeBasic,
        StandardServer,
        StandardEvaluationServer,
        StandardServerCore,
        StandardServerCoreV,
        StandardServerV
    )
}
