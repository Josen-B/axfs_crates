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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_node_new() {
        let file = FileNode::new();
        let attr = file.get_attr().unwrap();
        assert!(attr.is_file());
        assert_eq!(attr.size(), 0);
    }

    #[test]
    fn test_file_node_write_at() {
        let file = FileNode::new();
        let data = b"Hello, World!";
        let written = file.write_at(0, data).unwrap();
        assert_eq!(written, data.len());
        assert_eq!(file.get_attr().unwrap().size(), data.len() as u64);
    }

    #[test]
    fn test_file_node_write_at_offset() {
        let file = FileNode::new();
        let data = b"World!";
        file.write_at(0, b"Hello, ").unwrap();
        let written = file.write_at(7, data).unwrap();
        assert_eq!(written, data.len());

        let mut buf = [0; 20];
        let read = file.read_at(0, &mut buf).unwrap();
        assert_eq!(read, 13);
        assert_eq!(&buf[..13], b"Hello, World!");
    }

    #[test]
    fn test_file_node_read_at_empty() {
        let file = FileNode::new();
        let mut buf = [0; 100];
        let read = file.read_at(0, &mut buf).unwrap();
        assert_eq!(read, 0);
    }

    #[test]
    fn test_file_node_read_at() {
        let file = FileNode::new();
        let data = b"Hello, World!";
        file.write_at(0, data).unwrap();

        let mut buf = [0; 100];
        let read = file.read_at(0, &mut buf).unwrap();
        assert_eq!(read, data.len());
        assert_eq!(&buf[..data.len()], data);
    }

    #[test]
    fn test_file_node_read_at_partial() {
        let file = FileNode::new();
        let data = b"Hello, World!";
        file.write_at(0, data).unwrap();

        let mut buf = [0; 5];
        let read = file.read_at(7, &mut buf).unwrap();
        assert_eq!(read, 5);
        assert_eq!(&buf, b"World");
    }

    #[test]
    fn test_file_node_read_at_offset() {
        let file = FileNode::new();
        let data = b"Hello, World!";
        file.write_at(0, data).unwrap();

        let mut buf = [0; 100];
        let read = file.read_at(7, &mut buf).unwrap();
        assert_eq!(read, 6);
        assert_eq!(&buf[..6], b"World!");
    }

    #[test]
    fn test_file_node_truncate_shrink() {
        let file = FileNode::new();
        file.write_at(0, b"Hello, World!").unwrap();
        assert_eq!(file.get_attr().unwrap().size(), 13);

        file.truncate(5).unwrap();
        assert_eq!(file.get_attr().unwrap().size(), 5);

        let mut buf = [0; 100];
        let read = file.read_at(0, &mut buf).unwrap();
        assert_eq!(read, 5);
        assert_eq!(&buf[..5], b"Hello");
    }

    #[test]
    fn test_file_node_truncate_grow() {
        let file = FileNode::new();
        file.write_at(0, b"Hello").unwrap();
        assert_eq!(file.get_attr().unwrap().size(), 5);

        file.truncate(10).unwrap();
        assert_eq!(file.get_attr().unwrap().size(), 10);

        let mut buf = [0; 100];
        let read = file.read_at(0, &mut buf).unwrap();
        assert_eq!(read, 10);
        assert_eq!(&buf[..5], b"Hello");
        assert_eq!(&buf[5..10], [0, 0, 0, 0, 0]);
    }

    #[test]
    fn test_file_node_truncate_zero() {
        let file = FileNode::new();
        file.write_at(0, b"Hello, World!").unwrap();
        assert_eq!(file.get_attr().unwrap().size(), 13);

        file.truncate(0).unwrap();
        assert_eq!(file.get_attr().unwrap().size(), 0);

        let mut buf = [0; 100];
        let read = file.read_at(0, &mut buf).unwrap();
        assert_eq!(read, 0);
    }

    #[test]
    fn test_file_node_write_extends() {
        let file = FileNode::new();
        file.write_at(0, b"Hello").unwrap();
        file.write_at(10, b"World").unwrap();

        assert_eq!(file.get_attr().unwrap().size(), 15);

        let mut buf = [0; 20];
        let read = file.read_at(0, &mut buf).unwrap();
        assert_eq!(read, 15);
        assert_eq!(&buf[..5], b"Hello");
        assert_eq!(&buf[5..10], [0, 0, 0, 0, 0]);
        assert_eq!(&buf[10..15], b"World");
    }

    #[test]
    fn test_file_node_get_attr() {
        let file = FileNode::new();
        let attr = file.get_attr().unwrap();
        assert!(attr.is_file());
        assert!(!attr.is_dir());
        assert_eq!(attr.size(), 0);
        assert_eq!(attr.blocks(), 0);
    }

    #[test]
    fn test_file_node_get_attr_after_write() {
        let file = FileNode::new();
        file.write_at(0, b"Hello").unwrap();
        let attr = file.get_attr().unwrap();
        assert_eq!(attr.size(), 5);
    }

    #[test]
    fn test_file_node_operations_combined() {
        let file = FileNode::new();

        // Write data
        file.write_at(0, b"Hello, ").unwrap();
        file.write_at(7, b"World!").unwrap();

        // Read back
        let mut buf = [0; 20];
        file.read_at(0, &mut buf).unwrap();
        assert_eq!(&buf[..13], b"Hello, World!");

        // Truncate and read
        file.truncate(5).unwrap();
        let mut buf2 = [0; 20];
        file.read_at(0, &mut buf2).unwrap();
        assert_eq!(&buf2[..5], b"Hello");
    }
}
