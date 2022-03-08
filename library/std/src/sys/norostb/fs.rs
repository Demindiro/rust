/// ## Path format
///
/// ```
/// table/[tags,...[/id]]
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
use norostb_rt::kernel::syscall::{self, Handle, Id, ObjectInfo, QueryHandle, TableId, TableInfo};

#[derive(Debug)]
pub struct File {
    handle: Handle,
}

const TABLE_OBJECT_SEPARATOR: u8 = b'/';
const TAG_SEPARATOR: u8 = b',';

#[derive(Clone, Debug)]
pub enum FileAttr {
    Table { entries: u64 },
    Object { size: u64 },
}

#[derive(Clone, Debug)]
pub enum ReadDir {
    None,
    Tables(Option<TableId>),
    Objects { table_id: TableId, table_info: TableInfo, query: QueryHandle },
}

#[derive(Clone, Debug)]
pub enum DirEntry {
    Table { id: TableId, info: TableInfo },
    Object { table_id: TableId, table_info: TableInfo, id: Id, name: OsString },
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
        let mut buf = [0; 4096];
        match mem::replace(self, Self::None) {
            Self::None => None,
            Self::Tables(tbl) => syscall::next_table(tbl).map(|(id, info)| {
                *self = Self::Tables(Some(id));
                Ok(DirEntry::Table { id, info })
            }),
            Self::Objects { table_id, table_info, query } => {
                let mut info = ObjectInfo::new(&mut buf);
                match syscall::query_next(query, &mut info) {
                    Ok(()) => {
                        let inner = if info.tags_count() == 0 {
                            Vec::new()
                        } else {
                            let len =
                                (0..info.tags_count()).map(|i| info.tag(i).len()).sum::<usize>()
                                    + info.tags_count()
                                    - 1;
                            let mut inner = Vec::with_capacity(len);
                            inner.extend(info.tag(0));
                            for t in (1..info.tags_count()).map(|i| info.tag(i)) {
                                inner.push(TAG_SEPARATOR);
                                inner.extend(t);
                            }
                            inner
                        };
                        let name = OsString::from_inner(Buf { inner }).into();
                        *self = Self::Objects { table_id, table_info: table_info.clone(), query };
                        Some(Ok(DirEntry::Object { table_id, table_info, name, id: info.id }))
                    }
                    // TODO don't hardcode error code
                    Err((e, 0)) if e.get() == 1 => None,
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
            Self::Object { table_info, name, id, .. } => {
                let inner = table_info
                    .name()
                    .iter()
                    .chain(&[TABLE_OBJECT_SEPARATOR])
                    .chain(&name.as_inner().inner)
                    .chain(&[TABLE_OBJECT_SEPARATOR])
                    .chain(id_to_ascii(*id, &mut [0; 20]))
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
            Self::Object { id, .. } => {
                OsString::from_inner(Buf { inner: id.0.to_string().into() }).into()
            }
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
            // syscall::create takes only a table ID and a string representing tags
            let mut path = path_inner(path).splitn(2, |c| *c == b'/');
            let table = path.next().expect("at least one match");
            let tags = if let Some(p) = path.next() {
                p
            } else {
                return Err(io::const_io_error!(io::ErrorKind::Other, "expected full path"));
            };
            let table = find_table(table)?.0;
            syscall::create(table, tags, crate::time::Duration::MAX)
                .map_err(|_| io::const_io_error!(io::ErrorKind::Other, "failed to open object"))
                .map(|handle| File { handle })
        } else {
            // Find a unique ID
            let (table_id, id) = find_unique_object_with_path(path)?;
            syscall::open(table_id, id)
                .map_err(|_| io::const_io_error!(io::ErrorKind::Other, "failed to open object"))
                .map(|handle| File { handle })
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
        syscall::read(self.handle, buf)
            .map_err(|_| io::const_io_error!(io::ErrorKind::Uncategorized, "TODO failed read"))
    }

    pub fn read_vectored(&self, _bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        unsupported()
    }

    pub fn is_read_vectored(&self) -> bool {
        false
    }

    pub fn read_buf(&self, buf: &mut ReadBuf<'_>) -> io::Result<()> {
        // SAFETY: we don't deinitialize any part of the buffer/
        let s = unsafe { buf.unfilled_mut() };
        let len = syscall::read_uninit(self.handle, s)
            .map_err(|_| io::const_io_error!(io::ErrorKind::Uncategorized, "TODO failed read"))?;
        // SAFETY: the kernel has initialized `len` bytes.
        unsafe {
            buf.assume_init(buf.filled().len() + len);
        }
        buf.add_filled(len);
        Ok(())
    }

    pub fn write(&self, buf: &[u8]) -> io::Result<usize> {
        syscall::write(self.handle, buf)
            .map_err(|_| io::const_io_error!(io::ErrorKind::Uncategorized, "TODO failed write"))
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

    pub fn seek(&self, _pos: SeekFrom) -> io::Result<u64> {
        // TODO
        unsupported()
    }

    pub fn duplicate(&self) -> io::Result<File> {
        syscall::duplicate_handle(self.handle)
            .map_err(|_| io::const_io_error!(io::ErrorKind::Uncategorized, "TODO failed duplicate"))
            .map(|handle| Self { handle })
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
    match split_path(path)? {
        SplitPath::None => Ok(ReadDir::Tables(None)),
        SplitPath::Table { table } => {
            let (table_id, table_info) = find_table(table)?;
            let query = syscall::query_table(table_id, &[])
                .map_err(|_| io::const_io_error!(io::ErrorKind::Other, "error querying table"))?;
            Ok(ReadDir::Objects { table_id, table_info, query })
        }
        SplitPath::Tags { table, tags } => {
            let mut t = [Default::default(); 256];
            let tags = split_tags(tags, &mut t)?;
            let (table_id, table_info) = find_table(table)?;
            let query = syscall::query_table(table_id, tags)
                .map_err(|_| io::const_io_error!(io::ErrorKind::Other, "error querying table"))?;
            Ok(ReadDir::Objects { table_id, table_info, query })
        }
        SplitPath::Id { table, tags, id } => todo!("{:?}/{:?}/{}", table, tags, id.0),
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
    let _ = find_unique_object_with_path(path)?;
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
    Tags { table: &'a [u8], tags: &'a [u8] },
    Id { table: &'a [u8], tags: &'a [u8], id: Id },
}

/// Split a path by table, tags and ID.
fn split_path(path: &Path) -> io::Result<SplitPath<'_>> {
    let path = path_inner(path);

    if path.is_empty() {
        return Ok(SplitPath::None);
    }

    fn split(s: &[u8], sep: u8) -> Option<(&[u8], &[u8])> {
        s.iter().position(|c| *c == sep).map(|i| {
            // TODO parse tags
            let (table, tags) = s.split_at(i);
            (table, &tags[1..])
        })
    }

    if let Some((table, tags)) = split(path, TABLE_OBJECT_SEPARATOR) {
        if let Some((tags, id)) = split(tags, TABLE_OBJECT_SEPARATOR) {
            let id = ascii_to_id(id)?;
            Ok(SplitPath::Id { table, tags, id })
        } else if tags.is_empty() {
            Ok(SplitPath::Table { table })
        } else {
            Ok(SplitPath::Tags { table, tags })
        }
    } else {
        Ok(SplitPath::Table { table: path })
    }
}

/// Get a reference to the underlying `[u8]` of a [`Path`].
fn path_inner(path: &Path) -> &[u8] {
    &path.as_os_str().as_inner().inner
}

/// Convert an ASCII string to an [`Id`].
fn ascii_to_id(s: &[u8]) -> io::Result<Id> {
    s.iter()
        .try_fold(0, |v, &d| match d {
            b'0'..=b'9' => Ok(v + u64::from(d - b'0')),
            _ => Err(io::const_io_error!(io::ErrorKind::InvalidInput, "invalid ID digit")),
        })
        .map(Id)
}

/// Convert an [`Id`] to an ASCII string.
fn id_to_ascii(id: Id, buf: &mut [u8; 20]) -> &[u8] {
    let mut id = id.0;
    let mut len = 0;
    for c in buf.iter_mut().rev() {
        *c = (id % 10) as u8 + b'0';
        id /= 10;
        len += 1;
        if id == 0 {
            break;
        }
    }
    &buf[buf.len() - len..]
}

/// Find exactly one object matching the given tags.
///
/// # Errors
///
/// There are no or multiple objects matching the tags.
fn find_unique_object(table_id: TableId, tags: &[u8]) -> io::Result<Id> {
    let mut t = [Default::default(); 256];
    let tags = split_tags(tags, &mut t)?;
    let query = syscall::query_table(table_id, tags)
        .map_err(|_| io::const_io_error!(io::ErrorKind::Other, "error querying table"))?;
    let mut info = ObjectInfo::default();
    syscall::query_next(query, &mut info)
        .map_err(|_| io::const_io_error!(io::ErrorKind::Other, "no object with tags"))?;
    if syscall::query_next(query, &mut ObjectInfo::default()).is_ok() {
        return Err(io::const_io_error!(io::ErrorKind::Other, "multiple objects with tags"));
    }
    return Ok(info.id);
}

/// Find exactly one object matching the given path.
///
/// # Errors
///
/// There are no or multiple objects matching the path.
fn find_unique_object_with_path(path: &Path) -> io::Result<(TableId, Id)> {
    // Find a unique ID
    match split_path(path)? {
        SplitPath::Id { id, table, .. } => Ok((find_table(table)?.0, id)),
        SplitPath::Tags { table, tags } => {
            let (table_id, _) = find_table(table)?;
            Ok((table_id, find_unique_object(table_id, tags)?))
        }
        SplitPath::Table { .. } | SplitPath::None => {
            Err(io::const_io_error!(io::ErrorKind::InvalidInput, "expected tags and/or id"))
        }
    }
}

/// Split a string of tags separated by commas (`','`).
fn split_tags<'a, 'b>(
    tags: &'a [u8],
    buf: &'b mut [syscall::Slice<'a, u8>; 256],
) -> io::Result<&'b [syscall::Slice<'a, u8>]> {
    let mut i = 0;
    for t in tags.split(|c| *c == TAG_SEPARATOR) {
        let e = io::const_io_error!(io::ErrorKind::InvalidInput, "too many tags");
        *buf.get_mut(i).ok_or(e)? = t.into();
        i += 1;
    }
    Ok(&buf[..i])
}
