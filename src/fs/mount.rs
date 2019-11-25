use alloc::collections::BTreeMap;
use alloc::sync::{Arc, Weak};

use spin::RwLock;

use fs::vfs::{FileSystem, FileType, FsError, INode, Result, FileSystemMetadata, INodeMetadata};
use alloc::string::String;
use core::any::Any;

/// A wrapper for another filesystem that allows you to mount another file system to any inode.
pub struct MountFS {
    inner: Arc<dyn FileSystem>,
    // TODO: Maybe some filesystems use multiple Arc's for a single inode
    mountpoints: RwLock<BTreeMap<usize, Arc<MountFS>>>,
    self_mountpoint: Option<Arc<MountedNode>>,
    self_ref: Weak<MountFS>,
}

impl MountFS {
    /// Creates a new instance of `MountFS` with root filesystem `fs`
    pub fn new(fs: Arc<dyn FileSystem>) -> Arc<MountFS> {
        MountFS {
            inner: fs,
            mountpoints: RwLock::new(BTreeMap::new()),
            self_mountpoint: None,
            self_ref: Weak::default(),
        }.wrap()
    }

    /// Get the root inode of this mount
    pub fn root(&self) -> Arc<MountedNode> {
        MountedNode {
            inode: self.inner.root(),
            fs: self.self_ref.upgrade().unwrap(),
            self_ref: Weak::default(),
        }.wrap()
    }

    /// Wraps the `MountFS` in an `Arc` and sets the `self_ref` variable
    fn wrap(self) -> Arc<MountFS> {
        let fs = Arc::new(self);
        let weak = Arc::downgrade(&fs);
        let ptr = Arc::into_raw(fs) as *mut MountFS;

        unsafe {
            (*ptr).self_ref = weak;
            Arc::from_raw(ptr)
        }
    }
}

impl FileSystem for MountFS {
    fn sync(&self) -> Result<()> {
        self.inner.sync()?;

        for mount_fs in self.mountpoints.read().values() {
            mount_fs.sync()?;
        }

        Ok(())
    }

    fn root(&self) -> Arc<dyn INode> {
        self.root()
    }

    fn metadata(&self) -> FileSystemMetadata {
        self.inner.metadata()
    }
}

/// An inode implementation for `MountFS` that forwards most implementations to the inner filesystem
pub struct MountedNode {
    pub inode: Arc<dyn INode>,
    pub fs: Arc<MountFS>,
    self_ref: Weak<MountedNode>,
}

impl MountedNode {
    /// Mount the filesystem `fs` if this inode is a directory.
    pub fn mount(&self, fs: Arc<dyn FileSystem>) -> Result<Arc<MountFS>> {
        if self.inode.metadata()?.type_ != FileType::Directory {
            return Err(FsError::NotDirectory);
        }

        let mounted_fs = MountFS {
            inner: fs,
            mountpoints: RwLock::new(BTreeMap::new()),
            self_mountpoint: Some(self.self_ref.upgrade().unwrap()),
            self_ref: Weak::default(),
        }.wrap();

        self.fs.mountpoints.write()
            .insert(self.inode.metadata()?.inode, mounted_fs.clone());

        Ok(mounted_fs)
    }

    /// If a filesystem is mounted here, it returns the root inode of that filesystem. Else it
    /// returns self
    fn overlaid_inode(&self) -> Arc<MountedNode> {
        let inode = self.inode.metadata().unwrap().inode;

        if let Some(sub_fs) = self.fs.mountpoints.read().get(&inode) {
            sub_fs.root()
        } else {
            self.self_ref.upgrade().unwrap()
        }
    }

    /// Check if this inode is the root of the `MountFS`
    fn is_root(&self) -> bool {
        self.inode.filesystem().root().metadata().unwrap().inode ==
            self.inode.metadata().unwrap().inode
    }

    /// Create a new inode which returns `MountedNode`
    pub fn create(&self, name: &str, type_: FileType, permissions: u32) -> Result<Arc<MountedNode>> {
        Ok(MountedNode {
            inode: self.inode.create(name, type_, permissions)?,
            fs: self.fs.clone(),
            self_ref: Weak::default(),
        }.wrap())
    }

    /// Find a node if this inode is a directory and return a `MountedNode`
    pub fn find(&self, name: &str) -> Result<Arc<MountedNode>> {
        if self.inode.metadata()?.type_ != FileType::Directory {
            return Err(FsError::NotDirectory);
        }

        match name {
            "." => Ok(self.self_ref.upgrade().ok_or(FsError::EntryNotFound)?),
            ".." => {
                if self.is_root() {
                    match &self.fs.self_mountpoint {
                        Some(inode) => inode.find(".."),
                        None => Ok(self.self_ref.upgrade().ok_or(FsError::EntryNotFound)?),
                    }
                } else {
                    Ok(MountedNode {
                        inode: self.inode.find(name)?,
                        fs: self.fs.clone(),
                        self_ref: Weak::default(),
                    }.wrap())
                }
            },
            _ => {
                Ok(MountedNode {
                    inode: self.overlaid_inode().inode.find(name)?,
                    fs: self.fs.clone(),
                    self_ref: Weak::default(),
                }.wrap().overlaid_inode())
            }
        }
    }

    /// Wraps the `MountedNode` in an `Arc` and sets the `self_ref` variable
    fn wrap(self) -> Arc<MountedNode> {
        let inode = Arc::new(self);
        let weak = Arc::downgrade(&inode);
        let ptr = Arc::into_raw(inode) as *mut MountedNode;

        unsafe {
            (*ptr).self_ref = weak;
            Arc::from_raw(ptr)
        }
    }
}

impl INode for MountedNode {
    fn read_at(&self, offset: usize, buf: &mut [u8]) -> Result<usize> {
        self.inode.read_at(offset, buf)
    }

    fn write_at(&self, offset: usize, buf: &[u8]) -> Result<usize> {
        self.inode.write_at(offset, buf)
    }

    fn metadata(&self) -> Result<INodeMetadata> {
        self.inode.metadata()
    }

    fn set_metadata(&self, metadata: INodeMetadata) -> Result<()> {
        self.inode.set_metadata(metadata)
    }

    fn sync_all(&self) -> Result<()> {
        self.inode.sync_all()
    }

    fn sync_data(&self) -> Result<()> {
        self.inode.sync_data()
    }

    fn resize(&self, new_len: usize) -> Result<()> {
        self.inode.resize(new_len)
    }

    fn create(&self, name: &str, type_: FileType, permissions: u32) -> Result<Arc<dyn INode>> {
        Ok(self.create(name, type_, permissions)?)
    }

    fn link(&self, name: &str, other: &Arc<dyn INode>) -> Result<()> {
        let other = &other.downcast_ref::<MountedNode>().ok_or(FsError::NotSameFileSystem)?.inode;
        self.inode.link(name, other)
    }

    fn unlink(&self, name: &str) -> Result<()> {
        if self.fs.mountpoints.read().contains_key(&self.inode.metadata()?.inode) {
            return Err(FsError::Busy);
        }

        self.inode.unlink(name)
    }

    fn move_(&self, old_name: &str, target: &Arc<dyn INode>, new_name: &str) -> Result<()> {
        let target = &target.downcast_ref::<MountedNode>().ok_or(FsError::NotSameFileSystem)?.inode;
        self.inode.move_(old_name, target, new_name)
    }

    fn find(&self, name: &str) -> Result<Arc<dyn INode>> {
        Ok(self.find(name)?)
    }

    fn get_entry(&self, index: usize) -> Result<String> {
        self.inode.get_entry(index)
    }

    fn filesystem(&self) -> Arc<dyn FileSystem> {
        self.fs.clone()
    }

    fn as_any_ref(&self) -> &dyn Any {
        self
    }
}