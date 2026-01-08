use axfs_vfs::{VfsNodeAttr, VfsNodeOps, VfsNodePerm, VfsNodeType, VfsResult};

/// A zero device behaves like `/dev/zero`.
///
/// This device always returns null bytes (`\0`) on reads and discards
/// all written data. It is commonly used for creating zero-filled
/// files or buffers.
///
/// # Behavior
///
/// - Read operations: Always return buffer filled with null bytes
/// - Write operations: Accept all data but discard it
/// - Truncate operations: Always succeed with no effect
///
/// # Unix Equivalent
///
/// This device behaves similarly to `/dev/zero` in Unix-like systems.
pub struct ZeroDev;

impl VfsNodeOps for ZeroDev {
    /// Returns attributes of the zero device.
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

    /// Reads from the zero device.
    ///
    /// This operation fills the entire buffer with null bytes.
    ///
    /// # Arguments
    ///
    /// * `_offset` - The read offset (ignored)
    /// * `buf` - The buffer to fill with null bytes
    ///
    /// # Returns
    ///
    /// Always returns the buffer length.
    fn read_at(&self, _offset: u64, buf: &mut [u8]) -> VfsResult<usize> {
        buf.fill(0);
        Ok(buf.len())
    }

    /// Writes to the zero device.
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

    /// Truncates the zero device (no effect).
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
