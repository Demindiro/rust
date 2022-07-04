use super::cvt_err;
/// ## Path format
///
/// ```
/// table/[path]
/// ```
///
/// ### Examples
///
/// ```
/// pci
/// pci/
/// pci/vendor-id:1234,device-id:1111
/// pci/vendor-id:1234,device-id:1111/8
/// pci//8
/// ```
use crate::ffi::OsString;
use crate::hash::Hash;
use crate::io::{self, IoSlice, IoSliceMut, ReadBuf, SeekFrom};
use crate::os::norostb::prelude::*;
use crate::path::{Path, PathBuf};
use crate::sys::time::SystemTime;
use crate::sys::unsupported;
use crate::sys_common::AsInner;
use norostb_rt::{io as rt_io, NewObject, Object};

#[derive(Debug)]
pub struct File(pub(crate) Object);

#[derive(Clone, Debug)]
pub enum FileAttr {}

#[derive(Debug)]
pub struct ReadDir(Option<Object>);

#[derive(Clone, Debug)]
pub struct DirEntry(OsString);

#[derive(Clone, Debug)]
pub struct OpenOptions {
    create: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FilePermissions(());

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FileType {
    Object,
}

#[derive(Debug)]
pub struct DirBuilder {}

impl FileAttr {
    pub fn size(&self) -> u64 {
        match self {
            _ => unreachable!(),
        }
    }

    pub fn perm(&self) -> FilePermissions {
        FilePermissions(())
    }

    pub fn file_type(&self) -> FileType {
        FileType::Object
    }

    pub fn modified(&self) -> io::Result<SystemTime> {
        unsupported()
    }

    pub fn accessed(&self) -> io::Result<SystemTime> {
        unsupported()
    }

    pub fn created(&self) -> io::Result<SystemTime> {
        unsupported()
    }
}

impl FilePermissions {
    pub fn readonly(&self) -> bool {
        false
    }

    pub fn set_readonly(&mut self, _readonly: bool) {}
}

impl FileType {
    pub fn is_dir(&self) -> bool {
        match self {
            Self::Object => false,
        }
    }

    pub fn is_file(&self) -> bool {
        match self {
            Self::Object => true,
        }
    }

    pub fn is_symlink(&self) -> bool {
        match self {
            Self::Object => false,
        }
    }
}

impl Iterator for ReadDir {
    type Item = io::Result<DirEntry>;

    fn next(&mut self) -> Option<io::Result<DirEntry>> {
        let mut vec = Vec::with_capacity(4096);
        match self.0.as_mut()?.read_uninit(vec.spare_capacity_mut()) {
            Ok((i, _)) if i.is_empty() => None,
            Ok((i, _)) => {
                let l = i.len();
                // SAFETY: all bytes in i are initialized and i is a slice of vec
                unsafe { vec.set_len(l) }
                Some(Ok(DirEntry(OsString::from_vec(vec))))
            }
            Err(e) => {
                self.0 = None;
                Some(Err(cvt_err(e)))
            }
        }
    }
}

impl DirEntry {
    pub fn path(&self) -> PathBuf {
        self.0.clone().into()
    }

    pub fn file_name(&self) -> OsString {
        // TODO this isn't quite accurate.
        self.0.clone().into()
    }

    pub fn metadata(&self) -> io::Result<FileAttr> {
        unsupported()
    }

    pub fn file_type(&self) -> io::Result<FileType> {
        unsupported()
    }
}

impl OpenOptions {
    pub fn new() -> OpenOptions {
        OpenOptions { create: false }
    }

    pub fn read(&mut self, _read: bool) {}
    pub fn write(&mut self, _write: bool) {}
    pub fn append(&mut self, _append: bool) {}
    pub fn truncate(&mut self, _truncate: bool) {}
    pub fn create(&mut self, create: bool) {
        self.create = create;
    }
    pub fn create_new(&mut self, create_new: bool) {
        self.create = create_new;
    }
}

impl File {
    pub fn open(path: &Path, opts: &OpenOptions) -> io::Result<File> {
        let root = rt_io::file_root().ok_or(super::ERR_UNSET)?;
        if opts.create { root.create(path_inner(path)) } else { root.open(path_inner(path)) }
            .map(File)
            .map_err(cvt_err)
    }

    pub fn file_attr(&self) -> io::Result<FileAttr> {
        unsupported()
    }

    pub fn fsync(&self) -> io::Result<()> {
        unsupported()
    }

    pub fn datasync(&self) -> io::Result<()> {
        unsupported()
    }

    pub fn truncate(&self, _size: u64) -> io::Result<()> {
        unsupported()
    }

    pub fn read(&self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.read(buf).map_err(cvt_err)
    }

    pub fn read_vectored(&self, _bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        unsupported()
    }

    pub fn is_read_vectored(&self) -> bool {
        false
    }

    pub fn read_buf(&self, buf: &mut ReadBuf<'_>) -> io::Result<()> {
        // SAFETY: we don't deinitialize any part of the buffer
        let s = unsafe { buf.unfilled_mut() };
        let len = self.0.read_uninit(s).map_err(cvt_err)?.0.len();
        // SAFETY: the kernel has initialized `len` bytes.
        unsafe {
            buf.assume_init(buf.filled().len() + len);
        }
        buf.add_filled(len);
        Ok(())
    }

    pub fn write(&self, buf: &[u8]) -> io::Result<usize> {
        self.0.write(buf).map_err(cvt_err)
    }

    pub fn write_vectored(&self, _bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        unsupported()
    }

    pub fn is_write_vectored(&self) -> bool {
        false
    }

    pub fn flush(&self) -> io::Result<()> {
        // TODO
        Ok(())
    }

    pub fn seek(&self, pos: SeekFrom) -> io::Result<u64> {
        let pos = match pos {
            SeekFrom::Start(n) => rt_io::SeekFrom::Start(n),
            SeekFrom::Current(n) => rt_io::SeekFrom::Current(n),
            SeekFrom::End(n) => rt_io::SeekFrom::End(n),
        };
        self.0.seek(pos).map_err(cvt_err)
    }

    pub fn duplicate(&self) -> io::Result<File> {
        Object::new(NewObject::Duplicate { handle: self.0.as_raw() }).map_err(cvt_err).map(Self)
    }

    pub fn set_permissions(&self, _perm: FilePermissions) -> io::Result<()> {
        unsupported()
    }
}

impl DirBuilder {
    pub fn new() -> DirBuilder {
        DirBuilder {}
    }

    pub fn mkdir(&self, _p: &Path) -> io::Result<()> {
        unsupported()
    }
}

pub fn readdir(path: &Path) -> io::Result<ReadDir> {
    let mut p;
    let path = path_inner(path);
    let path = if path.last() != Some(&b'/') {
        p = Vec::with_capacity(path.len() + 1);
        p.extend(path);
        p.push(b'/');
        &p[..]
    } else {
        path
    };
    rt_io::file_root()
        .ok_or(super::ERR_UNSET)?
        .open(path)
        .map(|o| ReadDir(Some(o)))
        .map_err(cvt_err)
}

pub fn unlink(p: &Path) -> io::Result<()> {
    rt_io::file_root().ok_or(super::ERR_UNSET)?.destroy(path_inner(p)).map(|_| ()).map_err(cvt_err)
}

pub fn rename(_old: &Path, _new: &Path) -> io::Result<()> {
    unsupported()
}

pub fn set_perm(_p: &Path, _perm: FilePermissions) -> io::Result<()> {
    unsupported()
}

pub fn rmdir(p: &Path) -> io::Result<()> {
    unlink(p)
}

pub fn remove_dir_all(path: &Path) -> io::Result<()> {
    rmdir(path)
}

pub fn try_exists(path: &Path) -> io::Result<bool> {
    readdir(path).map(|mut q| q.next().is_some())
}

pub fn readlink(_p: &Path) -> io::Result<PathBuf> {
    // Symlinks are not supported at all
    // UNIX returns "InvalidInput" which is rather confusing, so let's not do the same.
    Err(io::const_io_error!(io::ErrorKind::Uncategorized, "not a symlnk"))
}

pub fn symlink(_original: &Path, _link: &Path) -> io::Result<()> {
    unsupported()
}

pub fn link(_src: &Path, _dst: &Path) -> io::Result<()> {
    unsupported()
}

pub fn stat(_path: &Path) -> io::Result<FileAttr> {
    unsupported()
}

pub fn lstat(path: &Path) -> io::Result<FileAttr> {
    stat(path)
}

pub fn canonicalize(_p: &Path) -> io::Result<PathBuf> {
    unsupported()
}

pub fn copy(_from: &Path, _to: &Path) -> io::Result<u64> {
    unsupported()
}

/// Get a reference to the underlying `[u8]` of a [`Path`].
fn path_inner(path: &Path) -> &[u8] {
    &path.as_os_str().as_inner().inner
}
