//! System tests for axfs_vfs
//!
//! This module contains system-level tests that verify the integration
//! of axfs_vfs components in a simulated operating system environment.
//! These tests verify the behavior of VFS in real-world scenarios.

use axerrno::ax_err;
use axfs_vfs::{VfsDirEntry, VfsNodeAttr, VfsNodeOps, VfsNodeRef, VfsNodeType, VfsOps, VfsResult};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// A simulated inode structure for system-level testing
#[derive(Debug, Clone)]
struct SimulatedInode {
    ino: u64,
    ty: VfsNodeType,
    size: u64,
    blocks: u64,
    perm: u16,
}

/// A simulated file for system-level testing
struct SimulatedFile {
    inode: SimulatedInode,
    data: Arc<Mutex<Vec<u8>>>,
}

impl SimulatedFile {
    fn new(ino: u64) -> Self {
        SimulatedFile {
            inode: SimulatedInode {
                ino,
                ty: VfsNodeType::File,
                size: 0,
                blocks: 0,
                perm: 0o644,
            },
            data: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl VfsNodeOps for SimulatedFile {
    fn open(&self) -> VfsResult {
        Ok(())
    }

    fn release(&self) -> VfsResult {
        Ok(())
    }

    fn get_attr(&self) -> VfsResult<VfsNodeAttr> {
        let data = self.data.lock().unwrap();
        let size = data.len() as u64;
        let blocks = (size + 511) / 512; // Round up to 512-byte blocks
        Ok(VfsNodeAttr::new_file(size, blocks))
    }

    fn read_at(&self, offset: u64, buf: &mut [u8]) -> VfsResult<usize> {
        let data = self.data.lock().unwrap();
        let offset = offset as usize;
        let to_read = buf.len().min(data.len().saturating_sub(offset));
        buf[..to_read].copy_from_slice(&data[offset..offset + to_read]);
        Ok(to_read)
    }

    fn write_at(&self, offset: u64, buf: &[u8]) -> VfsResult<usize> {
        let mut data = self.data.lock().unwrap();
        let offset = offset as usize;
        let new_len = (offset + buf.len()).max(data.len());
        data.resize(new_len, 0);
        data[offset..offset + buf.len()].copy_from_slice(buf);
        Ok(buf.len())
    }

    fn fsync(&self) -> VfsResult {
        Ok(())
    }

    fn truncate(&self, size: u64) -> VfsResult {
        let mut data = self.data.lock().unwrap();
        data.resize(size as usize, 0);
        Ok(())
    }

    fn as_any(&self) -> &dyn core::any::Any {
        self
    }
}

/// A simulated directory for system-level testing
struct SimulatedDirectory {
    inode: SimulatedInode,
    entries: Arc<Mutex<HashMap<String, VfsNodeRef>>>,
}

impl SimulatedDirectory {
    fn new(ino: u64, is_root: bool) -> Self {
        SimulatedDirectory {
            inode: SimulatedInode {
                ino,
                ty: VfsNodeType::Dir,
                size: 4096,
                blocks: 8,
                perm: if is_root { 0o755 } else { 0o755 },
            },
            entries: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn add_entry(&self, name: &str, node: VfsNodeRef) {
        self.entries.lock().unwrap().insert(name.to_string(), node);
    }
}

impl VfsNodeOps for SimulatedDirectory {
    fn open(&self) -> VfsResult {
        Ok(())
    }

    fn release(&self) -> VfsResult {
        Ok(())
    }

    fn get_attr(&self) -> VfsResult<VfsNodeAttr> {
        Ok(VfsNodeAttr::new_dir(self.inode.size, self.inode.blocks))
    }

    fn lookup(self: Arc<Self>, path: &str) -> VfsResult<VfsNodeRef> {
        let entries = self.entries.lock().unwrap();
        entries.get(path).cloned().ok_or(axerrno::AxError::NotFound)
    }

    fn create(&self, path: &str, ty: VfsNodeType) -> VfsResult {
        let mut entries = self.entries.lock().unwrap();
        if entries.contains_key(path) {
            return Ok(());
        }

        let node: VfsNodeRef = match ty {
            VfsNodeType::File => Arc::new(SimulatedFile::new(rand::random())),
            VfsNodeType::Dir => Arc::new(SimulatedDirectory::new(rand::random(), false)),
            _ => return ax_err!(Unsupported),
        };

        entries.insert(path.to_string(), node);
        Ok(())
    }

    fn remove(&self, path: &str) -> VfsResult {
        let mut entries = self.entries.lock().unwrap();
        entries.remove(path).ok_or(axerrno::AxError::NotFound)?;
        Ok(())
    }

    fn read_dir(&self, start_idx: usize, dirents: &mut [VfsDirEntry]) -> VfsResult<usize> {
        let entries = self.entries.lock().unwrap();
        let entry_names: Vec<_> = entries.keys().collect();
        let count = dirents
            .len()
            .min(entry_names.len().saturating_sub(start_idx));

        for i in 0..count {
            let name = entry_names[start_idx + i];
            let node = entries.get(name).unwrap();
            let attr = node.get_attr()?;

            dirents[i] = VfsDirEntry::new(name, attr.file_type());
        }

        Ok(count)
    }

    fn rename(&self, src_path: &str, dst_path: &str) -> VfsResult {
        let mut entries = self.entries.lock().unwrap();
        if !entries.contains_key(src_path) {
            return ax_err!(NotFound);
        }

        let node = entries.remove(src_path).unwrap();
        entries.insert(dst_path.to_string(), node);
        Ok(())
    }

    fn as_any(&self) -> &dyn core::any::Any {
        self
    }
}

/// A simulated filesystem for system-level testing
struct SimulatedFilesystem {
    root: VfsNodeRef,
}

impl SimulatedFilesystem {
    fn new() -> Self {
        let root = Arc::new(SimulatedDirectory::new(1, true));
        SimulatedFilesystem { root }
    }

    fn setup(&self) -> VfsResult {
        // Setup initial directory structure
        let root = self.root.clone();
        root.create("bin", VfsNodeType::Dir)?;
        root.create("etc", VfsNodeType::Dir)?;
        root.create("home", VfsNodeType::Dir)?;
        root.create("var", VfsNodeType::Dir)?;

        // Create some files
        root.create("test.txt", VfsNodeType::File)?;
        root.create("README.md", VfsNodeType::File)?;
        Ok(())
    }
}

impl VfsOps for SimulatedFilesystem {
    fn mount(&self, _path: &str, _mount_point: VfsNodeRef) -> VfsResult {
        self.setup()
    }

    fn umount(&self) -> VfsResult {
        Ok(())
    }

    fn statfs(&self) -> VfsResult<axfs_vfs::FileSystemInfo> {
        ax_err!(Unsupported)
    }

    fn root_dir(&self) -> VfsNodeRef {
        Arc::clone(&self.root)
    }
}

// ============== System Tests ==============

#[test]
fn test_system_filesystem_lifecycle() {
    let fs = SimulatedFilesystem::new();

    // Test mount
    let result = fs.mount("/", fs.root_dir());
    assert!(result.is_ok());

    // Test umount
    let result = fs.umount();
    assert!(result.is_ok());
}

#[test]
fn test_system_directory_structure_setup() {
    let fs = SimulatedFilesystem::new();
    fs.mount("/", fs.root_dir()).unwrap();

    let root = fs.root_dir();
    let dirents: &mut [VfsDirEntry] =
        &mut (0..100).map(|_| VfsDirEntry::default()).collect::<Vec<_>>();
    let count = root.read_dir(0, dirents).unwrap();

    // Should have created directories and files
    assert!(count > 0);
}

#[test]
fn test_system_file_create_read_write() {
    let fs = SimulatedFilesystem::new();
    fs.mount("/", fs.root_dir()).unwrap();

    let root = fs.root_dir();
    root.create("system_test.txt", VfsNodeType::File).unwrap();

    // Lookup the file
    let file = root.lookup("system_test.txt").unwrap();

    // Write data
    let test_data = b"System test data";
    let written = file.write_at(0, test_data).unwrap();
    assert_eq!(written, test_data.len());

    // Read data back
    let mut read_buf = vec![0u8; 100];
    let read_bytes = file.read_at(0, &mut read_buf).unwrap();
    assert_eq!(read_bytes, test_data.len());
    assert_eq!(&read_buf[..read_bytes], test_data);
}

#[test]
fn test_system_file_operations_sequence() {
    let fs = SimulatedFilesystem::new();
    fs.mount("/", fs.root_dir()).unwrap();

    let root = fs.root_dir();

    // Create file
    root.create("seq_test.txt", VfsNodeType::File).unwrap();
    let file = root.lookup("seq_test.txt").unwrap();

    // Write first chunk
    file.write_at(0, b"Hello, ").unwrap();

    // Write second chunk at offset
    file.write_at(7, b"World!").unwrap();

    // Read full content
    let mut buf = vec![0u8; 20];
    let n = file.read_at(0, &mut buf).unwrap();
    assert_eq!(&buf[..n], b"Hello, World!");

    // Truncate
    file.truncate(5).unwrap();

    // Read after truncate
    let mut buf2 = vec![0u8; 20];
    let n2 = file.read_at(0, &mut buf2).unwrap();
    assert_eq!(&buf2[..n2], b"Hello");
}

#[test]
fn test_system_directory_operations() {
    let fs = SimulatedFilesystem::new();
    fs.mount("/", fs.root_dir()).unwrap();

    let root = fs.root_dir();

    // Create subdirectory
    root.create("test_dir", VfsNodeType::Dir).unwrap();
    let dir = root.clone().lookup("test_dir").unwrap();

    // Create file in subdirectory
    dir.create("file_in_dir.txt", VfsNodeType::File).unwrap();

    // Create duplicate should not fail
    root.create("test_dir", VfsNodeType::Dir).unwrap();

    // Remove directory
    root.remove("test_dir").unwrap();

    // Lookup should fail
    assert!(root.lookup("test_dir").is_err());
}

#[test]
fn test_system_file_rename() {
    let fs = SimulatedFilesystem::new();
    fs.mount("/", fs.root_dir()).unwrap();

    let root = fs.root_dir();

    // Create file
    root.create("old_name.txt", VfsNodeType::File).unwrap();
    let file = root.clone().lookup("old_name.txt").unwrap();

    // Write data
    file.write_at(0, b"Rename test").unwrap();

    // Rename file
    root.rename("old_name.txt", "new_name.txt").unwrap();

    // Old name should not exist
    assert!(root.clone().lookup("old_name.txt").is_err());

    // New name should exist
    let new_file = root.lookup("new_name.txt").unwrap();

    // Data should be preserved
    let mut buf = vec![0u8; 20];
    let n = new_file.read_at(0, &mut buf).unwrap();
    assert_eq!(&buf[..n], b"Rename test");
}

#[test]
fn test_system_directory_pagination() {
    let fs = SimulatedFilesystem::new();
    fs.mount("/", fs.root_dir()).unwrap();

    let root = fs.root_dir();

    // Create multiple files
    for i in 0..15 {
        root.create(&format!("file_{}.txt", i), VfsNodeType::File)
            .unwrap();
    }

    // Read first page
    let mut dirents1: Vec<VfsDirEntry> = (0..5).map(|_| VfsDirEntry::default()).collect();
    let count1 = root.read_dir(0, &mut dirents1).unwrap();
    assert!(count1 > 0);

    // Read second page
    let mut dirents2: Vec<VfsDirEntry> = (0..5).map(|_| VfsDirEntry::default()).collect();
    let count2 = root.read_dir(5, &mut dirents2).unwrap();
    assert!(count2 > 0);

    // Read third page
    let mut dirents3: Vec<VfsDirEntry> = (0..5).map(|_| VfsDirEntry::default()).collect();
    let count3 = root.read_dir(10, &mut dirents3).unwrap();
    assert!(count3 > 0);
}

#[test]
fn test_system_file_truncate_extend() {
    let fs = SimulatedFilesystem::new();
    fs.mount("/", fs.root_dir()).unwrap();

    let root = fs.root_dir();
    root.create("truncate_test.txt", VfsNodeType::File).unwrap();
    let file = root.lookup("truncate_test.txt").unwrap();

    // Write initial data
    file.write_at(0, b"12345").unwrap();

    // Extend file
    file.truncate(100).unwrap();

    let mut buf = vec![0u8; 200];
    let n = file.read_at(0, &mut buf).unwrap();
    assert_eq!(n, 100);
    assert_eq!(&buf[..5], b"12345");
    assert!(&buf[5..n].iter().all(|&b| b == 0));

    // Shrink file
    file.truncate(3).unwrap();

    let mut buf2 = vec![0u8; 20];
    let n2 = file.read_at(0, &mut buf2).unwrap();
    assert_eq!(n2, 3);
    assert_eq!(&buf2[..n2], b"123");
}

#[test]
fn test_system_file_fsync() {
    let fs = SimulatedFilesystem::new();
    fs.mount("/", fs.root_dir()).unwrap();

    let root = fs.root_dir();
    root.create("fsync_test.txt", VfsNodeType::File).unwrap();
    let file = root.lookup("fsync_test.txt").unwrap();

    // Write data
    file.write_at(0, b"Fsync test").unwrap();

    // Sync should succeed
    let result = file.fsync();
    assert!(result.is_ok());
}

#[test]
fn test_system_node_attributes() {
    let fs = SimulatedFilesystem::new();
    fs.mount("/", fs.root_dir()).unwrap();

    let root = fs.root_dir();

    // Test directory attributes
    let dir_attr = root.get_attr().unwrap();
    assert!(dir_attr.is_dir());
    assert!(!dir_attr.is_file());
    assert_eq!(dir_attr.size(), 4096);

    // Create file
    root.create("attr_test.txt", VfsNodeType::File).unwrap();
    let file = root.lookup("attr_test.txt").unwrap();

    // Test file attributes
    let file_attr = file.get_attr().unwrap();
    assert!(!file_attr.is_dir());
    assert!(file_attr.is_file());
    assert_eq!(file_attr.size(), 0);

    // Write data and check updated size
    file.write_at(0, b"Test").unwrap();
    let file_attr2 = file.get_attr().unwrap();
    assert_eq!(file_attr2.size(), 4);
}

#[test]
fn test_system_error_handling() {
    let fs = SimulatedFilesystem::new();
    fs.mount("/", fs.root_dir()).unwrap();

    let root = fs.root_dir();

    // Lookup non-existent file
    let result = root.clone().lookup("nonexistent.txt");
    assert!(result.is_err());

    // Remove non-existent file
    let result = root.remove("nonexistent.txt");
    assert!(result.is_err());

    // Rename non-existent file
    let result = root.rename("nonexistent.txt", "new.txt");
    assert!(result.is_err());
}
