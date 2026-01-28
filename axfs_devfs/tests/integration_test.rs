//! System tests for axfs_devfs
//!
//! This module contains system-level tests that verify integration
//! of axfs_devfs in a simulated operating system environment.
//! These tests verify the behavior of the device filesystem
//! in real-world scenarios.

use axfs_devfs::{DeviceFileSystem, NullDev, UrandomDev, ZeroDev};
use axfs_vfs::{VfsDirEntry, VfsNodeType, VfsOps};
use std::sync::Arc;

// ============== System-Level Integration Tests ==============

#[test]
fn test_system_complete_filesystem_lifecycle() {
    // Simulate a complete device filesystem lifecycle:
    // mount -> add devices -> access devices -> unmount

    let fs = DeviceFileSystem::new();

    // Mount
    let root = fs.root_dir();
    fs.mount("/", root.clone()).unwrap();

    // Add standard Unix-like devices
    let null: Arc<NullDev> = Arc::new(NullDev);
    let zero: Arc<ZeroDev> = Arc::new(ZeroDev);
    let urandom: Arc<UrandomDev> = Arc::new(UrandomDev::new(12345));

    fs.add("null", null);
    fs.add("zero", zero);
    fs.add("urandom", urandom);

    // Access and test each device
    let null_dev = root.clone().lookup("null").unwrap();
    let zero_dev = root.clone().lookup("zero").unwrap();
    let urandom_dev = root.clone().lookup("urandom").unwrap();

    // Test null device
    let mut buf = [1u8; 100];
    let n = null_dev.read_at(0, &mut buf).unwrap();
    assert_eq!(n, 0);
    assert_eq!(buf, [1u8; 100]); // Buffer unchanged

    // Test zero device
    let mut buf = [1u8; 100];
    let n = zero_dev.read_at(0, &mut buf).unwrap();
    assert_eq!(n, 100);
    assert_eq!(buf, [0u8; 100]);

    // Test urandom device
    let mut buf = [0u8; 100];
    let n = urandom_dev.read_at(0, &mut buf).unwrap();
    assert_eq!(n, 100);
    let all_zeros = buf.iter().all(|&b| b == 0);
    assert!(!all_zeros);

    // Unmount
    fs.umount().unwrap();
}

#[test]
fn test_system_device_organization_by_type() {
    // Test organizing devices in subdirectories by type

    let fs = DeviceFileSystem::new();
    let root = fs.root_dir();

    // Create device type directories
    let char_dev = fs.mkdir("char");
    let misc_dev = fs.mkdir("misc");

    // Add character devices to char directory
    let null: Arc<NullDev> = Arc::new(NullDev);
    let zero: Arc<ZeroDev> = Arc::new(ZeroDev);
    char_dev.add("null", null);
    char_dev.add("zero", zero);

    // Add misc devices
    let urandom: Arc<UrandomDev> = Arc::new(UrandomDev::new(12345));
    misc_dev.add("urandom", urandom);

    // Verify structure
    assert!(root.clone().lookup("char/null").is_ok());
    assert!(root.clone().lookup("char/zero").is_ok());
    assert!(root.clone().lookup("misc/urandom").is_ok());

    // Verify directory listings
    let mut dirents: Vec<VfsDirEntry> = (0..10)
        .map(|_| VfsDirEntry::new("", VfsNodeType::File))
        .collect();

    // List root - should have ., .., char, misc
    let count = root.read_dir(0, &mut dirents).unwrap();
    assert!(count >= 4);

    let entries: Vec<&[u8]> = dirents
        .iter()
        .take(count)
        .map(|e| e.name_as_bytes())
        .collect();
    let has_char = entries.iter().any(|&e| e == b"char");
    let has_misc = entries.iter().any(|&e| e == b"misc");
    assert!(has_char);
    assert!(has_misc);

    // List char directory - should have ., .., null, zero
    let char_node = root.clone().lookup("char").unwrap();
    dirents.clear();
    dirents.extend((0..10).map(|_| VfsDirEntry::new("", VfsNodeType::File)));
    let count = char_node.read_dir(0, &mut dirents).unwrap();
    assert!(count >= 4);

    let entries: Vec<&[u8]> = dirents
        .iter()
        .take(count)
        .map(|e| e.name_as_bytes())
        .collect();
    let has_null = entries.iter().any(|&e| e == b"null");
    let has_zero = entries.iter().any(|&e| e == b"zero");
    assert!(has_null);
    assert!(has_zero);
}

#[test]
fn test_system_multiple_filesystem_instances() {
    // Test that multiple filesystem instances are independent

    let fs1 = DeviceFileSystem::new();
    let fs2 = DeviceFileSystem::new();

    // Add different devices to each filesystem
    let null1: Arc<NullDev> = Arc::new(NullDev);
    let zero1: Arc<ZeroDev> = Arc::new(ZeroDev);

    fs1.add("null", null1);
    fs2.add("zero", zero1);

    // Each filesystem should only have its own devices
    let root1 = fs1.root_dir();
    let root2 = fs2.root_dir();

    assert!(root1.clone().lookup("null").is_ok());
    assert!(root1.clone().lookup("zero").is_err());

    assert!(root2.clone().lookup("zero").is_ok());
    assert!(root2.clone().lookup("null").is_err());
}

#[test]
fn test_system_device_write_behavior() {
    // Test that devices handle writes correctly

    let fs = DeviceFileSystem::new();
    let root = fs.root_dir();

    // Add devices
    let null: Arc<NullDev> = Arc::new(NullDev);
    let zero: Arc<ZeroDev> = Arc::new(ZeroDev);
    let urandom: Arc<UrandomDev> = Arc::new(UrandomDev::new(12345));

    fs.add("null", null);
    fs.add("zero", zero);
    fs.add("urandom", urandom);

    // All devices should accept writes but discard data
    let null_dev = root.clone().lookup("null").unwrap();
    let zero_dev = root.clone().lookup("zero").unwrap();
    let urandom_dev = root.clone().lookup("urandom").unwrap();

    let data = b"Important data to be discarded";
    let len = data.len();

    let n1 = null_dev.write_at(0, data).unwrap();
    assert_eq!(n1, len);

    let n2 = zero_dev.write_at(0, data).unwrap();
    assert_eq!(n2, len);

    let n3 = urandom_dev.write_at(0, data).unwrap();
    assert_eq!(n3, len);

    // Verify that reads are not affected by writes
    let mut buf = [0u8; 100];
    null_dev.read_at(0, &mut buf).unwrap();
    assert_eq!(buf, [0u8; 100]); // null returns 0 bytes, buffer unchanged

    let mut buf = [1u8; 100];
    zero_dev.read_at(0, &mut buf).unwrap();
    assert_eq!(buf, [0u8; 100]); // zero fills with zeros
}

#[test]
fn test_system_null_device_for_discarding_output() {
    // Test null device's practical use case: discarding output

    let fs = DeviceFileSystem::new();
    let root = fs.root_dir();

    let null: Arc<NullDev> = Arc::new(NullDev);
    fs.add("null", null);

    let dev = root.clone().lookup("null").unwrap();

    // Simulate discarding large amounts of data
    let large_data = vec![0u8; 1_000_000];
    let n = dev.write_at(0, &large_data).unwrap();
    assert_eq!(n, 1_000_000);

    // Data is discarded, nothing to verify
    // Just ensure it doesn't panic or error

    // Read still returns 0 bytes
    let mut buf = [1u8; 100];
    let n = dev.read_at(0, &mut buf).unwrap();
    assert_eq!(n, 0);
    assert_eq!(buf, [1u8; 100]);
}

#[test]
fn test_system_zero_device_for_initializing_data() {
    // Test zero device's practical use case: initializing zeroed buffers

    let fs = DeviceFileSystem::new();
    let root = fs.root_dir();

    let zero: Arc<ZeroDev> = Arc::new(ZeroDev);
    fs.add("zero", zero);

    let dev = root.clone().lookup("zero").unwrap();

    // Simulate allocating and zeroing memory
    let mut buffer1 = vec![42u8; 1024];
    let mut buffer2 = vec![99u8; 2048];
    let mut buffer3 = vec![255u8; 4096];

    dev.read_at(0, &mut buffer1).unwrap();
    dev.read_at(0, &mut buffer2).unwrap();
    dev.read_at(0, &mut buffer3).unwrap();

    assert_eq!(buffer1, vec![0u8; 1024]);
    assert_eq!(buffer2, vec![0u8; 2048]);
    assert_eq!(buffer3, vec![0u8; 4096]);
}

#[test]
fn test_system_urandom_device_randomness() {
    // Test urandom device produces sufficiently random data

    let fs = DeviceFileSystem::new();
    let root = fs.root_dir();

    let urandom: Arc<UrandomDev> = Arc::new(UrandomDev::new(12345));
    fs.add("urandom", urandom);

    let dev = root.clone().lookup("urandom").unwrap();

    // Generate multiple buffers of random data
    let mut buf1 = [0u8; 1024];
    let mut buf2 = [0u8; 1024];
    let mut buf3 = [0u8; 1024];

    dev.read_at(0, &mut buf1).unwrap();
    dev.read_at(0, &mut buf2).unwrap();
    dev.read_at(0, &mut buf3).unwrap();

    // All three buffers should be different
    assert_ne!(buf1, buf2);
    assert_ne!(buf2, buf3);
    assert_ne!(buf1, buf3);

    // Verify data distribution is reasonable
    let count_nonzero = buf1.iter().filter(|&&b| b != 0).count();
    assert!(count_nonzero > 500); // Most bytes should be non-zero
}

#[test]
fn test_system_readonly_filesystem_behavior() {
    // Test that device filesystem is effectively read-only

    let fs = DeviceFileSystem::new();
    let root = fs.root_dir();

    let null: Arc<NullDev> = Arc::new(NullDev);
    fs.add("null", null);

    // Attempt to create a new device (should fail)
    let result = root.create("new_device", VfsNodeType::CharDevice);
    assert!(result.is_err());

    // Attempt to remove an existing device (should fail)
    let result = root.remove("null");
    assert!(result.is_err());

    // Attempt to create a directory in mounted filesystem (should fail)
    let result = root.create("new_dir", VfsNodeType::Dir);
    assert!(result.is_err());

    // Original device should still exist
    assert!(root.clone().lookup("null").is_ok());
}

#[test]
fn test_system_large_data_operations() {
    // Test operations with large data sizes

    let fs = DeviceFileSystem::new();
    let root = fs.root_dir();

    let zero: Arc<ZeroDev> = Arc::new(ZeroDev);
    let urandom: Arc<UrandomDev> = Arc::new(UrandomDev::new(54321));

    fs.add("zero", zero);
    fs.add("urandom", urandom);

    let zero_dev = root.clone().lookup("zero").unwrap();
    let urandom_dev = root.clone().lookup("urandom").unwrap();

    // Test large read from zero device
    let mut large_zero_buf = vec![1u8; 100_000];
    let n = zero_dev.read_at(0, &mut large_zero_buf).unwrap();
    assert_eq!(n, 100_000);
    assert!(large_zero_buf.iter().all(|&b| b == 0));

    // Test large read from urandom device
    let mut large_random_buf = vec![0u8; 100_000];
    let n = urandom_dev.read_at(0, &mut large_random_buf).unwrap();
    assert_eq!(n, 100_000);
    let all_zeros = large_random_buf.iter().all(|&b| b == 0);
    assert!(!all_zeros);

    // Test large write to null device
    let large_data = vec![99u8; 1_000_000];
    let null: Arc<NullDev> = Arc::new(NullDev);
    fs.add("null", null);
    let null_dev = root.clone().lookup("null").unwrap();
    let n = null_dev.write_at(0, &large_data).unwrap();
    assert_eq!(n, 1_000_000);
}

#[test]
fn test_system_device_attributes_consistency() {
    // Test that device attributes remain consistent

    let fs = DeviceFileSystem::new();
    let root = fs.root_dir();

    let null: Arc<NullDev> = Arc::new(NullDev);
    let zero: Arc<ZeroDev> = Arc::new(ZeroDev);
    let urandom: Arc<UrandomDev> = Arc::new(UrandomDev::new(98765));

    fs.add("null", null);
    fs.add("zero", zero);
    fs.add("urandom", urandom);

    let null_dev = root.clone().lookup("null").unwrap();
    let zero_dev = root.clone().lookup("zero").unwrap();
    let urandom_dev = root.clone().lookup("urandom").unwrap();

    // All devices should be character devices
    assert_eq!(
        null_dev.get_attr().unwrap().file_type(),
        VfsNodeType::CharDevice
    );
    assert_eq!(
        zero_dev.get_attr().unwrap().file_type(),
        VfsNodeType::CharDevice
    );
    assert_eq!(
        urandom_dev.get_attr().unwrap().file_type(),
        VfsNodeType::CharDevice
    );

    // All devices should have zero size
    assert_eq!(null_dev.get_attr().unwrap().size(), 0);
    assert_eq!(zero_dev.get_attr().unwrap().size(), 0);
    assert_eq!(urandom_dev.get_attr().unwrap().size(), 0);

    // Attributes should remain consistent after operations
    let _ = null_dev.write_at(0, b"test").unwrap();
    let mut buf = [0u8; 100];
    let _ = null_dev.read_at(0, &mut buf).unwrap();
    assert_eq!(null_dev.get_attr().unwrap().size(), 0);

    let _ = zero_dev.write_at(0, b"test").unwrap();
    let _ = zero_dev.read_at(0, &mut buf).unwrap();
    assert_eq!(zero_dev.get_attr().unwrap().size(), 0);
}

#[test]
fn test_system_directory_traversal_depth() {
    // Test deep directory hierarchies for device organization

    let fs = DeviceFileSystem::new();
    let root = fs.root_dir();

    // Create deep hierarchy
    let l1 = fs.mkdir("level1");
    let l2 = l1.mkdir("level2");
    let l3 = l2.mkdir("level3");
    let l4 = l3.mkdir("level4");

    // Add devices at each level
    let null1: Arc<NullDev> = Arc::new(NullDev);
    let null2: Arc<NullDev> = Arc::new(NullDev);
    let null3: Arc<NullDev> = Arc::new(NullDev);
    let null4: Arc<NullDev> = Arc::new(NullDev);

    l1.add("null1", null1);
    l2.add("null2", null2);
    l3.add("null3", null3);
    l4.add("null4", null4);

    // Verify access through deep paths
    assert!(root.clone().lookup("level1/null1").is_ok());
    assert!(root.clone().lookup("level1/level2/null2").is_ok());
    assert!(root.clone().lookup("level1/level2/level3/null3").is_ok());
    assert!(root
        .clone()
        .lookup("level1/level2/level3/level4/null4")
        .is_ok());

    // Verify each level's directory listing
    let level1_node = root.clone().lookup("level1").unwrap();
    let mut dirents: Vec<VfsDirEntry> = (0..10)
        .map(|_| VfsDirEntry::new("", VfsNodeType::File))
        .collect();
    let count = level1_node.read_dir(0, &mut dirents).unwrap();
    let entries: Vec<&[u8]> = dirents
        .iter()
        .take(count)
        .map(|e| e.name_as_bytes())
        .collect();
    assert!(entries.iter().any(|&e| e == b"null1"));
    assert!(entries.iter().any(|&e| e == b"level2"));
}

#[test]
fn test_system_device_offset_operations() {
    // Test read/write at various offsets

    let fs = DeviceFileSystem::new();
    let root = fs.root_dir();

    let zero: Arc<ZeroDev> = Arc::new(ZeroDev);
    let urandom: Arc<UrandomDev> = Arc::new(UrandomDev::new(11111));

    fs.add("zero", zero);
    fs.add("urandom", urandom);

    let zero_dev = root.clone().lookup("zero").unwrap();
    let urandom_dev = root.clone().lookup("urandom").unwrap();

    // Test zero device at different offsets (offset ignored)
    let mut buf = [1u8; 100];
    zero_dev.read_at(0, &mut buf).unwrap();
    assert_eq!(buf, [0u8; 100]);

    let mut buf = [2u8; 100];
    zero_dev.read_at(1000, &mut buf).unwrap();
    assert_eq!(buf, [0u8; 100]);

    // Test urandom device at different offsets (offset ignored)
    let mut buf1 = [0u8; 50];
    let mut buf2 = [0u8; 50];
    urandom_dev.read_at(0, &mut buf1).unwrap();
    urandom_dev.read_at(100, &mut buf2).unwrap();
    // Both should be random, but different (different calls)
    assert_ne!(buf1, buf2);
}

#[test]
fn test_system_truncate_operations() {
    // Test truncate operations on devices (should succeed with no effect)

    let fs = DeviceFileSystem::new();
    let root = fs.root_dir();

    let null: Arc<NullDev> = Arc::new(NullDev);
    let zero: Arc<ZeroDev> = Arc::new(ZeroDev);
    let urandom: Arc<UrandomDev> = Arc::new(UrandomDev::new(22222));

    fs.add("null", null);
    fs.add("zero", zero);
    fs.add("urandom", urandom);

    let null_dev = root.clone().lookup("null").unwrap();
    let zero_dev = root.clone().lookup("zero").unwrap();
    let urandom_dev = root.clone().lookup("urandom").unwrap();

    // Truncate should succeed but have no effect
    assert!(null_dev.truncate(0).is_ok());
    assert!(null_dev.truncate(1000).is_ok());
    assert!(null_dev.truncate(u64::MAX).is_ok());

    assert!(zero_dev.truncate(0).is_ok());
    assert!(zero_dev.truncate(500).is_ok());
    assert!(zero_dev.truncate(u64::MAX).is_ok());

    assert!(urandom_dev.truncate(0).is_ok());
    assert!(urandom_dev.truncate(10000).is_ok());
    assert!(urandom_dev.truncate(u64::MAX).is_ok());

    // Device behavior should remain unchanged
    let mut buf = [1u8; 100];
    let n = null_dev.read_at(0, &mut buf).unwrap();
    assert_eq!(n, 0);

    zero_dev.read_at(0, &mut buf).unwrap();
    assert_eq!(buf, [0u8; 100]);

    urandom_dev.read_at(0, &mut buf).unwrap();
    let all_zeros = buf.iter().all(|&b| b == 0);
    assert!(!all_zeros);
}

#[test]
fn test_system_concurrent_device_access() {
    // Test accessing multiple devices independently

    let fs = DeviceFileSystem::new();
    let root = fs.root_dir();

    // Add multiple instances of each device type
    let devices = [
        ("null0", "zero0", "urandom0"),
        ("null1", "zero1", "urandom1"),
        ("null2", "zero2", "urandom2"),
        ("null3", "zero3", "urandom3"),
        ("null4", "zero4", "urandom4"),
    ];

    for (i, (null_name, zero_name, urandom_name)) in devices.iter().enumerate() {
        let null: Arc<NullDev> = Arc::new(NullDev);
        let zero: Arc<ZeroDev> = Arc::new(ZeroDev);
        let urandom: Arc<UrandomDev> = Arc::new(UrandomDev::new(33333 + i as u64));

        fs.add(null_name, null);
        fs.add(zero_name, zero);
        fs.add(urandom_name, urandom);
    }

    // Access all devices independently
    for (null_name, zero_name, urandom_name) in &devices {
        let null_dev = root.clone().lookup(null_name).unwrap();
        let zero_dev = root.clone().lookup(zero_name).unwrap();
        let urandom_dev = root.clone().lookup(urandom_name).unwrap();

        // Each device should work independently
        let mut buf = [1u8; 100];
        null_dev.read_at(0, &mut buf).unwrap();
        assert_eq!(buf, [1u8; 100]);

        zero_dev.read_at(0, &mut buf).unwrap();
        assert_eq!(buf, [0u8; 100]);

        urandom_dev.read_at(0, &mut buf).unwrap();
        let all_zeros = buf.iter().all(|&b| b == 0);
        assert!(!all_zeros);
    }
}
