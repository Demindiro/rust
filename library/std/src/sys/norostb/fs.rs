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
use crate::mem;
use crate::path::{Path, PathBuf};
use crate::sys::os_str::Buf;
use crate::sys::time::SystemTime;
use crate::sys::unsupported;
use crate::sys_common::{AsInner, FromInner};
use norostb_rt::{
    io as rt_io,
    table::{ObjectInfo, TableId, TableInfo, TableIter},
    Handle,
};

#[derive(Debug)]
pub struct File {
    pub(crate) handle: Handle,
}

const TABLE_OBJECT_SEPARATOR: u8 = b'/';

#[derive(Clone, Debug)]
pub enum FileAttr {
    Table { entries: u64 },
    Object { size: u64 },
}

#[derive(Debug)]
pub enum ReadDir {
    None,
    Tables(TableIter),
    Objects { table_id: TableId, table_info: TableInfo, query: Handle },
}

#[derive(Clone, Debug)]
pub enum DirEntry {
    Table { id: TableId, info: TableInfo },
    Object { table_id: TableId, table_info: TableInfo, name: OsString },
}

#[derive(Clone, Debug)]
pub struct OpenOptions {
    create: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FilePermissions(());

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FileType {
    Table,
    Object,
}

#[derive(Debug)]
pub struct DirBuilder {}

impl FileAttr {
    pub fn size(&self) -> u64 {
        match self {
            Self::Table { entries, .. } => *entries,
            Self::Object { size, .. } => *size,
        }
    }

    pub fn perm(&self) -> FilePermissions {
        FilePermissions(())
    }

    pub fn file_type(&self) -> FileType {
        match self {
            Self::Table { .. } => FileType::Table,
            Self::Object { .. } => FileType::Object,
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
            Self::Object => false,
        }
    }

    pub fn is_file(&self) -> bool {
        match self {
            Self::Table => false,
            Self::Object => true,
        }
    }

    pub fn is_symlink(&self) -> bool {
        match self {
            Self::Table => false,
            Self::Object => false,
        }
    }
}

impl Iterator for ReadDir {
    type Item = io::Result<DirEntry>;

    fn next(&mut self) -> Option<io::Result<DirEntry>> {
        match mem::replace(self, Self::None) {
            Self::None => None,
            Self::Tables(mut tbl) => tbl.next().map(|(id, info)| {
                *self = Self::Tables(tbl);
                Ok(DirEntry::Table { id, info })
            }),
            Self::Objects { table_id, table_info, query } => {
                let mut inner = Vec::with_capacity(4096);
                inner.resize(4096, 0);
                let mut info = ObjectInfo::new(&mut inner);
                match rt_io::query_next(query, &mut info) {
                    Ok(true) => {
                        inner.resize(info.path_len, 0);
                        let name = OsString::from_inner(Buf { inner }).into();
                        *self = Self::Objects { table_id, table_info: table_info.clone(), query };
                        Some(Ok(DirEntry::Object { table_id, table_info, name }))
                    }
                    Ok(false) => None,
                    Err(_) => unreachable!("kernel returned unknown error code"),
                }
            }
        }
    }
}

impl DirEntry {
    pub fn path(&self) -> PathBuf {
        match self {
            Self::Table { info, .. } => {
                let inner = info.name().into();
                OsString::from_inner(Buf { inner }).into()
            }
            Self::Object { table_info, name, .. } => {
                let inner = table_info
                    .name()
                    .iter()
                    .chain(&[TABLE_OBJECT_SEPARATOR])
                    .chain(&name.as_inner().inner)
                    .copied()
                    .collect();
                OsString::from_inner(Buf { inner }).into()
            }
        }
    }

    pub fn file_name(&self) -> OsString {
        match self {
            Self::Table { info, .. } => {
                let inner = info.name().iter().copied().collect();
                OsString::from_inner(Buf { inner }).into()
            }
            Self::Object { name, .. } => name.clone(),
        }
    }

    pub fn metadata(&self) -> io::Result<FileAttr> {
        match self {
            Self::Table { .. } => Ok(FileAttr::Table { entries: 0 }),
            Self::Object { .. } => Ok(FileAttr::Object { size: 0 }),
        }
    }

    pub fn file_type(&self) -> io::Result<FileType> {
        match self {
            Self::Table { .. } => Ok(FileType::Table),
            Self::Object { .. } => Ok(FileType::Object),
        }
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
        if opts.create {
            // syscall::create takes only a table ID and a string representing path
            let mut path = path_inner(path).splitn(2, |c| *c == b'/');
            let table = path.next().expect("at least one match");
            let path = if let Some(p) = path.next() {
                p
            } else {
                return Err(io::const_io_error!(io::ErrorKind::Other, "expected full path"));
            };
            let table = find_table(table)?.0;
            rt_io::create(table, path).map(|handle| File { handle }).map_err(cvt_err)
        } else {
            // Find a unique ID
            let (table_id, path) = split_into_table_and_path(path)?;
            rt_io::open(table_id, path).map(|handle| File { handle }).map_err(cvt_err)
        }
    }

    pub fn file_attr(&self) -> io::Result<FileAttr> {
        unsupported()
    }

    pub fn fsync(&self) -> io::Result<()> {
        // TODO
        Ok(())
    }

    pub fn datasync(&self) -> io::Result<()> {
        // TODO
        Ok(())
    }

    pub fn truncate(&self, _size: u64) -> io::Result<()> {
        unsupported()
    }

    pub fn read(&self, buf: &mut [u8]) -> io::Result<usize> {
        rt_io::read(self.handle, buf).map_err(cvt_err)
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
        let len = rt_io::read_uninit(self.handle, s).map_err(cvt_err)?;
        // SAFETY: the kernel has initialized `len` bytes.
        unsafe {
            buf.assume_init(buf.filled().len() + len);
        }
        buf.add_filled(len);
        Ok(())
    }

    pub fn write(&self, buf: &[u8]) -> io::Result<usize> {
        rt_io::write(self.handle, buf).map_err(cvt_err)
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
        rt_io::seek(self.handle, pos).map_err(cvt_err)
    }

    pub fn duplicate(&self) -> io::Result<File> {
        rt_io::duplicate(self.handle).map_err(cvt_err).map(|handle| Self { handle })
    }

    pub fn set_permissions(&self, _perm: FilePermissions) -> io::Result<()> {
        unsupported()
    }
}

impl Drop for File {
    fn drop(&mut self) {
        rt_io::close(self.handle);
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
    match split_path(path)? {
        SplitPath::None => Ok(ReadDir::Tables(TableIter::new().map_err(cvt_err)?)),
        SplitPath::Table { table } => {
            let (table_id, table_info) = find_table(table)?;
            let query = rt_io::query(table_id, &[]).map_err(cvt_err)?;
            Ok(ReadDir::Objects { table_id, table_info, query })
        }
        SplitPath::Path { table, path } => {
            let (table_id, table_info) = find_table(table)?;
            let query = rt_io::query(table_id, path).map_err(cvt_err)?;
            Ok(ReadDir::Objects { table_id, table_info, query })
        }
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

pub fn stat(path: &Path) -> io::Result<FileAttr> {
    let _ = split_into_table_and_path(path)?;
    // TODO stat doesn't actually exist yet though we do support FileAttr to some limited degree.
    Ok(FileAttr::Object { size: 0 })
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

/// Get table ID & info.
fn find_table(name: &[u8]) -> io::Result<(TableId, TableInfo)> {
    for entry in readdir(Path::new("")).unwrap().filter_map(Result::ok) {
        match entry {
            DirEntry::Table { id, info } => {
                if info.name() == name {
                    return Ok((id, info));
                }
            }
            DirEntry::Object { .. } => unreachable!(),
        }
    }
    Err(io::const_io_error!(io::ErrorKind::NotFound, "table not found"))
}

enum SplitPath<'a> {
    None,
    Table { table: &'a [u8] },
    Path { table: &'a [u8], path: &'a [u8] },
}

/// Split a path by table, path and ID.
fn split_path(path: &Path) -> io::Result<SplitPath<'_>> {
    let path = path_inner(path);

    if path.is_empty() {
        return Ok(SplitPath::None);
    }

    fn split(s: &[u8], sep: u8) -> Option<(&[u8], &[u8])> {
        s.iter().position(|c| *c == sep).map(|i| {
            // TODO parse path
            let (table, path) = s.split_at(i);
            (table, &path[1..])
        })
    }

    if let Some((table, path)) = split(path, TABLE_OBJECT_SEPARATOR) {
        if path.is_empty() {
            Ok(SplitPath::Table { table })
        } else {
            Ok(SplitPath::Path { table, path })
        }
    } else {
        Ok(SplitPath::Table { table: path })
    }
}

/// Get a reference to the underlying `[u8]` of a [`Path`].
fn path_inner(path: &Path) -> &[u8] {
    &path.as_os_str().as_inner().inner
}

/// Split a path into a table and object component
///
/// # Errors
///
/// - There is no separator.
/// - The table does not exist.
fn split_into_table_and_path(path: &Path) -> io::Result<(TableId, &[u8])> {
    // Find a unique ID
    match split_path(path)? {
        SplitPath::Path { path, table } => Ok((find_table(table)?.0, path)),
        SplitPath::Table { .. } | SplitPath::None => {
            Err(io::const_io_error!(io::ErrorKind::InvalidInput, "expected path and/or id"))
        }
    }
}
