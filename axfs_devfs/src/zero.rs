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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_dev_get_attr() {
        let zero = ZeroDev;
        let attr = zero.get_attr().unwrap();
        assert_eq!(attr.file_type(), VfsNodeType::CharDevice);
        assert_eq!(attr.size(), 0);
        assert_eq!(attr.blocks(), 0);
    }

    #[test]
    fn test_zero_dev_read() {
        let zero = ZeroDev;
        let mut buf = [1; 100];
        let read = zero.read_at(0, &mut buf).unwrap();
        assert_eq!(read, 100);
        // Buffer should be filled with zeros
        assert_eq!(buf, [0; 100]);
    }

    #[test]
    fn test_zero_dev_read_offset() {
        let zero = ZeroDev;
        let mut buf = [1; 50];
        let read = zero.read_at(100, &mut buf).unwrap();
        assert_eq!(read, 50);
        // Offset is ignored, buffer should still be filled with zeros
        assert_eq!(buf, [0; 50]);
    }

    #[test]
    fn test_zero_dev_read_empty() {
        let zero = ZeroDev;
        let mut buf = [];
        let read = zero.read_at(0, &mut buf).unwrap();
        assert_eq!(read, 0);
    }

    #[test]
    fn test_zero_dev_write() {
        let zero = ZeroDev;
        let data = b"Hello, World!";
        let written = zero.write_at(0, data).unwrap();
        assert_eq!(written, data.len());
        // Data is discarded
    }

    #[test]
    fn test_zero_dev_write_offset() {
        let zero = ZeroDev;
        let data = b"Test";
        let written = zero.write_at(100, data).unwrap();
        assert_eq!(written, data.len());
    }

    #[test]
    fn test_zero_dev_write_empty() {
        let zero = ZeroDev;
        let data: &[u8] = &[];
        let written = zero.write_at(0, data).unwrap();
        assert_eq!(written, 0);
    }

    #[test]
    fn test_zero_dev_truncate() {
        let zero = ZeroDev;
        assert!(zero.truncate(0).is_ok());
        assert!(zero.truncate(100).is_ok());
        assert!(zero.truncate(u64::MAX).is_ok());
    }

    #[test]
    fn test_zero_dev_combined_operations() {
        let zero = ZeroDev;

        // Write data (discarded)
        let data = b"Test data";
        zero.write_at(0, data).unwrap();

        // Truncate
        zero.truncate(10).unwrap();

        // Read should return zeros
        let mut buf = [1; 50];
        let read = zero.read_at(0, &mut buf).unwrap();
        assert_eq!(read, 50);
        assert_eq!(buf, [0; 50]);
    }
}
