use std::{marker::PhantomData, ptr::NonNull};

use crate::internal::SealedTrait;

pub trait SockAddrFamily: SealedTrait {
    type Inner;
}

pub struct AfInet;
impl SealedTrait for AfInet {}

pub struct AfInet6;
impl SealedTrait for AfInet6 {}

pub struct AfUnspec;
impl SealedTrait for AfUnspec {}

pub enum SockAddr<'a> {
    AfInet(SockAddrIn<'a, AfInet>),
    AfInet6(SockAddrIn<'a, AfInet6>),
}

#[repr(transparent)]
pub struct SockAddrIn<'a, S: SockAddrFamily> {
    pub(crate) addr: NonNull<S::Inner>,
    _marker: PhantomData<&'a S::Inner>,
}

impl<'a, S: SockAddrFamily> SockAddrIn<'a, S> {
    pub(crate) unsafe fn from_raw(addr: NonNull<S::Inner>) -> SockAddrIn<'a, S> {
        SockAddrIn {
            addr,
            _marker: PhantomData,
        }
    }
}
