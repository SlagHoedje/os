use alloc::sync::Arc;

use fs::dev::DevFS;
use fs::vfs::{INode, INodeMetadata, FileType, FileSystem, FsError, Result, Timespec};
use alloc::string::String;
use core::any::Any;

pub struct ZeroNullDevice {
    null: bool,
    fs: Arc<DevFS>,
}

impl ZeroNullDevice {
    pub fn new(fs: Arc<DevFS>, null: bool) -> Arc<ZeroNullDevice> {
        Arc::new(ZeroNullDevice { null, fs })
    }
}

impl INode for ZeroNullDevice {
    fn read_at(&self, _offset: usize, buf: &mut [u8]) -> Result<usize> {
        if self.null {
            Ok(0)
        } else {
            for x in buf.iter_mut() {
                *x = 0;
            }

            Ok(buf.len())
        }
    }

    fn write_at(&self, _offset: usize, buf: &[u8]) -> Result<usize> {
        Ok(buf.len())
    }

    fn metadata(&self) -> Result<INodeMetadata> {
        Ok(INodeMetadata {
            inode: 0,
            size: 0,
            access_time: Timespec { sec: 0, nanosec: 0 },
            modification_time: Timespec { sec: 0, nanosec: 0 },
            change_time: Timespec { sec: 0, nanosec: 0 },
            type_: FileType::CharDevice,
            permissions: 0o666,
            links: 1,
            uid: 0,
            gid: 0,
            // TODO: rdev?
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
        Err(FsError::Unsupported)
    }

    fn create(&self, _name: &str, _type_: FileType, _permissions: u32) -> Result<Arc<dyn INode>> {
        Err(FsError::NotDirectory)
    }

    fn link(&self, _name: &str, _other: &Arc<dyn INode>) -> Result<()> {
        Err(FsError::NotDirectory)
    }

    fn unlink(&self, _name: &str) -> Result<()> {
        Err(FsError::NotDirectory)
    }

    fn move_(&self, _old_name: &str, _target: &Arc<dyn INode>, _new_name: &str) -> Result<()> {
        Err(FsError::NotDirectory)
    }

    fn find(&self, _name: &str) -> Result<Arc<dyn INode>> {
        Err(FsError::NotDirectory)
    }

    fn get_entry(&self, _index: usize) -> Result<String> {
        Err(FsError::NotDirectory)
    }

    fn filesystem(&self) -> Arc<dyn FileSystem> {
        self.fs.clone()
    }

    fn as_any_ref(&self) -> &dyn Any {
        self
    }
}