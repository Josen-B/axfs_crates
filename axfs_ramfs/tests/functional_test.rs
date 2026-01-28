//! Functional tests for axfs_ramfs
//!
//! This module contains functional tests that verify the core functionality
//! of the axfs_ramfs crate using the actual implementation.

use axfs_ramfs::{DirNode, RamFileSystem};
use axfs_vfs::{VfsDirEntry, VfsNodeType, VfsOps};

// ============== Filesystem Operations Tests ==============

#[test]
fn test_ramfs_new() {
    let fs = RamFileSystem::new();
    let root = fs.root_dir();
    assert!(root.open().is_ok());
}

#[test]
fn test_ramfs_default() {
    let fs = RamFileSystem::default();
    let root = fs.root_dir();
    assert!(root.open().is_ok());
}

#[test]
fn test_ramfs_root_dir() {
    let fs = RamFileSystem::new();
    let root = fs.root_dir();

    let attr = root.get_attr().unwrap();
    assert!(attr.is_dir());
    assert!(!attr.is_file());
    assert_eq!(attr.size(), 4096);
}

#[test]
fn test_ramfs_root_dir_node() {
    let fs = RamFileSystem::new();
    let root = fs.root_dir_node();
    let entries = root.get_entries();
    assert!(entries.is_empty());
}

// ============== Directory Operations Tests ==============

#[test]
fn test_directory_create() {
    let fs = RamFileSystem::new();
    let root = fs.root_dir();
    root.create("test_dir", VfsNodeType::Dir).unwrap();

    let dir = root.lookup("test_dir").unwrap();
    let attr = dir.get_attr().unwrap();
    assert!(attr.is_dir());
}

#[test]
fn test_directory_create_existing() {
    let fs = RamFileSystem::new();
    let root = fs.root_dir();
    root.create("test_dir", VfsNodeType::Dir).unwrap();

    let result = root.create("test_dir", VfsNodeType::Dir);
    assert!(result.is_err());
}

#[test]
fn test_directory_remove() {
    let fs = RamFileSystem::new();
    let root = fs.root_dir();
    root.create("test_dir", VfsNodeType::Dir).unwrap();

    let result = root.remove("test_dir");
    assert!(result.is_ok());

    let result = root.clone().lookup("test_dir");
    assert!(result.is_err());
}

#[test]
fn test_directory_remove_non_empty() {
    let fs = RamFileSystem::new();
    let root = fs.root_dir();
    root.create("test_dir", VfsNodeType::Dir).unwrap();
    let dir = root.clone().lookup("test_dir").unwrap();
    dir.create("file.txt", VfsNodeType::File).unwrap();

    let result = root.remove("test_dir");
    assert!(result.is_err());
}

#[test]
fn test_directory_lookup() {
    let fs = RamFileSystem::new();
    let root = fs.root_dir();
    root.create("test_dir", VfsNodeType::Dir).unwrap();
    root.create("test_file.txt", VfsNodeType::File).unwrap();

    let dir = root.clone().lookup("test_dir").unwrap();
    let file = root.clone().lookup("test_file.txt").unwrap();

    let dir_attr = dir.get_attr().unwrap();
    let file_attr = file.get_attr().unwrap();
    assert!(dir_attr.is_dir());
    assert!(file_attr.is_file());
}

#[test]
fn test_directory_lookup_nested() {
    let fs = RamFileSystem::new();
    let root = fs.root_dir();
    root.create("parent", VfsNodeType::Dir).unwrap();
    let parent = root.clone().lookup("parent").unwrap();
    parent.create("child", VfsNodeType::Dir).unwrap();
    parent.create("file.txt", VfsNodeType::File).unwrap();

    // Test nested path lookup
    let result = root.clone().lookup("parent/child");
    assert!(result.is_ok());

    let result = root.clone().lookup("parent/file.txt");
    assert!(result.is_ok());
}

#[test]
fn test_directory_lookup_not_found() {
    let fs = RamFileSystem::new();
    let root = fs.root_dir();

    let result = root.lookup("nonexistent");
    assert!(result.is_err());
}

#[test]
fn test_directory_lookup_dot() {
    let fs = RamFileSystem::new();
    let root = fs.root_dir();

    let result = root.lookup(".");
    assert!(result.is_ok());
}

#[test]
fn test_directory_lookup_dotdot() {
    let fs = RamFileSystem::new();
    let root = fs.root_dir();
    root.create("test_dir", VfsNodeType::Dir).unwrap();
    let dir = root.lookup("test_dir").unwrap();

    let result = dir.lookup("..");
    assert!(result.is_ok());
    let attr = result.unwrap().get_attr().unwrap();
    assert!(attr.is_dir());
}

#[test]
fn test_directory_read_dir() {
    let fs = RamFileSystem::new();
    let root = fs.root_dir();
    root.create("dir1", VfsNodeType::Dir).unwrap();
    root.create("dir2", VfsNodeType::Dir).unwrap();
    root.create("file1.txt", VfsNodeType::File).unwrap();

    let mut dirents: [VfsDirEntry; 10] = core::array::from_fn(|_| VfsDirEntry::default());
    let count = root.read_dir(0, &mut dirents).unwrap();

    // Should have . and .. as well as the 3 entries
    assert!(count >= 3);
}

#[test]
fn test_directory_get_entries() {
    let fs = RamFileSystem::new();
    let root = fs.root_dir();
    root.create("dir1", VfsNodeType::Dir).unwrap();
    root.create("file1.txt", VfsNodeType::File).unwrap();

    let root_node = fs.root_dir_node();
    let entries = root_node.get_entries();

    assert_eq!(entries.len(), 2);
    assert!(entries.contains(&"dir1".to_string()));
    assert!(entries.contains(&"file1.txt".to_string()));
}

#[test]
fn test_directory_exist() {
    let fs = RamFileSystem::new();
    let root = fs.root_dir();
    root.create("test.txt", VfsNodeType::File).unwrap();

    let root_node = fs.root_dir_node();
    assert!(root_node.exist("test.txt"));
    assert!(!root_node.exist("nonexistent.txt"));
}

// ============== File Operations Tests ==============

#[test]
fn test_file_create() {
    let fs = RamFileSystem::new();
    let root = fs.root_dir();
    root.create("test.txt", VfsNodeType::File).unwrap();

    let file = root.lookup("test.txt").unwrap();
    let attr = file.get_attr().unwrap();
    assert!(attr.is_file());
    assert_eq!(attr.size(), 0);
}

#[test]
fn test_file_create_existing() {
    let fs = RamFileSystem::new();
    let root = fs.root_dir();
    root.create("test.txt", VfsNodeType::File).unwrap();

    let result = root.create("test.txt", VfsNodeType::File);
    assert!(result.is_err());
}

#[test]
fn test_file_write() {
    let fs = RamFileSystem::new();
    let root = fs.root_dir();
    root.create("test.txt", VfsNodeType::File).unwrap();
    let file = root.lookup("test.txt").unwrap();

    let data = b"Hello, World!";
    let written = file.write_at(0, data).unwrap();
    assert_eq!(written, data.len());

    let attr = file.get_attr().unwrap();
    assert_eq!(attr.size(), data.len() as u64);
}

#[test]
fn test_file_write_at_offset() {
    let fs = RamFileSystem::new();
    let root = fs.root_dir();
    root.create("test.txt", VfsNodeType::File).unwrap();
    let file = root.lookup("test.txt").unwrap();

    file.write_at(0, b"Hello, ").unwrap();
    let written = file.write_at(7, b"World!").unwrap();
    assert_eq!(written, 6);

    let mut buf = [0u8; 20];
    let read = file.read_at(0, &mut buf).unwrap();
    assert_eq!(read, 13);
    assert_eq!(&buf[..13], b"Hello, World!");
}

#[test]
fn test_file_read() {
    let fs = RamFileSystem::new();
    let root = fs.root_dir();
    root.create("test.txt", VfsNodeType::File).unwrap();
    let file = root.lookup("test.txt").unwrap();

    file.write_at(0, b"Hello, World!").unwrap();

    let mut buf = [0u8; 100];
    let read = file.read_at(0, &mut buf).unwrap();
    assert_eq!(read, 13);
    assert_eq!(&buf[..13], b"Hello, World!");
}

#[test]
fn test_file_read_partial() {
    let fs = RamFileSystem::new();
    let root = fs.root_dir();
    root.create("test.txt", VfsNodeType::File).unwrap();
    let file = root.lookup("test.txt").unwrap();

    file.write_at(0, b"Hello, World!").unwrap();

    let mut buf = [0u8; 5];
    let read = file.read_at(7, &mut buf).unwrap();
    assert_eq!(read, 5);
    assert_eq!(&buf[..5], b"World");
}

#[test]
fn test_file_read_empty() {
    let fs = RamFileSystem::new();
    let root = fs.root_dir();
    root.create("test.txt", VfsNodeType::File).unwrap();
    let file = root.lookup("test.txt").unwrap();

    let mut buf = [0u8; 100];
    let read = file.read_at(0, &mut buf).unwrap();
    assert_eq!(read, 0);
}

#[test]
fn test_file_truncate_extend() {
    let fs = RamFileSystem::new();
    let root = fs.root_dir();
    root.create("test.txt", VfsNodeType::File).unwrap();
    let file = root.lookup("test.txt").unwrap();

    file.write_at(0, b"Hello").unwrap();
    assert_eq!(file.get_attr().unwrap().size(), 5);

    file.truncate(100).unwrap();
    assert_eq!(file.get_attr().unwrap().size(), 100);

    let mut buf = [0u8; 200];
    let read = file.read_at(0, &mut buf).unwrap();
    assert_eq!(read, 100);
    assert_eq!(&buf[..5], b"Hello");
    assert!(&buf[5..100].iter().all(|&b| b == 0));
}

#[test]
fn test_file_truncate_shrink() {
    let fs = RamFileSystem::new();
    let root = fs.root_dir();
    root.create("test.txt", VfsNodeType::File).unwrap();
    let file = root.lookup("test.txt").unwrap();

    file.write_at(0, b"Hello, World!").unwrap();
    assert_eq!(file.get_attr().unwrap().size(), 13);

    file.truncate(5).unwrap();
    assert_eq!(file.get_attr().unwrap().size(), 5);

    let mut buf = [0u8; 100];
    let read = file.read_at(0, &mut buf).unwrap();
    assert_eq!(read, 5);
    assert_eq!(&buf[..5], b"Hello");
}

// ============== File Remove Tests ==============

#[test]
fn test_file_remove() {
    let fs = RamFileSystem::new();
    let root = fs.root_dir();
    root.create("test.txt", VfsNodeType::File).unwrap();

    let result = root.remove("test.txt");
    assert!(result.is_ok());

    let result = root.clone().lookup("test.txt");
    assert!(result.is_err());
}

#[test]
fn test_file_remove_nonexistent() {
    let fs = RamFileSystem::new();
    let root = fs.root_dir();

    let result = root.remove("nonexistent.txt");
    assert!(result.is_err());
}

// ============== Mount Operations Tests ==============

#[test]
fn test_ramfs_mount() {
    let fs = RamFileSystem::new();

    let root = fs.root_dir();
    let result = fs.mount("/", root);
    assert!(result.is_ok());
}

#[test]
fn test_ramfs_umount() {
    let fs = RamFileSystem::new();
    let root = fs.root_dir();

    root.create("test.txt", VfsNodeType::File).unwrap();
    let result = fs.umount();
    assert!(result.is_ok());
}

// ============== Hierarchy Tests ==============

#[test]
fn test_nested_directories() {
    let fs = RamFileSystem::new();
    let root = fs.root_dir();

    root.create("a", VfsNodeType::Dir).unwrap();
    root.create("a/b", VfsNodeType::Dir).unwrap();
    root.create("a/b/c", VfsNodeType::Dir).unwrap();
    root.create("a/b/c/file.txt", VfsNodeType::File).unwrap();

    let file = root.lookup("a/b/c/file.txt").unwrap();
    assert!(file.get_attr().unwrap().is_file());
}

#[test]
fn test_multiple_files_in_directory() {
    let fs = RamFileSystem::new();
    let root = fs.root_dir();

    root.create("dir", VfsNodeType::Dir).unwrap();
    let dir = root.lookup("dir").unwrap();

    for i in 0..10 {
        dir.create(&format!("file{}.txt", i), VfsNodeType::File)
            .unwrap();
    }

    let root_node = fs.root_dir_node();
    assert!(root_node.exist("dir"));

    let dir_node = dir.as_any().downcast_ref::<DirNode>().unwrap();
    assert_eq!(dir_node.get_entries().len(), 10);
}

#[test]
fn test_mixed_entries_in_directory() {
    let fs = RamFileSystem::new();
    let root = fs.root_dir();

    root.create("file1.txt", VfsNodeType::File).unwrap();
    root.create("dir1", VfsNodeType::Dir).unwrap();
    root.create("file2.txt", VfsNodeType::File).unwrap();
    root.create("dir2", VfsNodeType::Dir).unwrap();

    let root_node = fs.root_dir_node();
    let entries = root_node.get_entries();
    assert_eq!(entries.len(), 4);
}
