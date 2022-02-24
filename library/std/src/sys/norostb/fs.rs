use crate::ffi::OsString;
use crate::fmt;
use crate::hash::Hash;
use crate::io::{self, IoSlice, IoSliceMut, ReadBuf, SeekFrom};
use crate::mem;
use crate::path::{Path, PathBuf};
use crate::sys::os_str::Buf;
use crate::sys::time::SystemTime;
use crate::sys::unsupported;
use crate::sys_common::FromInner;
use norostb_rt::kernel::syscall;

pub struct File(!);

#[derive(Clone, Debug)]
pub enum FileAttr {
    Table { entries: u64 },
}

#[derive(Debug)]
pub enum ReadDir {
    None,
    Tables(Option<syscall::TableId>),
}

pub enum DirEntry {
    Table(syscall::TableInfo),
}

#[derive(Clone, Debug)]
pub struct OpenOptions {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FilePermissions(());

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FileType {
    Table,
}

#[derive(Debug)]
pub struct DirBuilder {}

impl FileAttr {
    pub fn size(&self) -> u64 {
        match self {
            Self::Table { entries, .. } => *entries,
        }
    }

    pub fn perm(&self) -> FilePermissions {
        FilePermissions(())
    }

    pub fn file_type(&self) -> FileType {
        match self {
            Self::Table { .. } => FileType::Table,
        }
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
            Self::Table => true,
        }
    }

    pub fn is_file(&self) -> bool {
        match self {
            Self::Table => false,
        }
    }

    pub fn is_symlink(&self) -> bool {
        match self {
            Self::Table => false,
        }
    }
}

impl Iterator for ReadDir {
    type Item = io::Result<DirEntry>;

    fn next(&mut self) -> Option<io::Result<DirEntry>> {
        match mem::replace(self, Self::None) {
            Self::None => None,
            Self::Tables(tbl) => syscall::next_table(tbl).map(|(id, info)| {
                *self = Self::Tables(Some(id));
                Ok(DirEntry::Table(info))
            }),
        }
    }
}

impl DirEntry {
    pub fn path(&self) -> PathBuf {
        match self {
            Self::Table(tbl) => {
                let inner = tbl.name().iter().chain(b":").copied().collect();
                OsString::from_inner(Buf { inner }).into()
            }
        }
    }

    pub fn file_name(&self) -> OsString {
        match self {
            Self::Table(tbl) => {
                let inner = tbl.name().iter().copied().collect();
                OsString::from_inner(Buf { inner }).into()
            }
        }
    }

    pub fn metadata(&self) -> io::Result<FileAttr> {
        match self {
            Self::Table(_) => Ok(FileAttr::Table { entries: 0 }),
        }
    }

    pub fn file_type(&self) -> io::Result<FileType> {
        match self {
            Self::Table(_) => Ok(FileType::Table),
        }
    }
}

impl OpenOptions {
    pub fn new() -> OpenOptions {
        OpenOptions {}
    }

    pub fn read(&mut self, _read: bool) {}
    pub fn write(&mut self, _write: bool) {}
    pub fn append(&mut self, _append: bool) {}
    pub fn truncate(&mut self, _truncate: bool) {}
    pub fn create(&mut self, _create: bool) {}
    pub fn create_new(&mut self, _create_new: bool) {}
}

impl File {
    pub fn open(_path: &Path, _opts: &OpenOptions) -> io::Result<File> {
        unsupported()
    }

    pub fn file_attr(&self) -> io::Result<FileAttr> {
        self.0
    }

    pub fn fsync(&self) -> io::Result<()> {
        self.0
    }

    pub fn datasync(&self) -> io::Result<()> {
        self.0
    }

    pub fn truncate(&self, _size: u64) -> io::Result<()> {
        self.0
    }

    pub fn read(&self, _buf: &mut [u8]) -> io::Result<usize> {
        self.0
    }

    pub fn read_vectored(&self, _bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        self.0
    }

    pub fn is_read_vectored(&self) -> bool {
        self.0
    }

    pub fn read_buf(&self, _buf: &mut ReadBuf<'_>) -> io::Result<()> {
        self.0
    }

    pub fn write(&self, _buf: &[u8]) -> io::Result<usize> {
        self.0
    }

    pub fn write_vectored(&self, _bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        self.0
    }

    pub fn is_write_vectored(&self) -> bool {
        self.0
    }

    pub fn flush(&self) -> io::Result<()> {
        self.0
    }

    pub fn seek(&self, _pos: SeekFrom) -> io::Result<u64> {
        self.0
    }

    pub fn duplicate(&self) -> io::Result<File> {
        self.0
    }

    pub fn set_permissions(&self, _perm: FilePermissions) -> io::Result<()> {
        self.0
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

impl fmt::Debug for File {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0
    }
}

pub fn readdir(p: &Path) -> io::Result<ReadDir> {
    if p.as_os_str().is_empty() {
        // List all tables.
        Ok(ReadDir::Tables(None))
    } else {
        todo!()
    }
}

pub fn unlink(_p: &Path) -> io::Result<()> {
    unsupported()
}

pub fn rename(_old: &Path, _new: &Path) -> io::Result<()> {
    unsupported()
}

pub fn set_perm(_p: &Path, _perm: FilePermissions) -> io::Result<()> {
    unsupported()
}

pub fn rmdir(_p: &Path) -> io::Result<()> {
    unsupported()
}

pub fn remove_dir_all(_path: &Path) -> io::Result<()> {
    unsupported()
}

pub fn try_exists(_path: &Path) -> io::Result<bool> {
    unsupported()
}

pub fn readlink(_p: &Path) -> io::Result<PathBuf> {
    unsupported()
}

pub fn symlink(_original: &Path, _link: &Path) -> io::Result<()> {
    unsupported()
}

pub fn link(_src: &Path, _dst: &Path) -> io::Result<()> {
    unsupported()
}

pub fn stat(_p: &Path) -> io::Result<FileAttr> {
    unsupported()
}

pub fn lstat(_p: &Path) -> io::Result<FileAttr> {
    unsupported()
}

pub fn canonicalize(_p: &Path) -> io::Result<PathBuf> {
    unsupported()
}

pub fn copy(_from: &Path, _to: &Path) -> io::Result<u64> {
    unsupported()
}
