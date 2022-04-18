use crate::sys_common::{AsInner, FromInner, IntoInner};
use crate::{fs, sys};

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

impl AsHandle for fs::File {
    fn as_handle(&self) -> Handle {
        self.as_inner().handle
    }
}

impl IntoHandle for fs::File {
    fn into_handle(self) -> Handle {
        self.into_inner().handle
    }
}

impl FromHandle for fs::File {
    unsafe fn from_handle(handle: Handle) -> Self {
        Self::from_inner(sys::fs::File { handle })
    }
}
