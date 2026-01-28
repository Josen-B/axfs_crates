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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_null_dev_get_attr() {
        let null = NullDev;
        let attr = null.get_attr().unwrap();
        assert_eq!(attr.file_type(), VfsNodeType::CharDevice);
        assert_eq!(attr.size(), 0);
        assert_eq!(attr.blocks(), 0);
    }

    #[test]
    fn test_null_dev_read() {
        let null = NullDev;
        let mut buf = [0; 100];
        let read = null.read_at(0, &mut buf).unwrap();
        assert_eq!(read, 0);
        // Buffer should remain unchanged
        assert_eq!(buf, [0; 100]);
    }

    #[test]
    fn test_null_dev_read_offset() {
        let null = NullDev;
        let mut buf = [1; 50];
        let read = null.read_at(100, &mut buf).unwrap();
        assert_eq!(read, 0);
        // Buffer should remain unchanged
        assert_eq!(buf, [1; 50]);
    }

    #[test]
    fn test_null_dev_write() {
        let null = NullDev;
        let data = b"Hello, World!";
        let written = null.write_at(0, data).unwrap();
        assert_eq!(written, data.len());
        // Data is discarded, so we can't verify it, just ensure it doesn't panic
    }

    #[test]
    fn test_null_dev_write_offset() {
        let null = NullDev;
        let data = b"Test";
        let written = null.write_at(100, data).unwrap();
        assert_eq!(written, data.len());
    }

    #[test]
    fn test_null_dev_write_empty() {
        let null = NullDev;
        let data: &[u8] = &[];
        let written = null.write_at(0, data).unwrap();
        assert_eq!(written, 0);
    }

    #[test]
    fn test_null_dev_truncate() {
        let null = NullDev;
        assert!(null.truncate(0).is_ok());
        assert!(null.truncate(100).is_ok());
        assert!(null.truncate(u64::MAX).is_ok());
    }

    #[test]
    fn test_null_dev_combined_operations() {
        let null = NullDev;
        
        // Write data
        let data = b"Test data";
        null.write_at(0, data).unwrap();
        
        // Truncate
        null.truncate(10).unwrap();
        
        // Read should still return 0 bytes
        let mut buf = [0; 100];
        let read = null.read_at(0, &mut buf).unwrap();
        assert_eq!(read, 0);
    }
}
