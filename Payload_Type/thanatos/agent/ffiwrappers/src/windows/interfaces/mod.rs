use crate::internal::SealedTrait;

mod socket;

mod adapters;
pub use adapters::*;

mod unicast;
pub use unicast::*;

pub trait AdapterFamily: SealedTrait {
    const VALUE: u16;
}

pub mod adaptertypes {
    use super::AdapterFamily;
    use crate::socket::{AfInet, AfInet6, AfUnspec, SockAddrFamily};
    use windows::Win32::Networking::WinSock::{
        AF_INET, AF_INET6, AF_UNSPEC, SOCKADDR_IN, SOCKADDR_IN6,
    };

    impl AdapterFamily for AfInet {
        const VALUE: u16 = AF_INET.0;
    }
    impl SockAddrFamily for AfInet {
        type Inner = SOCKADDR_IN;
    }

    impl AdapterFamily for AfInet6 {
        const VALUE: u16 = AF_INET6.0;
    }

    impl SockAddrFamily for AfInet6 {
        type Inner = SOCKADDR_IN6;
    }

    impl AdapterFamily for AfUnspec {
        const VALUE: u16 = AF_UNSPEC.0;
    }
}
