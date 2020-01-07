#![allow(clippy::ptr_arg)]

use std::{
    ffi::{OsStr, OsString},
    mem,
    path::{Path, PathBuf},
};

// SAFETY: These assume that `OsStr`/`Path` and `OsString`/`PathBuf` have the
// same representation as `[u8]` and `Vec<u8>` respectively.
//
// - Unix-like systems have `std::os::unix::ffi::{OsStrExt, OsStringExt}`,
//   which allow for converting freely between these types.
//
// - Windows internally uses `Wtf8Buf` and `Wtf8`, which are backed by
//   `Vec<u8>` and `[u8]` respectively.

pub trait ByteRepr {
    unsafe fn from_bytes(bytes: &[u8]) -> &Self;
    unsafe fn from_bytes_mut(bytes: &mut [u8]) -> &mut Self;

    fn as_bytes(&self) -> &[u8];
    unsafe fn as_bytes_mut(&mut self) -> &mut [u8];

    #[inline(always)]
    fn is_empty(&self) -> bool {
        self.as_bytes().is_empty()
    }

    #[inline(always)]
    fn as_ptr(&self) -> *const u8 {
        self.as_bytes().as_ptr()
    }
}

pub trait ByteBufRepr {
    unsafe fn from_bytes(bytes: Vec<u8>) -> Self;
    unsafe fn from_bytes_ref(bytes: &Vec<u8>) -> &Self;
    unsafe fn from_bytes_mut(bytes: &mut Vec<u8>) -> &mut Self;

    fn into_bytes(self) -> Vec<u8>;
    fn as_bytes(&self) -> &Vec<u8>;
    unsafe fn as_bytes_mut(&mut self) -> &mut Vec<u8>;

    #[inline(always)]
    fn is_empty(&self) -> bool {
        self.as_bytes().is_empty()
    }

    #[inline(always)]
    fn as_ptr(&self) -> *const u8 {
        self.as_bytes().as_ptr()
    }

    #[inline(always)]
    unsafe fn set_byte_len(&mut self, len: usize) {
        self.as_bytes_mut().set_len(len);
    }

    #[inline(always)]
    unsafe fn set_end(&mut self, end: *const u8) {
        let len = end.sub(self.as_ptr() as usize) as usize;
        self.set_byte_len(len);
    }
}

macro_rules! impl_byte_repr {
    ($($t:ty),+) => {
        $(impl ByteRepr for $t {
            #[inline(always)]
            unsafe fn from_bytes(bytes: &[u8]) -> &Self {
                &*(bytes as *const _ as *const _)
            }

            #[inline(always)]
            unsafe fn from_bytes_mut(bytes: &mut [u8]) -> &mut Self {
                &mut *(bytes as *mut _ as *mut _)
            }

            #[inline(always)]
            fn as_bytes(&self) -> &[u8] {
                unsafe { &*(self as *const _ as *const _) }
            }

            #[inline(always)]
            unsafe fn as_bytes_mut(&mut self) -> &mut [u8] {
                &mut *(self as *mut _ as *mut _)
            }
        })+
    };
}

macro_rules! impl_byte_buf_repr {
    ($($t:ty),+) => {
        $(impl ByteBufRepr for $t {
            #[inline(always)]
            unsafe fn from_bytes(bytes: Vec<u8>) -> Self {
                mem::transmute(bytes)
            }

            #[inline(always)]
            unsafe fn from_bytes_ref(bytes: &Vec<u8>) -> &Self {
                &*(bytes as *const _ as *const _)
            }

            #[inline(always)]
            unsafe fn from_bytes_mut(bytes: &mut Vec<u8>) -> &mut Self {
                &mut *(bytes as *mut _ as *mut _)
            }

            #[inline(always)]
            fn into_bytes(self) -> Vec<u8> {
                unsafe { mem::transmute(self) }
            }

            #[inline(always)]
            fn as_bytes(&self) -> &Vec<u8> {
                unsafe { &*(self as *const _ as *const _) }
            }

            #[inline(always)]
            unsafe fn as_bytes_mut(&mut self) -> &mut Vec<u8> {
                &mut *(self as *mut _ as *mut _)
            }
        })+
    }
}

impl_byte_repr!(OsStr, Path);
impl_byte_buf_repr!(OsString, PathBuf);
