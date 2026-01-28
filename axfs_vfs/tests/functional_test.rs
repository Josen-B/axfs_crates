//! Functional tests for axfs_vfs
//!
//! This module contains functional tests that verify the core functionality
//! of the axfs_vfs crate using mock implementations.

use axfs_vfs::{
    VfsDirEntry, VfsNodeAttr, VfsNodeOps, VfsNodePerm, VfsNodeRef, VfsNodeType, VfsOps, VfsResult,
};
use std::sync::Arc;

/// Mock filesystem for testing
struct MockFileSystem {
    root: VfsNodeRef,
}

impl MockFileSystem {
    fn new() -> Self {
        MockFileSystem {
            root: Arc::new(MockDirectory::new()),
        }
    }
}

impl VfsOps for MockFileSystem {
    fn root_dir(&self) -> VfsNodeRef {
        Arc::clone(&self.root)
    }
}

/// Mock directory node for testing
struct MockDirectory;

impl MockDirectory {
    fn new() -> Self {
        MockDirectory
    }
}

impl VfsNodeOps for MockDirectory {
    fn open(&self) -> VfsResult {
        Ok(())
    }

    fn release(&self) -> VfsResult {
        Ok(())
    }

    fn get_attr(&self) -> VfsResult<VfsNodeAttr> {
        Ok(VfsNodeAttr::new_dir(4096, 8))
    }

    fn lookup(self: Arc<Self>, _path: &str) -> VfsResult<VfsNodeRef> {
        Ok(Arc::new(MockFile::new()))
    }

    fn create(&self, _path: &str, _ty: VfsNodeType) -> VfsResult {
        Ok(())
    }

    fn remove(&self, _path: &str) -> VfsResult {
        Ok(())
    }

    fn read_dir(&self, _start_idx: usize, _dirents: &mut [VfsDirEntry]) -> VfsResult<usize> {
        Ok(0)
    }

    fn as_any(&self) -> &dyn core::any::Any {
        self
    }
}

/// Mock file node for testing
struct MockFile;

impl MockFile {
    fn new() -> Self {
        MockFile
    }
}

impl VfsNodeOps for MockFile {
    fn open(&self) -> VfsResult {
        Ok(())
    }

    fn release(&self) -> VfsResult {
        Ok(())
    }

    fn get_attr(&self) -> VfsResult<VfsNodeAttr> {
        Ok(VfsNodeAttr::new_file(1024, 2))
    }

    fn read_at(&self, _offset: u64, buf: &mut [u8]) -> VfsResult<usize> {
        buf.fill(0);
        Ok(buf.len())
    }

    fn write_at(&self, _offset: u64, buf: &[u8]) -> VfsResult<usize> {
        Ok(buf.len())
    }

    fn fsync(&self) -> VfsResult {
        Ok(())
    }

    fn truncate(&self, _size: u64) -> VfsResult {
        Ok(())
    }

    fn as_any(&self) -> &dyn core::any::Any {
        self
    }
}

// ============== Functional Tests ==============

#[test]
fn test_vfs_ops_root_dir() {
    let fs = MockFileSystem::new();
    let root = fs.root_dir();
    assert!(root.open().is_ok());
}

#[test]
fn test_vfs_node_ops_directory_lifecycle() {
    let dir = Arc::new(MockDirectory::new());

    // Test open and release
    assert!(dir.open().is_ok());
    assert!(dir.release().is_ok());

    // Test get_attr
    let attr = dir.get_attr();
    assert!(attr.is_ok());
    let attr = attr.unwrap();
    assert_eq!(attr.size(), 4096);
    assert_eq!(attr.blocks(), 8);
}

#[test]
fn test_vfs_node_ops_directory_lookup() {
    let dir = Arc::new(MockDirectory::new());

    // Test lookup
    let result = dir.lookup("test.txt");
    assert!(result.is_ok());
    let node = result.unwrap();
    assert!(node.open().is_ok());
}

#[test]
fn test_vfs_node_ops_directory_operations() {
    let dir = Arc::new(MockDirectory::new());

    // Test create
    assert!(dir.create("new_file.txt", VfsNodeType::File).is_ok());
    assert!(dir.create("new_dir", VfsNodeType::Dir).is_ok());

    // Test remove
    assert!(dir.remove("old_file.txt").is_ok());

    // Test read_dir
    let mut dirents: [VfsDirEntry; 10] = core::array::from_fn(|_| VfsDirEntry::default());
    let count = dir.read_dir(0, &mut dirents);
    assert!(count.is_ok());
}

#[test]
fn test_vfs_node_ops_file_lifecycle() {
    let file = Arc::new(MockFile::new());

    // Test open and release
    assert!(file.open().is_ok());
    assert!(file.release().is_ok());

    // Test get_attr
    let attr = file.get_attr();
    assert!(attr.is_ok());
    let attr = attr.unwrap();
    assert_eq!(attr.size(), 1024);
    assert_eq!(attr.blocks(), 2);
}

#[test]
fn test_vfs_node_ops_file_read_write() {
    let file = Arc::new(MockFile::new());

    // Test write
    let write_data = b"Hello, World!";
    let write_result = file.write_at(0, write_data);
    assert!(write_result.is_ok());
    assert_eq!(write_result.unwrap(), write_data.len());

    // Test read
    let mut read_buf = [0u8; 64];
    let read_result = file.read_at(0, &mut read_buf);
    assert!(read_result.is_ok());
    let bytes_read = read_result.unwrap();
    assert_eq!(bytes_read, read_buf.len());
}

#[test]
fn test_vfs_node_ops_file_operations() {
    let file = Arc::new(MockFile::new());

    // Test fsync
    assert!(file.fsync().is_ok());

    // Test truncate
    assert!(file.truncate(2048).is_ok());
    assert!(file.truncate(512).is_ok());
}

#[test]
fn test_vfs_node_ops_as_any_downcast() {
    let dir: VfsNodeRef = Arc::new(MockDirectory::new());
    let file: VfsNodeRef = Arc::new(MockFile::new());

    // Test downcast directory
    let dir_any = dir.as_any();
    assert!(dir_any.downcast_ref::<MockDirectory>().is_some());

    // Test downcast file
    let file_any = file.as_any();
    assert!(file_any.downcast_ref::<MockFile>().is_some());

    // Test failed downcast
    assert!(dir_any.downcast_ref::<MockFile>().is_none());
    assert!(file_any.downcast_ref::<MockDirectory>().is_none());
}

// ============== Error Handling Tests ==============

#[test]
fn test_vfs_node_ops_file_read_write_empty() {
    let file = Arc::new(MockFile::new());

    // Test read with empty buffer
    let mut buf = [0u8; 0];
    let result = file.read_at(0, &mut buf);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 0);

    // Test write with empty buffer
    let result = file.write_at(0, &[]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 0);
}

#[test]
fn test_vfs_node_ops_file_operations_at_offset() {
    let file = Arc::new(MockFile::new());

    // Test write at offset
    let write_data = b"offset data";
    let result = file.write_at(100, write_data);
    assert!(result.is_ok());

    // Test read at offset
    let mut buf = [0u8; 64];
    let result = file.read_at(100, &mut buf);
    assert!(result.is_ok());
}

#[test]
fn test_vfs_node_ops_directory_read_dir_pagination() {
    let dir = Arc::new(MockDirectory::new());

    // Test read_dir with different start indices
    let mut dirents: [VfsDirEntry; 10] = core::array::from_fn(|_| VfsDirEntry::default());

    for start_idx in [0, 5, 10] {
        let count = dir.read_dir(start_idx, &mut dirents);
        assert!(count.is_ok());
    }
}

#[test]
fn test_vfs_node_ops_node_types() {
    let dir = Arc::new(MockDirectory::new());
    let file = Arc::new(MockFile::new());

    // Test directory attributes
    let dir_attr = dir.get_attr().unwrap();
    // Default is directory type
    assert_eq!(dir_attr.perm().bits(), VfsNodePerm::default_dir().bits());

    // Test file attributes
    let file_attr = file.get_attr().unwrap();
    // Default is directory type, but file should have file type
    assert_eq!(file_attr.size(), 1024);
}
