use alloc::vec::Vec;
use axfs_vfs::{impl_vfs_non_dir_default, VfsNodeAttr, VfsNodeOps, VfsResult};
use spin::RwLock;

/// The file node in RAM filesystem.
///
/// This represents a regular file stored in memory.
/// It implements the VFS node operations trait to provide file operations.
///
/// # Fields
///
/// - `content` - The file content stored as a byte vector
pub struct FileNode {
    content: RwLock<Vec<u8>>,
}

impl FileNode {
    /// Creates a new empty file node.
    ///
    /// # Returns
    ///
    /// A new file node with empty content.
    pub(super) const fn new() -> Self {
        Self {
            content: RwLock::new(Vec::new()),
        }
    }
}

impl VfsNodeOps for FileNode {
    /// Returns the attributes of this file.
    ///
    /// # Returns
    ///
    /// Returns file attributes with current size.
    fn get_attr(&self) -> VfsResult<VfsNodeAttr> {
        Ok(VfsNodeAttr::new_file(self.content.read().len() as _, 0))
    }

    /// Truncates or extends the file to the specified size.
    ///
    /// If `size` is smaller than current size, the file is truncated.
    /// If larger, the file is extended with zeros.
    ///
    /// # Arguments
    ///
    /// * `size` - The new size in bytes
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success.
    fn truncate(&self, size: u64) -> VfsResult {
        let mut content = self.content.write();
        if size < content.len() as u64 {
            content.truncate(size as _);
        } else {
            content.resize(size as _, 0);
        }
        Ok(())
    }

    /// Reads data from the file at the given offset.
    ///
    /// # Arguments
    ///
    /// * `offset` - The byte offset to start reading from
    /// * `buf` - The buffer to read data into
    ///
    /// # Returns
    ///
    /// Returns the number of bytes actually read.
    fn read_at(&self, offset: u64, buf: &mut [u8]) -> VfsResult<usize> {
        let content = self.content.read();
        let start = content.len().min(offset as usize);
        let end = content.len().min(offset as usize + buf.len());
        let src = &content[start..end];
        buf[..src.len()].copy_from_slice(src);
        Ok(src.len())
    }

    /// Writes data to the file at the given offset.
    ///
    /// The file is automatically extended if necessary.
    ///
    /// # Arguments
    ///
    /// * `offset` - The byte offset to start writing at
    /// * `buf` - The buffer containing data to write
    ///
    /// # Returns
    ///
    /// Returns the number of bytes written.
    fn write_at(&self, offset: u64, buf: &[u8]) -> VfsResult<usize> {
        let offset = offset as usize;
        let mut content = self.content.write();
        if offset + buf.len() > content.len() {
            content.resize(offset + buf.len(), 0);
        }
        let dst = &mut content[offset..offset + buf.len()];
        dst.copy_from_slice(&buf[..dst.len()]);
        Ok(buf.len())
    }

    impl_vfs_non_dir_default! {}
}
