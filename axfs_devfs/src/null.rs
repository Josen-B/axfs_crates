use axfs_vfs::{VfsNodeAttr, VfsNodeOps, VfsNodePerm, VfsNodeType, VfsResult};

/// A null device behaves like `/dev/null`.
///
/// This device always returns 0 bytes on reads and discards all
/// written data. It is commonly used for discarding unwanted output.
///
/// # Behavior
///
/// - Read operations: Always return 0 bytes (end of file)
/// - Write operations: Accept all data but discard it
/// - Truncate operations: Always succeed with no effect
///
/// # Unix Equivalent
///
/// This device behaves similarly to `/dev/null` in Unix-like systems.
pub struct NullDev;

impl VfsNodeOps for NullDev {
    /// Returns attributes of the null device.
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

    /// Reads from the null device.
    ///
    /// This operation always returns 0 bytes (end of file).
    ///
    /// # Arguments
    ///
    /// * `_offset` - The read offset (ignored)
    /// * `_buf` - The buffer to read into (unchanged)
    ///
    /// # Returns
    ///
    /// Always returns 0 bytes read.
    fn read_at(&self, _offset: u64, _buf: &mut [u8]) -> VfsResult<usize> {
        Ok(0)
    }

    /// Writes to the null device.
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
    /// Always returns the number of bytes written (but data is discarded).
    fn write_at(&self, _offset: u64, buf: &[u8]) -> VfsResult<usize> {
        Ok(buf.len())
    }

    /// Truncates the null device (no effect).
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
