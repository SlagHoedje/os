use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::sync::{Arc, Weak};
use alloc::vec::Vec;
use core::any::Any;
use core::sync::atomic::{AtomicUsize, Ordering};

use spin::RwLock;

use fs::vfs::{FileSystem, FileSystemMetadata, FileType, FsError, INode, INodeMetadata, Result, Timespec};

/// A basic filesystem implementation that is stored in RAM.
pub struct Ramdisk {
    root: Arc<LockedRamdiskINode>,
}

impl Ramdisk {
    pub fn new() -> Arc<Ramdisk> {
        let root = Arc::new(LockedRamdiskINode::new(RamdiskINode {
            parent_ref: Weak::default(),
            self_ref: Weak::default(),
            children: BTreeMap::new(),
            metadata: INodeMetadata {
                inode: next_inode(),
                size: 0,
                access_time: Timespec { sec: 0, nanosec: 0 },
                modification_time: Timespec { sec: 0, nanosec: 0 },
                change_time: Timespec { sec: 0, nanosec: 0 },
                type_: FileType::Directory,
                permissions: 0o777,
                links: 1,
                uid: 0,
                gid: 0,
            },
            content: Vec::new(),
            filesystem: Weak::new(),
        }));

        let filesystem = Arc::new(Ramdisk { root });
        let mut root = filesystem.root.write();
        root.parent_ref = Arc::downgrade(&filesystem.root);
        root.self_ref = Arc::downgrade(&filesystem.root);
        root.filesystem = Arc::downgrade(&filesystem);
        core::mem::drop(root);

        filesystem
    }
}

impl FileSystem for Ramdisk {
    fn sync(&self) -> Result<()> {
        Ok(())
    }

    fn root(&self) -> Arc<dyn INode> {
        self.root.clone()
    }

    fn metadata(&self) -> FileSystemMetadata {
        FileSystemMetadata {
            // TODO: Inaccurate file counting
            files: self.root.read().children.len(),
            files_free: 0,
            max_name_len: 0,
        }
    }
}

/// A locked version of `RamdiskINode` so it can be written to without mutability.
pub type LockedRamdiskINode = RwLock<RamdiskINode>;

/// An inode implementation for `Ramdisk`
pub struct RamdiskINode {
    parent_ref: Weak<LockedRamdiskINode>,
    self_ref: Weak<LockedRamdiskINode>,
    children: BTreeMap<String, Arc<LockedRamdiskINode>>,
    metadata: INodeMetadata,
    content: Vec<u8>,
    filesystem: Weak<Ramdisk>,
}

impl INode for LockedRamdiskINode {
    fn read_at(&self, offset: usize, buf: &mut [u8]) -> Result<usize> {
        let file = self.read();

        if file.metadata.type_ == FileType::Directory {
            return Err(FsError::IsDirectory)
        }

        let start = file.content.len().min(offset);
        let end = file.content.len().min(offset + buf.len());

        let src = &file.content[start..end];

        buf[0..src.len()].copy_from_slice(src);

        Ok(src.len())
    }

    fn write_at(&self, offset: usize, buf: &[u8]) -> Result<usize> {
        let mut file = self.write();

        if file.metadata.type_ == FileType::Directory {
            return Err(FsError::IsDirectory);
        }

        let content = &mut file.content;
        if offset + buf.len() > content.len() {
            content.resize(offset + buf.len(), 0);
        }

        content[offset..offset + buf.len()].copy_from_slice(buf);

        Ok(buf.len())
    }

    fn metadata(&self) -> Result<INodeMetadata> {
        let file = self.read();
        let mut metadata = file.metadata;
        metadata.size = file.content.len();
        Ok(metadata)
    }

    fn set_metadata(&self, metadata: INodeMetadata) -> Result<()> {
        let mut file = self.write();

        file.metadata.access_time = metadata.access_time;
        file.metadata.modification_time = metadata.modification_time;
        file.metadata.change_time = metadata.change_time;
        file.metadata.permissions = metadata.permissions;
        file.metadata.uid = metadata.uid;
        file.metadata.gid = metadata.gid;

        Ok(())
    }

    fn sync_all(&self) -> Result<()> {
        Ok(())
    }

    fn sync_data(&self) -> Result<()> {
        Ok(())
    }

    fn resize(&self, new_len: usize) -> Result<()> {
        let mut file = self.write();

        if file.metadata.type_ != FileType::File {
            return Err(FsError::NotFile);
        }

        file.content.resize(new_len, 0);

        Ok(())
    }

    fn create(&self, name: &str, type_: FileType, permissions: u32) -> Result<Arc<dyn INode>> {
        let mut file = self.write();

        if file.metadata.type_ != FileType::Directory {
            return Err(FsError::NotDirectory);
        }

        let new_file = Arc::new(LockedRamdiskINode::new(RamdiskINode {
            parent_ref: file.self_ref.clone(),
            self_ref: Weak::default(),
            children: BTreeMap::new(),
            metadata: INodeMetadata {
                inode: next_inode(),
                size: 0,
                access_time: Timespec { sec: 0, nanosec: 0 },
                modification_time: Timespec { sec: 0, nanosec: 0 },
                change_time: Timespec { sec: 0, nanosec: 0 },
                type_,
                permissions: permissions as u16,
                links: 1,
                uid: 0,
                gid: 0,
            },
            content: Vec::new(),
            filesystem: file.filesystem.clone(),
        }));

        new_file.write().self_ref = Arc::downgrade(&new_file);
        file.children.insert(String::from(name), new_file.clone());

        Ok(new_file)
    }

    fn link(&self, name: &str, other: &Arc<dyn INode>) -> Result<()> {
        let mut file = self.write();
        let mut other = other.downcast_ref::<LockedRamdiskINode>()
            .ok_or(FsError::NotSameFileSystem)?.write();

        if file.metadata.type_ != FileType::Directory {
            return Err(FsError::NotDirectory);
        }

        if other.metadata.type_ == FileType::Directory {
            return Err(FsError::IsDirectory);
        }

        if file.children.contains_key(name) {
            return Err(FsError::EntryExists);
        }

        file.children.insert(String::from(name), other.self_ref.upgrade().unwrap());
        other.metadata.links += 1;

        Ok(())
    }

    fn unlink(&self, name: &str) -> Result<()> {
        let mut file = self.write();

        if file.metadata.type_ != FileType::Directory {
            return Err(FsError::NotDirectory);
        }

        if name == "." || name == ".." {
            return Err(FsError::DirectoryNotEmpty);
        }

        let other = file.children.get(name)
            .ok_or(FsError::EntryNotFound)?;

        if !other.read().children.is_empty() {
            return Err(FsError::DirectoryNotEmpty);
        }

        other.write().metadata.links -= 1;
        file.children.remove(name);

        Ok(())
    }

    fn move_(&self, old_name: &str, target: &Arc<dyn INode>, new_name: &str) -> Result<()> {
        let file = self.find(old_name)?;
        target.link(new_name, &file)?;

        // If unlinking the original file goes wrong, we need to revert changes by unlinking the new
        // location.
        if let Err(err) = self.unlink(old_name) {
            target.unlink(new_name)?;

            Err(err)
        } else {
            Ok(())
        }
    }

    fn find(&self, name: &str) -> Result<Arc<dyn INode>> {
        let file = self.read();

        if file.metadata.type_ != FileType::Directory {
            return Err(FsError::NotDirectory);
        }

        match name {
            "." => Ok(file.self_ref.upgrade().ok_or(FsError::EntryNotFound)?),
            ".." => Ok(file.parent_ref.upgrade().ok_or(FsError::EntryNotFound)?),
            _ => Ok(file.children.get(name).ok_or(FsError::EntryNotFound)?.clone()),
        }
    }

    fn get_entry(&self, index: usize) -> Result<String> {
        let file = self.read();

        if file.metadata.type_ != FileType::Directory {
            return Err(FsError::NotDirectory);
        }

        match index {
            0 => Ok(String::from(".")),
            1 => Ok(String::from("..")),
            index => {
                if let Some(s) = file.children.keys().nth(index - 2) {
                    Ok(s.clone())
                } else {
                    Err(FsError::EntryNotFound)
                }
            }
        }
    }

    fn filesystem(&self) -> Arc<dyn FileSystem> {
        self.read().filesystem.upgrade().unwrap()
    }

    fn as_any_ref(&self) -> &dyn Any {
        self
    }
}

fn next_inode() -> usize {
    static NEXT_INODE: AtomicUsize = AtomicUsize::new(1);
    NEXT_INODE.fetch_add(1, Ordering::SeqCst)
}