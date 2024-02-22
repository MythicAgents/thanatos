use ffiwrappers::windows::sysinfoapi::{OsVersionInfo, ProductType};

use crate::proto::checkin::windows_info::Product;

macro_rules! product_mapping {
    ($v:ident, $($field:ident),*) => {
        match $v {
            ffiwrappers::windows::sysinfoapi::ProductType::Other(o) => $crate::proto::checkin::windows_info::Product::Other(o),
            $(ffiwrappers::windows::sysinfoapi::ProductType::$field => $crate::proto::checkin::windows_info::Product::ProductType($crate::proto::checkin::WindowsProductType::$field.into()),)*
        }
    };
}

impl From<ProductType> for Product {
    fn from(value: ProductType) -> Self {
        product_mapping!(
            value,
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
}

pub fn build_number() -> u32 {
    let osversion = OsVersionInfo::new();
    osversion.build_number()
}

pub fn product() -> Product {
    let osversion = OsVersionInfo::new();
    osversion.product_type().into()
}
