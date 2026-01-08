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
