//! System tests for axfs_ramfs
//!
//! This module contains system-level tests that verify the integration
//! of axfs_ramfs in a simulated operating system environment.
//! These tests verify the behavior of RAM filesystem in real-world scenarios.

use axfs_ramfs::{DirNode, RamFileSystem};
use axfs_vfs::{VfsDirEntry, VfsNodeType, VfsOps};

// ============== System-Level Integration Tests ==============

#[test]
fn test_system_complete_filesystem_lifecycle() {
    // Simulate a complete filesystem lifecycle:
    // mount -> create files/directories -> read/write -> cleanup -> unmount

    let fs = RamFileSystem::new();

    // Mount
    let root = fs.root_dir();
    fs.mount("/", root.clone()).unwrap();

    // Create directory structure
    root.create("home", VfsNodeType::Dir).unwrap();
    root.create("home/user", VfsNodeType::Dir).unwrap();
    root.create("home/user/documents", VfsNodeType::Dir)
        .unwrap();

    // Create files
    root.create("home/user/documents/readme.txt", VfsNodeType::File)
        .unwrap();
    root.create("home/user/documents/data.txt", VfsNodeType::File)
        .unwrap();

    // Write data
    let readme = root
        .clone()
        .lookup("home/user/documents/readme.txt")
        .unwrap();
    readme.write_at(0, b"Welcome to RAM filesystem!").unwrap();

    // Read data
    let mut buf = [0u8; 100];
    let n = readme.read_at(0, &mut buf).unwrap();
    assert_eq!(&buf[..n], b"Welcome to RAM filesystem!");

    // Cleanup
    root.remove("home/user/documents/data.txt").unwrap();
    root.remove("home/user/documents/readme.txt").unwrap();
    root.remove("home/user/documents").unwrap();
    root.remove("home/user").unwrap();
    root.remove("home").unwrap();

    // Unmount
    fs.umount().unwrap();
}

#[test]
fn test_system_hierarchical_directory_operations() {
    // Test complex directory hierarchies

    let fs = RamFileSystem::new();
    let root = fs.root_dir();

    // Create deep hierarchy
    root.create("level1", VfsNodeType::Dir).unwrap();
    root.create("level1/level2", VfsNodeType::Dir).unwrap();
    root.create("level1/level2/level3", VfsNodeType::Dir)
        .unwrap();
    root.create("level1/level2/level3/level4", VfsNodeType::Dir)
        .unwrap();

    // Create files at each level
    root.create("level1/file1.txt", VfsNodeType::File).unwrap();
    root.create("level1/level2/file2.txt", VfsNodeType::File)
        .unwrap();
    root.create("level1/level2/level3/file3.txt", VfsNodeType::File)
        .unwrap();
    root.create("level1/level2/level3/level4/file4.txt", VfsNodeType::File)
        .unwrap();

    // Verify each file exists
    assert!(root.clone().lookup("level1/file1.txt").is_ok());
    assert!(root.clone().lookup("level1/level2/file2.txt").is_ok());
    assert!(root
        .clone()
        .lookup("level1/level2/level3/file3.txt")
        .is_ok());
    assert!(root
        .clone()
        .lookup("level1/level2/level3/level4/file4.txt")
        .is_ok());

    // Write to each file
    let f1 = root.clone().lookup("level1/file1.txt").unwrap();
    let f2 = root.clone().lookup("level1/level2/file2.txt").unwrap();
    let f3 = root
        .clone()
        .lookup("level1/level2/level3/file3.txt")
        .unwrap();
    let f4 = root
        .clone()
        .lookup("level1/level2/level3/level4/file4.txt")
        .unwrap();

    f1.write_at(0, b"file at level 1").unwrap();
    f2.write_at(0, b"file at level 2").unwrap();
    f3.write_at(0, b"file at level 3").unwrap();
    f4.write_at(0, b"file at level 4").unwrap();

    // Verify contents
    let mut buf = [0u8; 100];
    let n1 = f1.read_at(0, &mut buf).unwrap();
    assert_eq!(&buf[..n1], b"file at level 1");
}

#[test]
fn test_system_bulk_file_operations() {
    // Test creating and managing many files

    let fs = RamFileSystem::new();
    let root = fs.root_dir();

    root.create("test_dir", VfsNodeType::Dir).unwrap();
    let test_dir = root.clone().lookup("test_dir").unwrap();

    // Create many files
    for i in 0..50 {
        let filename = format!("file_{:03}.txt", i);
        test_dir.create(&filename, VfsNodeType::File).unwrap();
    }

    // Write data to each file
    for i in 0..50 {
        let filename = format!("file_{:03}.txt", i);
        let file = test_dir.clone().lookup(&filename).unwrap();
        let content = format!("Content of file {}", i);
        file.write_at(0, content.as_bytes()).unwrap();
    }

    // Read and verify some files
    for i in [0, 25, 49].iter() {
        let filename = format!("file_{:03}.txt", i);
        let file = test_dir.clone().lookup(&filename).unwrap();
        let mut buf = [0u8; 100];
        let n = file.read_at(0, &mut buf).unwrap();
        let expected = format!("Content of file {}", i);
        assert_eq!(&buf[..n], expected.as_bytes());
    }
}

#[test]
fn test_system_file_content_modification() {
    // Test various file content modification scenarios

    let fs = RamFileSystem::new();
    let root = fs.root_dir();

    root.create("test.txt", VfsNodeType::File).unwrap();
    let file = root.clone().lookup("test.txt").unwrap();

    // Initial write
    file.write_at(0, b"Initial content").unwrap();
    assert_eq!(file.get_attr().unwrap().size(), 15);

    // Append by writing at end
    file.write_at(15, b", then more").unwrap();
    assert_eq!(file.get_attr().unwrap().size(), 26);

    // Read complete content
    let mut buf = [0u8; 100];
    let n = file.read_at(0, &mut buf).unwrap();
    assert_eq!(&buf[..n], b"Initial content, then more");

    // Overwrite middle
    file.write_at(8, b"CHANGED").unwrap();

    let mut buf2 = [0u8; 100];
    let n2 = file.read_at(0, &mut buf2).unwrap();
    assert_eq!(&buf2[..n2], b"Initial CHANGED, then more");
}

#[test]
fn test_system_directory_traversal() {
    // Test directory traversal with . and ..

    let fs = RamFileSystem::new();
    let root = fs.root_dir();

    // Create structure: root/a/b/c/file.txt
    root.create("a", VfsNodeType::Dir).unwrap();
    root.create("a/b", VfsNodeType::Dir).unwrap();
    root.create("a/b/c", VfsNodeType::Dir).unwrap();
    root.create("a/b/c/file.txt", VfsNodeType::File).unwrap();
    root.create("a/b/c/other.txt", VfsNodeType::File).unwrap();

    // Traverse from root
    let c = root.clone().lookup("a/b/c").unwrap();
    assert!(c.clone().lookup("file.txt").is_ok());
    assert!(c.clone().lookup("other.txt").is_ok());

    // Traverse up with ..
    let b = c.clone().lookup("..").unwrap();
    assert!(b.clone().lookup("c").is_ok());

    let a = b.lookup("..").unwrap();
    assert!(a.clone().lookup("b").is_ok());

    // Test . stays in same directory
    let c2 = c.clone().lookup(".").unwrap();
    assert!(c2.clone().lookup("file.txt").is_ok());

    // Complex path navigation - from c (a/b/c) go up multiple levels
    let root_via_c = c.clone().lookup("../../..").unwrap();
    assert!(root_via_c.clone().lookup("a").is_ok());
}

#[test]
fn test_system_directory_listing_with_pagination() {
    // Test reading directory entries with pagination

    let fs = RamFileSystem::new();
    let root = fs.root_dir();

    root.create("test", VfsNodeType::Dir).unwrap();
    let test_dir = root.clone().lookup("test").unwrap();

    // Create entries
    for i in 0..20 {
        let name = if i < 10 {
            format!("dir_{}", i)
        } else {
            format!("file_{}.txt", i - 10)
        };
        let ty = if i < 10 {
            VfsNodeType::Dir
        } else {
            VfsNodeType::File
        };
        test_dir.create(&name, ty).unwrap();
    }

    // Read in pages
    let mut page1: [VfsDirEntry; 5] = core::array::from_fn(|_| VfsDirEntry::default());
    let count1 = test_dir.read_dir(0, &mut page1).unwrap();

    let mut page2: [VfsDirEntry; 5] = core::array::from_fn(|_| VfsDirEntry::default());
    let count2 = test_dir.read_dir(5, &mut page2).unwrap();

    let mut page3: [VfsDirEntry; 5] = core::array::from_fn(|_| VfsDirEntry::default());
    let count3 = test_dir.read_dir(10, &mut page3).unwrap();

    // First 2 entries are . and ..
    assert!(count1 >= 3);
    assert!(count2 > 0);
    assert!(count3 > 0);
}

#[test]
fn test_system_file_truncate_operations() {
    // Test truncate in various scenarios

    let fs = RamFileSystem::new();
    let root = fs.root_dir();

    root.create("test.txt", VfsNodeType::File).unwrap();
    let file = root.clone().lookup("test.txt").unwrap();

    // Write initial data
    let data = b"0123456789ABCDEF";
    file.write_at(0, data).unwrap();
    assert_eq!(file.get_attr().unwrap().size(), 16);

    // Truncate to middle
    file.truncate(8).unwrap();
    assert_eq!(file.get_attr().unwrap().size(), 8);

    let mut buf = [0u8; 20];
    let n = file.read_at(0, &mut buf).unwrap();
    assert_eq!(&buf[..n], b"01234567");

    // Extend beyond original size
    file.truncate(20).unwrap();
    assert_eq!(file.get_attr().unwrap().size(), 20);

    let mut buf2 = [0u8; 30];
    file.read_at(0, &mut buf2).unwrap();
    assert_eq!(&buf2[..8], b"01234567");
    // Remaining bytes should be zeros
    assert!(&buf2[8..20].iter().all(|&b| b == 0));

    // Truncate to zero
    file.truncate(0).unwrap();
    assert_eq!(file.get_attr().unwrap().size(), 0);

    let mut buf3 = [0u8; 10];
    let n3 = file.read_at(0, &mut buf3).unwrap();
    assert_eq!(n3, 0);
}

#[test]
fn test_system_file_operations_at_various_offsets() {
    // Test file read/write at different offsets

    let fs = RamFileSystem::new();
    let root = fs.root_dir();

    root.create("test.txt", VfsNodeType::File).unwrap();
    let file = root.clone().lookup("test.txt").unwrap();

    // Write at different offsets
    file.write_at(0, b"00").unwrap();
    file.write_at(10, b"10").unwrap();
    file.write_at(20, b"20").unwrap();
    file.write_at(30, b"30").unwrap();

    // Verify size is updated - sparse file, size is last offset + write length
    assert_eq!(file.get_attr().unwrap().size(), 32);

    // Read from start - reads up to 10 bytes since file size is 32
    let mut buf = [0u8; 10];
    let n = file.read_at(0, &mut buf).unwrap();
    assert_eq!(n, 10);
    assert_eq!(&buf[..2], b"00");
    // The rest should be zeros until next write at offset 10
    assert_eq!(&buf[2..10], [0u8; 8]);

    let mut buf2 = [0u8; 10];
    let n2 = file.read_at(10, &mut buf2).unwrap();
    // read_at(10) reads from offset 10 onwards: "10" then zeros until buffer full
    assert_eq!(n2, 10);
    assert_eq!(&buf2[..2], b"10");
    // The rest should be zeros until next write at offset 20
    assert_eq!(&buf2[2..10], [0u8; 8]);
}

#[test]
fn test_system_multiple_filesystem_instances() {
    // Test that multiple filesystem instances are independent

    let fs1 = RamFileSystem::new();
    let fs2 = RamFileSystem::new();

    let root1 = fs1.root_dir();
    let root2 = fs2.root_dir();

    // Create different files in each
    root1.create("file1.txt", VfsNodeType::File).unwrap();
    root2.create("file2.txt", VfsNodeType::File).unwrap();

    let file1 = root1.clone().lookup("file1.txt").unwrap();
    let file2 = root2.clone().lookup("file2.txt").unwrap();

    file1.write_at(0, b"filesystem 1").unwrap();
    file2.write_at(0, b"filesystem 2").unwrap();

    // Verify independence
    let mut buf1 = [0u8; 50];
    let n1 = file1.read_at(0, &mut buf1).unwrap();
    assert_eq!(&buf1[..n1], b"filesystem 1");

    let mut buf2 = [0u8; 50];
    let n2 = file2.read_at(0, &mut buf2).unwrap();
    assert_eq!(&buf2[..n2], b"filesystem 2");

    // Verify each only has its own file
    assert!(root2.clone().lookup("file1.txt").is_err());
    assert!(root1.clone().lookup("file2.txt").is_err());
}

#[test]
fn test_system_cleanup_after_operations() {
    // Test filesystem cleanup after multiple operations

    let fs = RamFileSystem::new();
    let root = fs.root_dir();

    // Create complex structure
    root.create("tmp", VfsNodeType::Dir).unwrap();
    root.create("tmp/sub1", VfsNodeType::Dir).unwrap();
    root.create("tmp/sub2", VfsNodeType::Dir).unwrap();

    let sub1 = root.clone().lookup("tmp/sub1").unwrap();
    let sub2 = root.clone().lookup("tmp/sub2").unwrap();

    for i in 0..10 {
        sub1.create(&format!("f{}.txt", i), VfsNodeType::File)
            .unwrap();
    }
    for i in 0..10 {
        sub2.create(&format!("d{}", i), VfsNodeType::Dir).unwrap();
    }

    // Remove all sub1 files
    for i in 0..10 {
        assert!(sub1.remove(&format!("f{}.txt", i)).is_ok());
    }

    // Remove sub1
    root.remove("tmp/sub1").unwrap();

    // Remove all sub2 directories (need to remove files first)
    for i in 0..10 {
        let d = sub2.clone().lookup(&format!("d{}", i)).unwrap();
        assert!(d.remove("nonexistent").is_err()); // Empty dir
    }

    for i in 0..10 {
        assert!(sub2.remove(&format!("d{}", i)).is_ok());
    }

    // Remove sub2
    root.remove("tmp/sub2").unwrap();

    // Remove tmp
    root.remove("tmp").unwrap();

    // Verify clean
    let entries = fs.root_dir_node().get_entries();
    assert!(entries.is_empty());
}

#[test]
fn test_system_read_after_write_cycles() {
    // Test reading after multiple write cycles

    let fs = RamFileSystem::new();
    let root = fs.root_dir();

    root.create("test.txt", VfsNodeType::File).unwrap();
    let file = root.clone().lookup("test.txt").unwrap();

    // Multiple write/read cycles
    for i in 0..10 {
        let data = format!("Cycle {}", i);
        file.write_at(0, data.as_bytes()).unwrap();

        let mut buf = [0u8; 20];
        let n = file.read_at(0, &mut buf).unwrap();
        assert_eq!(&buf[..n], data.as_bytes());
    }

    // Write growing content
    for i in 1..11 {
        let data = vec![b'A'; i * 10];
        file.write_at(0, &data).unwrap();

        let mut buf = [0u8; 200];
        let n = file.read_at(0, &mut buf).unwrap();
        assert_eq!(&buf[..n], &data);
    }
}

#[test]
fn test_system_concurrent_like_operations() {
    // Test sequential operations that might occur concurrently

    let fs = RamFileSystem::new();
    let root = fs.root_dir();

    root.create("shared.txt", VfsNodeType::File).unwrap();
    let shared = root.clone().lookup("shared.txt").unwrap();

    // Simulate multiple operations
    shared.write_at(0, b"first").unwrap();
    let mut buf1 = [0u8; 20];
    let n1 = shared.read_at(0, &mut buf1).unwrap();
    assert_eq!(&buf1[..n1], b"first");

    shared.write_at(5, b"second").unwrap();
    shared.write_at(11, b"third").unwrap();

    let mut buf2 = [0u8; 30];
    let n2 = shared.read_at(0, &mut buf2).unwrap();
    // File is sparse: 0-4="first", 5-10="second", 11-15="third"
    // Size is 16 bytes (last offset 11 + len 5 = 16)
    assert_eq!(n2, 16);
    assert_eq!(&buf2[..5], b"first");
    assert_eq!(&buf2[5..11], b"second");
    assert_eq!(&buf2[11..16], b"third");

    // Read specific sections
    let mut buf3 = [0u8; 10];
    let n3 = shared.read_at(5, &mut buf3).unwrap();
    // read_at(5) with 10-byte buffer reads: "secondthir" (10 bytes)
    // The full content from offset 5 is "secondthird" (11 bytes), but buffer is only 10
    assert_eq!(n3, 10);
    assert_eq!(&buf3[..n3], b"secondthir");
}

#[test]
fn test_system_directory_listing_consistency() {
    // Test that directory listing remains consistent after operations

    let fs = RamFileSystem::new();
    let root = fs.root_dir();

    root.create("test", VfsNodeType::Dir).unwrap();
    let test_dir = root.clone().lookup("test").unwrap();

    // Create some entries
    test_dir.create("a", VfsNodeType::Dir).unwrap();
    test_dir.create("b", VfsNodeType::Dir).unwrap();
    test_dir.create("c.txt", VfsNodeType::File).unwrap();

    let root_node = fs.root_dir_node();
    assert!(root_node.exist("test"));

    let test_node = test_dir.as_any().downcast_ref::<DirNode>().unwrap();
    let entries1 = test_node.get_entries();
    assert_eq!(entries1.len(), 3);

    // Add more entries
    test_dir.create("d", VfsNodeType::Dir).unwrap();
    test_dir.create("e.txt", VfsNodeType::File).unwrap();

    let entries2 = test_node.get_entries();
    assert_eq!(entries2.len(), 5);

    // Remove entries
    test_dir.remove("b").unwrap();
    test_dir.remove("c.txt").unwrap();

    let entries3 = test_node.get_entries();
    assert_eq!(entries3.len(), 3);
    assert!(entries3.contains(&"a".to_string()));
    assert!(entries3.contains(&"d".to_string()));
    assert!(entries3.contains(&"e.txt".to_string()));
}

#[test]
fn test_system_large_file_operations() {
    // Test operations on larger files

    let fs = RamFileSystem::new();
    let root = fs.root_dir();

    root.create("large.txt", VfsNodeType::File).unwrap();
    let file = root.clone().lookup("large.txt").unwrap();

    // Write large data
    let large_data: Vec<u8> = (0..10000).map(|i| (i % 256) as u8).collect();
    file.write_at(0, &large_data).unwrap();

    assert_eq!(file.get_attr().unwrap().size(), 10000);

    // Read in chunks
    let mut buf = [0u8; 100];
    for chunk in 0..100 {
        let offset = chunk * 100;
        let n = file.read_at(offset as u64, &mut buf).unwrap();
        assert_eq!(n, 100);
        for i in 0..100 {
            assert_eq!(buf[i], large_data[offset + i]);
        }
    }

    // Truncate large file
    file.truncate(5000).unwrap();
    assert_eq!(file.get_attr().unwrap().size(), 5000);

    // Verify truncated content
    let mut buf2 = [0u8; 5000];
    let n2 = file.read_at(0, &mut buf2).unwrap();
    assert_eq!(n2, 5000);
    for i in 0..5000 {
        assert_eq!(buf2[i], large_data[i]);
    }
}

#[test]
fn test_system_attributes_consistency() {
    // Test that file/directory attributes remain consistent

    let fs = RamFileSystem::new();
    let root = fs.root_dir();

    root.create("test_dir", VfsNodeType::Dir).unwrap();
    root.create("test_file.txt", VfsNodeType::File).unwrap();

    let dir = root.clone().lookup("test_dir").unwrap();
    let file = root.clone().lookup("test_file.txt").unwrap();

    // Initial attributes
    let dir_attr1 = dir.get_attr().unwrap();
    let file_attr1 = file.get_attr().unwrap();
    assert!(dir_attr1.is_dir());
    assert!(!dir_attr1.is_file());
    assert!(file_attr1.is_file());
    assert!(!file_attr1.is_dir());
    assert_eq!(file_attr1.size(), 0);

    // Write to file
    file.write_at(0, b"test data").unwrap();

    // Updated attributes
    let file_attr2 = file.get_attr().unwrap();
    assert_eq!(file_attr2.size(), 9);

    // Directory attributes unchanged
    let dir_attr2 = dir.get_attr().unwrap();
    assert_eq!(dir_attr2.size(), dir_attr1.size());
}
