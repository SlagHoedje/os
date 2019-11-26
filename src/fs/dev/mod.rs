use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::sync::{Arc, Weak};
use core::any::Any;

use spin::RwLock;

use fs::vfs::{FileSystem, FileSystemMetadata, FileType, FsError, INode, INodeMetadata, Result, Timespec};

pub mod zeronull;

/// The device file system usually mounted at `/dev/`
pub struct DevFS {
    devices: RwLock<BTreeMap<String, Arc<dyn INode>>>,
    self_ref: Weak<DevFS>,
}

impl FileSystem for DevFS {
    fn sync(&self) -> Result<()> {
        Ok(())
    }

    fn root(&self) -> Arc<dyn INode> {
        Arc::new(DevFSRootINode {
            fs: self.self_ref.upgrade().unwrap(),
        })
    }

    fn metadata(&self) -> FileSystemMetadata {
        FileSystemMetadata {
            files: 0,
            files_free: 0,
            max_name_len: 0
        }
    }
}

impl DevFS {
    /// Creates a new instance of `DevFS`
    pub fn new() -> Arc<DevFS> {
        DevFS {
            devices: RwLock::new(BTreeMap::new()),
            self_ref: Weak::default(),
        }.wrap()
    }

    /// Add a new device with name `name` (probably mounted at /dev/name)
    pub fn add(&self, name: &str, device: Arc<dyn INode>) -> Result<()> {
        let mut devices = self.devices.write();
        if devices.contains_key(name) {
            return Err(FsError::EntryExists);
        }

        devices.insert(String::from(name), device);
        Ok(())
    }

    /// Remove a device with name `name`
    pub fn remove(&self, name: &str) -> Result<()> {
        let mut devices = self.devices.write();
        devices.remove(name).ok_or(FsError::EntryNotFound)?;
        Ok(())
    }

    /// Wraps the `DevFS` in an `Arc` and sets the `self_ref` variable
    fn wrap(self) -> Arc<DevFS> {
        let fs = Arc::new(self);
        let weak = Arc::downgrade(&fs);
        let ptr = Arc::into_raw(fs) as *mut DevFS;

        unsafe {
            (*ptr).self_ref = weak;
            Arc::from_raw(ptr)
        }
    }
}

/// The root inode for `DevFS`
struct DevFSRootINode {
    fs: Arc<DevFS>,
}

impl INode for DevFSRootINode {
    fn read_at(&self, _offset: usize, _buf: &mut [u8]) -> Result<usize> {
        Err(FsError::IsDirectory)
    }

    fn write_at(&self, _offset: usize, _buf: &[u8]) -> Result<usize> {
        Err(FsError::IsDirectory)
    }

    fn metadata(&self) -> Result<INodeMetadata> {
        Ok(INodeMetadata {
            inode: 1,
            size: self.fs.devices.read().len(),
            access_time: Timespec { sec: 0, nanosec: 0 },
            modification_time: Timespec { sec: 0, nanosec: 0 },
            change_time: Timespec { sec: 0, nanosec: 0 },
            type_: FileType::Directory,
            permissions: 0o666,
            links: 1,
            uid: 0,
            gid: 0,
        })
    }

    fn set_metadata(&self, _metadata: INodeMetadata) -> Result<()> {
        Err(FsError::Unsupported)
    }

    fn sync_all(&self) -> Result<()> {
        Ok(())
    }

    fn sync_data(&self) -> Result<()> {
        Ok(())
    }

    fn resize(&self, _new_len: usize) -> Result<()> {
        Err(FsError::IsDirectory)
    }

    fn create(&self, _name: &str, _type_: FileType, _permissions: u32) -> Result<Arc<dyn INode>> {
        Err(FsError::Unsupported)
    }

    fn link(&self, _name: &str, _other: &Arc<dyn INode>) -> Result<()> {
        Err(FsError::Unsupported)
    }

    fn unlink(&self, _name: &str) -> Result<()> {
        Err(FsError::Unsupported)
    }

    fn move_(&self, _old_name: &str, _target: &Arc<dyn INode>, _new_name: &str) -> Result<()> {
        Err(FsError::Unsupported)
    }

    fn find(&self, name: &str) -> Result<Arc<dyn INode>> {
        match name {
            "." | ".." => Ok(self.fs.root()),
            name => self.fs.devices.read().get(name).cloned().ok_or(FsError::EntryNotFound)
        }
    }

    fn get_entry(&self, index: usize) -> Result<String> {
        match index {
            0 => Ok(String::from(".")),
            1 => Ok(String::from("..")),
            i => self.fs.devices.read().keys().nth(i - 2).cloned().ok_or(FsError::EntryNotFound)
        }
    }

    fn filesystem(&self) -> Arc<dyn FileSystem> {
        self.fs.clone()
    }

    fn as_any_ref(&self) -> &dyn Any {
        self
    }
}