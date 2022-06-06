use crate::sys_common::{AsInner, FromInner, IntoInner};
use crate::{fs, net, sys};

pub type Handle = u32;

pub trait AsHandle {
    fn as_handle(&self) -> Handle;
}

pub trait IntoHandle {
    fn into_handle(self) -> Handle;
}

pub trait FromHandle {
    unsafe fn from_handle(handle: Handle) -> Self;
}

macro_rules! impl_h {
    ($ty:path, $systy:path) => {
        impl AsHandle for $ty {
            fn as_handle(&self) -> Handle {
                self.as_inner().0.as_raw()
            }
        }

        impl IntoHandle for $ty {
            fn into_handle(self) -> Handle {
                self.into_inner().0.into_raw()
            }
        }

        impl FromHandle for $ty {
            unsafe fn from_handle(handle: Handle) -> Self {
                Self::from_inner($systy(norostb_rt::table::Object::from_raw(handle)))
            }
        }
    };
}

impl_h!(fs::File, sys::fs::File);
impl_h!(net::TcpStream, sys::net::TcpStream);
impl_h!(net::TcpListener, sys::net::TcpListener);
impl_h!(net::UdpSocket, sys::net::UdpSocket);
