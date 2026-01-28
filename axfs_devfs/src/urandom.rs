use axfs_vfs::{VfsNodeAttr, VfsNodeOps, VfsNodePerm, VfsNodeType, VfsResult};
use core::sync::atomic::{AtomicU64, Ordering};

/// A urandom device behaves like `/dev/urandom`.
///
/// This device generates pseudo-random bytes using a Linear Congruential
/// Generator (LCG) when read. It is useful for generating random data
/// for testing and non-cryptographic purposes.
///
/// # Behavior
///
/// - Read operations: Return pseudo-random bytes
/// - Write operations: Accept all data but discard it
/// - Truncate operations: Always succeed with no effect
///
/// # Unix Equivalent
///
/// This device behaves similarly to `/dev/urandom` in Unix-like systems,
/// though it uses a simple LCG and is not cryptographically secure.
///
/// # Algorithm
///
/// Uses a 64-bit LCG with the formula: `seed = seed * m + c`
/// where `m = 6364136223846793005` and `c = 1`.
pub struct UrandomDev {
    seed: AtomicU64,
}

impl UrandomDev {
    /// Creates a new urandom device with specified seed.
    ///
    /// # Arguments
    ///
    /// * `seed` - The initial seed value for the random number generator
    ///
    /// # Returns
    ///
    /// A new urandom device instance.
    pub const fn new(seed: u64) -> Self {
        Self {
            seed: AtomicU64::new(seed),
        }
    }

    /// Creates a new instance with a default seed.
    ///
    /// The default seed value is `0xa2ce_a2ce`.
    ///
    /// # Returns
    ///
    /// A new urandom device instance with default seed.
    fn new_with_default_seed() -> Self {
        Self::new(0xa2ce_a2ce)
    }

    /// Generates the next 64-bit pseudo-random number.
    ///
    /// This method uses a Linear Congruential Generator (LCG) algorithm:
    /// `seed = seed * 6364136223846793005 + 1`
    ///
    /// # Returns
    ///
    /// The next pseudo-random 64-bit value.
    fn next_u64(&self) -> u64 {
        let new_seed = self
            .seed
            .load(Ordering::SeqCst)
            .wrapping_mul(6364136223846793005)
            + 1;
        self.seed.store(new_seed, Ordering::SeqCst);
        new_seed
    }
}

impl Default for UrandomDev {
    /// Creates a default urandom device instance.
    ///
    /// This is equivalent to calling [`UrandomDev::new_with_default_seed()`].
    fn default() -> Self {
        Self::new_with_default_seed()
    }
}

impl VfsNodeOps for UrandomDev {
    /// Returns attributes of the urandom device.
    ///
    /// # Returns
    ///
    /// Returns character device attributes with zero size.
    fn get_attr(&self) -> VfsResult<VfsNodeAttr> {
        Ok(VfsNodeAttr::new(
            VfsNodePerm::default_file(),
            VfsNodeType::CharDevice,
            0,
            0,
        ))
    }

    /// Reads pseudo-random bytes from the device.
    ///
    /// This method fills the buffer with pseudo-random data.
    /// The offset parameter is ignored as this device generates
    /// fresh random data on each read.
    ///
    /// # Arguments
    ///
    /// * `_offset` - The read offset (ignored)
    /// * `buf` - The buffer to fill with random bytes
    ///
    /// # Returns
    ///
    /// Always returns the buffer length.
    fn read_at(&self, _offset: u64, buf: &mut [u8]) -> VfsResult<usize> {
        for chunk in buf.chunks_mut(8) {
            let random_value = self.next_u64();
            let bytes = random_value.to_ne_bytes();
            for (i, byte) in chunk.iter_mut().enumerate() {
                if i < bytes.len() {
                    *byte = bytes[i];
                }
            }
        }
        Ok(buf.len())
    }

    /// Writes to the urandom device.
    ///
    /// This operation discards all data.
    ///
    /// # Arguments
    ///
    /// * `_offset` - The write offset (ignored)
    /// * `buf` - The buffer containing data to write (discarded)
    ///
    /// # Returns
    ///
    /// Always returns the buffer length (but data is discarded).
    fn write_at(&self, _offset: u64, buf: &[u8]) -> VfsResult<usize> {
        Ok(buf.len())
    }

    /// Truncates the urandom device (no effect).
    ///
    /// This operation always succeeds.
    ///
    /// # Arguments
    ///
    /// * `_size` - The truncation size (ignored)
    ///
    /// # Returns
    ///
    /// Always returns `Ok(())`.
    fn truncate(&self, _size: u64) -> VfsResult {
        Ok(())
    }

    axfs_vfs::impl_vfs_non_dir_default! {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_urandom_new() {
        let _urandom = UrandomDev::new(12345);
        // Just ensure it doesn't panic
    }

    #[test]
    fn test_urandom_default() {
        let _urandom = UrandomDev::default();
        // Just ensure it doesn't panic
    }

    #[test]
    fn test_urandom_next_u64() {
        let urandom = UrandomDev::new(12345);
        let v1 = urandom.next_u64();
        let v2 = urandom.next_u64();
        // Values should be different (most likely)
        assert_ne!(v1, v2);
    }

    #[test]
    fn test_urandom_next_u64_deterministic() {
        let urandom1 = UrandomDev::new(12345);
        let urandom2 = UrandomDev::new(12345);

        // Same seed should produce same sequence
        let v1_1 = urandom1.next_u64();
        let v2_1 = urandom2.next_u64();
        assert_eq!(v1_1, v2_1);

        let v1_2 = urandom1.next_u64();
        let v2_2 = urandom2.next_u64();
        assert_eq!(v1_2, v2_2);
    }

    #[test]
    fn test_urandom_get_attr() {
        let urandom = UrandomDev::new(12345);
        let attr = urandom.get_attr().unwrap();
        assert_eq!(attr.file_type(), VfsNodeType::CharDevice);
        assert_eq!(attr.size(), 0);
        assert_eq!(attr.blocks(), 0);
    }

    #[test]
    fn test_urandom_read() {
        let urandom = UrandomDev::new(12345);
        let mut buf = [0; 100];
        let read = urandom.read_at(0, &mut buf).unwrap();
        assert_eq!(read, 100);
        // Buffer should be filled with random data
        // (unlikely to be all zeros)
        let all_zeros = buf.iter().all(|&b| b == 0);
        assert!(!all_zeros);
    }

    #[test]
    fn test_urandom_read_offset() {
        let urandom = UrandomDev::new(12345);
        let mut buf = [0; 50];
        let read = urandom.read_at(100, &mut buf).unwrap();
        assert_eq!(read, 50);
        // Offset is ignored, should still return random data
        let all_zeros = buf.iter().all(|&b| b == 0);
        assert!(!all_zeros);
    }

    #[test]
    fn test_urandom_read_empty() {
        let urandom = UrandomDev::new(12345);
        let mut buf = [];
        let read = urandom.read_at(0, &mut buf).unwrap();
        assert_eq!(read, 0);
    }

    #[test]
    fn test_urandom_read_deterministic() {
        let urandom1 = UrandomDev::new(12345);
        let urandom2 = UrandomDev::new(12345);

        let mut buf1 = [0; 100];
        let mut buf2 = [0; 100];

        urandom1.read_at(0, &mut buf1).unwrap();
        urandom2.read_at(0, &mut buf2).unwrap();

        // Same seed should produce same random sequence
        assert_eq!(buf1, buf2);
    }

    #[test]
    fn test_urandom_write() {
        let urandom = UrandomDev::new(12345);
        let data = b"Hello, World!";
        let written = urandom.write_at(0, data).unwrap();
        assert_eq!(written, data.len());
        // Data is discarded
    }

    #[test]
    fn test_urandom_write_offset() {
        let urandom = UrandomDev::new(12345);
        let data = b"Test";
        let written = urandom.write_at(100, data).unwrap();
        assert_eq!(written, data.len());
    }

    #[test]
    fn test_urandom_write_empty() {
        let urandom = UrandomDev::new(12345);
        let data: &[u8] = &[];
        let written = urandom.write_at(0, data).unwrap();
        assert_eq!(written, 0);
    }

    #[test]
    fn test_urandom_truncate() {
        let urandom = UrandomDev::new(12345);
        assert!(urandom.truncate(0).is_ok());
        assert!(urandom.truncate(100).is_ok());
        assert!(urandom.truncate(u64::MAX).is_ok());
    }

    #[test]
    fn test_urandom_combined_operations() {
        let urandom = UrandomDev::new(12345);

        // Write data (discarded)
        let data = b"Test data";
        urandom.write_at(0, data).unwrap();

        // Truncate
        urandom.truncate(10).unwrap();

        // Read should return random data
        let mut buf1 = [0; 50];
        let read = urandom.read_at(0, &mut buf1).unwrap();
        assert_eq!(read, 50);

        let all_zeros = buf1.iter().all(|&b| b == 0);
        assert!(!all_zeros);
    }
}
