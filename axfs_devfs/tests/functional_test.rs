//! Functional tests for axfs_devfs.
//!
//! These tests verify the functionality of the device filesystem
//! using actual implementations rather than mocks.

use axfs_devfs::{DeviceFileSystem, NullDev, UrandomDev, ZeroDev};
use axfs_vfs::{VfsError, VfsNodeOps, VfsNodeType, VfsOps, VfsResult};
use std::sync::Arc;

fn test_devfs_ops(devfs: &DeviceFileSystem) -> VfsResult {
    const N: usize = 32;
    let mut buf = [1; N];

    let root = devfs.root_dir();
    assert!(root.get_attr()?.is_dir());
    assert_eq!(root.get_attr()?.file_type(), VfsNodeType::Dir);
    assert_eq!(
        root.clone().lookup("urandom").err(),
        Some(VfsError::NotFound)
    );
    assert_eq!(
        root.clone().lookup("zero/").err(),
        Some(VfsError::NotADirectory)
    );

    let node = root.lookup("////null")?;
    assert_eq!(node.get_attr()?.file_type(), VfsNodeType::CharDevice);
    assert!(!node.get_attr()?.is_dir());
    assert_eq!(node.get_attr()?.size(), 0);
    assert_eq!(node.read_at(0, &mut buf)?, 0);
    assert_eq!(buf, [1; N]);
    assert_eq!(node.write_at(N as _, &buf)?, N);
    assert_eq!(node.lookup("/").err(), Some(VfsError::NotADirectory));

    let node = devfs.root_dir().lookup(".///.//././/.////zero")?;
    assert_eq!(node.get_attr()?.file_type(), VfsNodeType::CharDevice);
    assert!(!node.get_attr()?.is_dir());
    assert_eq!(node.get_attr()?.size(), 0);
    assert_eq!(node.read_at(10, &mut buf)?, N);
    assert_eq!(buf, [0; N]);
    assert_eq!(node.write_at(0, &buf)?, N);

    let foo = devfs.root_dir().lookup(".///.//././/.////foo")?;
    assert!(foo.get_attr()?.is_dir());
    assert_eq!(
        foo.read_at(10, &mut buf).err(),
        Some(VfsError::IsADirectory)
    );
    assert!(Arc::ptr_eq(
        &foo.clone().lookup("/f2")?,
        &devfs.root_dir().lookup(".//./foo///f2")?,
    ));
    assert_eq!(
        foo.clone().lookup("/bar//f1")?.get_attr()?.file_type(),
        VfsNodeType::CharDevice
    );
    assert_eq!(
        foo.lookup("/bar///")?.get_attr()?.file_type(),
        VfsNodeType::Dir
    );

    Ok(())
}

fn test_get_parent(devfs: &DeviceFileSystem) -> VfsResult {
    let root = devfs.root_dir();
    assert!(root.parent().is_none());

    let node = root.clone().lookup("null")?;
    assert!(node.parent().is_none());

    let node = root.clone().lookup(".//foo/bar")?;
    assert!(node.parent().is_some());
    let parent = node.parent().unwrap();
    assert!(Arc::ptr_eq(&parent, &root.clone().lookup("foo")?));
    assert!(parent.lookup("bar").is_ok());

    let node = root.clone().lookup("foo/..")?;
    assert!(Arc::ptr_eq(&node, &root.clone().lookup(".")?));

    assert!(Arc::ptr_eq(
        &root.clone().lookup("/foo/..")?,
        &devfs.root_dir().lookup(".//./foo/././bar/../..")?,
    ));
    assert!(Arc::ptr_eq(
        &root.clone().lookup("././/foo//./../foo//bar///..//././")?,
        &devfs.root_dir().lookup(".//./foo/")?,
    ));
    assert!(Arc::ptr_eq(
        &root.clone().lookup("///foo//bar///../f2")?,
        &root.lookup("foo/.//f2")?,
    ));

    Ok(())
}

#[test]
fn test_devfs() {
    // .
    // ├── foo
    // │   ├── bar
    // │   │   └── f1 (null)
    // │   └── f2 (zero)
    // ├── null
    // └── zero

    let devfs = DeviceFileSystem::new();
    devfs.add("null", Arc::new(NullDev));
    devfs.add("zero", Arc::new(ZeroDev));

    let dir_foo = devfs.mkdir("foo");
    dir_foo.add("f2", Arc::new(ZeroDev));
    let dir_bar = dir_foo.mkdir("bar");
    dir_bar.add("f1", Arc::new(NullDev));

    test_devfs_ops(&devfs).unwrap();
    test_get_parent(&devfs).unwrap();
}

// ===== DeviceFileSystem Tests =====

#[test]
fn test_devfs_new() {
    let fs = DeviceFileSystem::new();
    let root = fs.root_dir();
    assert!(root.get_attr().is_ok());
}

#[test]
fn test_devfs_default() {
    let fs = DeviceFileSystem::default();
    let root = fs.root_dir();
    assert!(root.get_attr().is_ok());
}

#[test]
fn test_devfs_root_dir() {
    let fs = DeviceFileSystem::new();
    let root = fs.root_dir();
    assert_eq!(root.get_attr().unwrap().file_type(), VfsNodeType::Dir);
}

// ===== DirNode Tests =====

#[test]
fn test_dir_node_add_device() {
    let fs = DeviceFileSystem::new();
    let root = fs.root_dir();

    // Add null device
    let null: Arc<NullDev> = Arc::new(NullDev);
    fs.add("null", null);

    // Lookup the device
    let result = root.lookup("null");
    assert!(result.is_ok());
}

#[test]
fn test_dir_node_lookup_existing() {
    let fs = DeviceFileSystem::new();
    let root = fs.root_dir();

    let null: Arc<NullDev> = Arc::new(NullDev);
    fs.add("null", null.clone());

    let result = root.lookup("null").unwrap();
    assert_eq!(
        result.get_attr().unwrap().file_type(),
        VfsNodeType::CharDevice
    );
}

#[test]
fn test_dir_node_lookup_not_found() {
    let fs = DeviceFileSystem::new();
    let root = fs.root_dir();

    let result = root.lookup("nonexistent");
    assert!(result.is_err());
}

#[test]
fn test_dir_node_lookup_dot() {
    let fs = DeviceFileSystem::new();
    let root = fs.root_dir();

    let result = root.lookup(".").unwrap();
    assert_eq!(result.get_attr().unwrap().file_type(), VfsNodeType::Dir);
}

#[test]
fn test_dir_node_read_dir() {
    let fs = DeviceFileSystem::new();
    let root = fs.root_dir();

    let null: Arc<NullDev> = Arc::new(NullDev);
    fs.add("null", null);

    let mut dirents: Vec<axfs_vfs::VfsDirEntry> = (0..10)
        .map(|_| axfs_vfs::VfsDirEntry::new("", VfsNodeType::File))
        .collect();
    let count = root.read_dir(0, &mut dirents).unwrap();
    assert!(count >= 2); // At least . and ..

    // First entry should be .
    assert_eq!(dirents[0].name_as_bytes(), b".");
    assert_eq!(dirents[0].entry_type(), VfsNodeType::Dir);

    // Second entry should be ..
    assert_eq!(dirents[1].name_as_bytes(), b"..");
    assert_eq!(dirents[1].entry_type(), VfsNodeType::Dir);

    // Third entry should be null
    if count > 2 {
        assert_eq!(dirents[2].name_as_bytes(), b"null");
        assert_eq!(dirents[2].entry_type(), VfsNodeType::CharDevice);
    }
}

#[test]
fn test_dir_node_mkdir() {
    let fs = DeviceFileSystem::new();
    let subdir = fs.mkdir("subdir");

    assert!(subdir.get_attr().is_ok());
    assert_eq!(subdir.get_attr().unwrap().file_type(), VfsNodeType::Dir);
}

#[test]
fn test_dir_node_create_not_supported() {
    let fs = DeviceFileSystem::new();
    let root = fs.root_dir();

    // Create should return PermissionDenied for device filesystem
    let result = root.create("newfile", VfsNodeType::File);
    assert!(result.is_err());
}

#[test]
fn test_dir_node_remove_not_supported() {
    let fs = DeviceFileSystem::new();
    let root = fs.root_dir();

    // Remove should return PermissionDenied for device filesystem
    let result = root.remove("null");
    assert!(result.is_err());
}

#[test]
fn test_dir_node_parent() {
    let fs = DeviceFileSystem::new();
    let root = fs.root_dir();

    // Root directory should have no parent initially
    assert!(root.parent().is_none());
}

#[test]
fn test_dir_node_nested_lookup() {
    let fs = DeviceFileSystem::new();
    let root = fs.root_dir();

    let subdir = fs.mkdir("subdir");
    let null: Arc<NullDev> = Arc::new(NullDev);
    subdir.add("null", null);

    // Lookup nested device
    let result = root.lookup("subdir/null");
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap().get_attr().unwrap().file_type(),
        VfsNodeType::CharDevice
    );
}

// ===== NullDev Tests =====

#[test]
fn test_null_device_behavior() {
    let fs = DeviceFileSystem::new();
    let root = fs.root_dir();

    let null: Arc<NullDev> = Arc::new(NullDev);
    fs.add("null", null.clone());

    // Lookup and test null device
    let device = root.lookup("null").unwrap();

    // Read should return 0 bytes
    let mut buf = [0u8; 100];
    let n = device.read_at(0, &mut buf).unwrap();
    assert_eq!(n, 0);

    // Write should succeed but discard data
    let data = b"Hello, World!";
    let n = device.write_at(0, data).unwrap();
    assert_eq!(n, data.len());
}

#[test]
fn test_null_device_multiple_reads() {
    let fs = DeviceFileSystem::new();
    let root = fs.root_dir();

    let null: Arc<NullDev> = Arc::new(NullDev);
    fs.add("null", null.clone());

    let device = root.lookup("null").unwrap();

    // Multiple reads should all return 0 bytes
    for _ in 0..5 {
        let mut buf = [1u8; 50];
        let n = device.read_at(0, &mut buf).unwrap();
        assert_eq!(n, 0);
        // Buffer should remain unchanged
        assert_eq!(buf, [1u8; 50]);
    }
}

// ===== ZeroDev Tests =====

#[test]
fn test_zero_device_behavior() {
    let fs = DeviceFileSystem::new();
    let root = fs.root_dir();

    let zero: Arc<ZeroDev> = Arc::new(ZeroDev);
    fs.add("zero", zero.clone());

    // Lookup and test zero device
    let device = root.lookup("zero").unwrap();

    // Read should fill buffer with zeros
    let mut buf = [1u8; 100];
    let n = device.read_at(0, &mut buf).unwrap();
    assert_eq!(n, 100);
    assert_eq!(buf, [0u8; 100]);

    // Write should succeed but discard data
    let data = b"Hello, World!";
    let n = device.write_at(0, data).unwrap();
    assert_eq!(n, data.len());
}

#[test]
fn test_zero_device_multiple_reads() {
    let fs = DeviceFileSystem::new();
    let root = fs.root_dir();

    let zero: Arc<ZeroDev> = Arc::new(ZeroDev);
    fs.add("zero", zero.clone());

    let device = root.lookup("zero").unwrap();

    // Multiple reads should all return zeros
    for i in 0..3 {
        let mut buf = [i as u8 + 1; 50];
        let n = device.read_at(0, &mut buf).unwrap();
        assert_eq!(n, 50);
        assert_eq!(buf, [0u8; 50]);
    }
}

// ===== UrandomDev Tests =====

#[test]
fn test_urandom_device_behavior() {
    let fs = DeviceFileSystem::new();
    let root = fs.root_dir();

    let urandom: Arc<UrandomDev> = Arc::new(UrandomDev::new(12345));
    fs.add("urandom", urandom.clone());

    // Lookup and test urandom device
    let device = root.lookup("urandom").unwrap();

    // Read should fill buffer with random data
    let mut buf = [0u8; 100];
    let n = device.read_at(0, &mut buf).unwrap();
    assert_eq!(n, 100);

    // Data should be random (not all zeros)
    let all_zeros = buf.iter().all(|&b| b == 0);
    assert!(!all_zeros);
}

#[test]
fn test_urandom_device_deterministic() {
    let fs = DeviceFileSystem::new();
    let root = fs.root_dir();

    let urandom1: Arc<UrandomDev> = Arc::new(UrandomDev::new(12345));
    let urandom2: Arc<UrandomDev> = Arc::new(UrandomDev::new(12345));

    fs.add("urandom1", urandom1);
    fs.add("urandom2", urandom2);

    // Lookup devices
    let dev1 = root.clone().lookup("urandom1").unwrap();
    let dev2 = root.clone().lookup("urandom2").unwrap();

    // Both should produce same sequence
    let mut buf1 = [0u8; 100];
    let mut buf2 = [0u8; 100];

    dev1.read_at(0, &mut buf1).unwrap();
    dev2.read_at(0, &mut buf2).unwrap();

    assert_eq!(buf1, buf2);
}

#[test]
fn test_urandom_device_different_seeds() {
    let fs = DeviceFileSystem::new();
    let root = fs.root_dir();

    let urandom1: Arc<UrandomDev> = Arc::new(UrandomDev::new(12345));
    let urandom2: Arc<UrandomDev> = Arc::new(UrandomDev::new(67890));

    fs.add("urandom1", urandom1);
    fs.add("urandom2", urandom2);

    // Lookup devices
    let dev1 = root.clone().lookup("urandom1").unwrap();
    let dev2 = root.clone().lookup("urandom2").unwrap();

    // Different seeds should produce different sequences (likely)
    let mut buf1 = [0u8; 100];
    let mut buf2 = [0u8; 100];

    dev1.read_at(0, &mut buf1).unwrap();
    dev2.read_at(0, &mut buf2).unwrap();

    assert_ne!(buf1, buf2);
}

// ===== Multiple Devices Tests =====

#[test]
fn test_multiple_devices_in_root() {
    let fs = DeviceFileSystem::new();
    let root = fs.root_dir();

    // Add multiple devices
    let null: Arc<NullDev> = Arc::new(NullDev);
    let zero: Arc<ZeroDev> = Arc::new(ZeroDev);
    let urandom: Arc<UrandomDev> = Arc::new(UrandomDev::new(12345));

    fs.add("null", null);
    fs.add("zero", zero);
    fs.add("urandom", urandom);

    // Read directory entries
    let mut dirents: Vec<axfs_vfs::VfsDirEntry> = (0..10)
        .map(|_| axfs_vfs::VfsDirEntry::new("", VfsNodeType::File))
        .collect();
    let count = root.read_dir(0, &mut dirents).unwrap();
    assert!(count >= 5); // ., .., null, zero, urandom

    // Verify devices are present
    let entries: Vec<&[u8]> = dirents
        .iter()
        .take(count)
        .map(|e| e.name_as_bytes())
        .collect();
    let has_null = entries.iter().any(|&e| e == b"null");
    let has_zero = entries.iter().any(|&e| e == b"zero");
    let has_urandom = entries.iter().any(|&e| e == b"urandom");
    assert!(has_null);
    assert!(has_zero);
    assert!(has_urandom);
}

#[test]
fn test_devices_in_subdirectories() {
    let fs = DeviceFileSystem::new();
    let root = fs.root_dir();

    // Create subdirectories
    let subdir1 = fs.mkdir("char");
    let subdir2 = fs.mkdir("block");

    // Add devices to subdirectories
    let null: Arc<NullDev> = Arc::new(NullDev);
    let zero: Arc<ZeroDev> = Arc::new(ZeroDev);

    subdir1.add("null", null);
    subdir2.add("zero", zero);

    // Lookup devices via paths
    let dev1 = root.clone().lookup("char/null").unwrap();
    let dev2 = root.clone().lookup("block/zero").unwrap();

    assert_eq!(
        dev1.get_attr().unwrap().file_type(),
        VfsNodeType::CharDevice
    );
    assert_eq!(
        dev2.get_attr().unwrap().file_type(),
        VfsNodeType::CharDevice
    );
}

#[test]
fn test_mixed_devices_and_directories() {
    let fs = DeviceFileSystem::new();
    let root = fs.root_dir();

    // Add devices
    let null: Arc<NullDev> = Arc::new(NullDev);
    fs.add("null", null);

    // Create subdirectory with devices
    let subdir = fs.mkdir("random");
    let urandom: Arc<UrandomDev> = Arc::new(UrandomDev::new(12345));
    subdir.add("urandom", urandom);

    // Verify both are accessible
    assert!(root.clone().lookup("null").is_ok());
    assert!(root.clone().lookup("random/urandom").is_ok());
    assert!(root.clone().lookup("random").is_ok());
}
