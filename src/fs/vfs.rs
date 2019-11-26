use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::any::Any;
use core::str;

pub type Result<T> = core::result::Result<T, FsError>;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum FsError {
    Unsupported,
    NotFile,
    NotDirectory,
    IsDirectory,
    EntryNotFound,
    EntryExists,
    NotSameFileSystem,
    DirectoryNotEmpty,
    Busy,
}

/// Abstract representation for any file system object, such as a directory or file.
pub trait INode: Any {
    /// Read bytes at `offset` into `buf`, returns the amount of bytes read.
    fn read_at(&self, offset: usize, buf: &mut [u8]) -> Result<usize>;

    /// Write bytes at `offset` from `buf`, returns the amount of bytes written.
    fn write_at(&self, offset: usize, buf: &[u8]) -> Result<usize>;

    // fn poll(&self) -> Result<PollStatus, FsError>;

    /// Returns the metadata of the inode.
    fn metadata(&self) -> Result<INodeMetadata>;

    /// Sets the metadata of an inode.
    fn set_metadata(&self, metadata: INodeMetadata) -> Result<()>;

    /// Sync data and metadata to the drive.
    fn sync_all(&self) -> Result<()>;

    /// Only sync data to the drive.
    fn sync_data(&self) -> Result<()>;

    /// Resize the file to the amount of bytes given.
    fn resize(&self, new_len: usize) -> Result<()>;

    // TODO: Data field for devices?
    /// Create a file if this inode is a directory.
    fn create(&self, name: &str, type_: FileType, permissions: u32) -> Result<Arc<dyn INode>>;

    /// Create a hard link to `other` if this inode is a directory.
    fn link(&self, name: &str, other: &Arc<dyn INode>) -> Result<()>;

    // TODO: Deletes file if it is the last hard link?
    /// Delete a hard link if this inode is a directory.
    fn unlink(&self, name: &str) -> Result<()>;

    /// Move a file from `self/old_name` to `target/new_name` if this inode is a directory. Can also
    /// be used to rename files by setting `target` to `self`.
    fn move_(&self, old_name: &str, target: &Arc<dyn INode>, new_name: &str) -> Result<()>;

    /// Find a file with name `name` and return it if this inode is a directory.
    fn find(&self, name: &str) -> Result<Arc<dyn INode>>;

    /// Get the name of the nth entry if this inode is a directory.
    fn get_entry(&self, index: usize) -> Result<String>;

    // fn io_control(&self, cmd: u32, data: usize) -> Result<()>;

    /// Get the parent filesystem this inode belongs to.
    fn filesystem(&self) -> Arc<dyn FileSystem>;

    fn as_any_ref(&self) -> &dyn Any;
}

impl dyn INode {
    pub fn downcast_ref<T: INode>(&self) -> Option<&T> {
        self.as_any_ref().downcast_ref::<T>()
    }

    pub fn list(&self) -> Result<Vec<String>> {
        let metadata = self.metadata()?;
        if metadata.type_ != FileType::Directory {
            return Err(FsError::NotDirectory);
        }

        let mut files = Vec::new();
        for i in 0.. {
            match self.get_entry(i) {
                Ok(file) => files.push(file),
                Err(_) => break,
            }
        }

        Ok(files)
    }

    /// Resolve a path starting from this inode (except when the path starts with /, then it starts
    /// from the root inode) and follow symbolic links at most `follow_times` times.
    pub fn resolve_follow(&self, path: &str, mut follow_times: usize) -> Result<Arc<dyn INode>> {
        let mut current = self.find(".")?;
        let mut path_rest = String::from(path);

        while &path_rest != "" {
            if current.metadata()?.type_ != FileType::Directory {
                return Err(FsError::NotDirectory);
            }

            if let Some('/') = path_rest.chars().next() {
                current = self.filesystem().root();
                path_rest = String::from(&path_rest[1..]);

                continue;
            }

            let next;
            match path_rest.find('/') {
                None => {
                    next = path_rest;
                    path_rest = String::new();
                },
                Some(pos) => {
                    next = String::from(&path_rest[..pos]);
                    path_rest = String::from(&path_rest[pos + 1..]);
                }
            }

            let inode = current.find(&next)?;

            if inode.metadata()?.type_ == FileType::SymbolicLink && follow_times > 0 {
                follow_times -= 1;

                let mut content = [0; 256];
                let len = inode.read_at(0, &mut content)?;
                let path = str::from_utf8(&content[..len]).map_err(|_| FsError::NotDirectory)?;

                let mut new_path = String::from(path);
                if let Some('/') = new_path.chars().last() {
                    new_path += "/";
                }

                new_path += &path_rest;

                path_rest = new_path;
            } else {
                current = inode
            }
        }

        Ok(current)
    }
}

pub trait FileSystem {
    /// Synchronize everything in this filesystem
    fn sync(&self) -> Result<()>;

    /// Get the root inode of this filesystem
    fn root(&self) -> Arc<dyn INode>;

    /// Get the metadata of the filesystem
    fn metadata(&self) -> FileSystemMetadata;
}

/// Common metadata every inode should provide.
#[derive(Debug, Copy, Clone)]
pub struct INodeMetadata {
    // pub dev: usize,

    /// Unique id for this inode
    pub inode: usize,

    /// Size in bytes
    pub size: usize,

    // pub blk_size: usize,
    // pub blocks: usize,

    /// Last access time
    pub access_time: Timespec,

    /// Last modification time
    pub modification_time: Timespec,

    // TODO: Difference with modification time?
    /// Last change time
    pub change_time: Timespec,

    /// Type of file
    pub type_: FileType,

    /// Permissions
    pub permissions: u16,

    /// Number of hard links
    pub links: usize,

    /// Owner user id
    pub uid: usize,

    /// Owner group id
    pub gid: usize,

    // pub rdev: usize,
}

/// Common metadata every filesystem should provide.
#[derive(Debug, Copy, Clone)]
pub struct FileSystemMetadata {
    // pub bsize: usize,
    // pub frsize: usize,
    // pub blocks: usize,
    // pub bfree: usize,
    // pub bavail: usize,

    /// Total number of unique inode ids on this filesystem
    pub files: usize,

    /// Total number of free inode ids on this filesystem
    pub files_free: usize,

    /// Maximum filename length
    pub max_name_len: usize,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Timespec {
    pub sec: i64,
    pub nanosec: i32,
}

/// The type of file for an inode.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum FileType {
    File,
    Directory,
    SymbolicLink,
    CharDevice,
    /*BlockDevice,
    NamedPipe,
    Socket,*/
}