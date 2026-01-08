//! Virtual filesystem interfaces used by [ArceOS](https://github.com/arceos-org/arceos).
//!
//! A filesystem is a set of files and directories (symbol links are not
//! supported currently), collectively referred to as **nodes**, which are
//! conceptually similar to [inodes] in Linux. A file system needs to implement
//! the [`VfsOps`] trait, its files and directories need to implement the
//! [`VfsNodeOps`] trait.
//!
//! The [`VfsOps`] trait provides the following operations on a filesystem:
//!
//! - [`mount()`](VfsOps::mount): Do something when the filesystem is mounted.
//! - [`umount()`](VfsOps::umount): Do something when the filesystem is unmounted.
//! - [`format()`](VfsOps::format): Format the filesystem.
//! - [`statfs()`](VfsOps::statfs): Get the attributes of the filesystem.
//! - [`root_dir()`](VfsOps::root_dir): Get root directory of the filesystem.
//!
//! The [`VfsNodeOps`] trait provides the following operations on a file or a
//! directory:
//!
//! | Operation | Description | file/directory |
//! | --- | --- | --- |
//! | [`open()`](VfsNodeOps::open) | Do something when the node is opened | both |
//! | [`release()`](VfsNodeOps::release) | Do something when the node is closed | both |
//! | [`get_attr()`](VfsNodeOps::get_attr) | Get the attributes of the node | both |
//! | [`read_at()`](VfsNodeOps::read_at) | Read data from the file | file |
//! | [`write_at()`](VfsNodeOps::write_at) | Write data to the file | file |
//! | [`fsync()`](VfsNodeOps::fsync) | Synchronize the file data to disk | file |
//! | [`truncate()`](VfsNodeOps::truncate) | Truncate the file | file |
//! | [`parent()`](VfsNodeOps::parent) | Get the parent directory | directory |
//! | [`lookup()`](VfsNodeOps::lookup) | Lookup the node with the given path | directory |
//! | [`create()`](VfsNodeOps::create) | Create a new node with the given path | directory |
//! | [`remove()`](VfsNodeOps::remove) | Remove the node with the given path | directory |
//! | [`read_dir()`](VfsNodeOps::read_dir) | Read directory entries | directory |
//!
//! [inodes]: https://en.wikipedia.org/wiki/Inode

#![no_std]

extern crate alloc;

mod macros;
mod structs;

pub mod path;

use alloc::sync::Arc;
use axerrno::{ax_err, AxError, AxResult};

pub use self::structs::{FileSystemInfo, VfsDirEntry, VfsNodeAttr, VfsNodePerm, VfsNodeType};

/// A wrapper of [`Arc<dyn VfsNodeOps>`].
///
/// This type is used to share ownership of a VFS node across threads.
pub type VfsNodeRef = Arc<dyn VfsNodeOps>;

/// Alias of [`AxError`].
///
/// Represents errors that can occur during VFS operations.
pub type VfsError = AxError;

/// Alias of [`AxResult`].
///
/// A result type for VFS operations that returns `Ok(T)` on success
/// and [`VfsError`] on failure.
///
/// # Type Parameters
///
/// * `T` - The type of the success value. Defaults to `()`.
pub type VfsResult<T = ()> = AxResult<T>;

/// Filesystem operations.
///
/// This trait defines the operations that a filesystem must implement.
/// All operations are safe by default and can be called concurrently from
/// multiple threads (hence `Send + Sync` bounds).
pub trait VfsOps: Send + Sync {
    /// Do something when the filesystem is mounted.
    ///
    /// This method is called when the filesystem is mounted at a specific path.
    /// The default implementation does nothing.
    ///
    /// # Arguments
    ///
    /// * `_path` - The path where the filesystem is being mounted
    /// * `_mount_point` - A reference to the mount point directory node
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the mount operation succeeds, or an error otherwise.
    fn mount(&self, _path: &str, _mount_point: VfsNodeRef) -> VfsResult {
        Ok(())
    }

    /// Do something when the filesystem is unmounted.
    ///
    /// This method is called when the filesystem is unmounted.
    /// The default implementation does nothing.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the unmount operation succeeds, or an error otherwise.
    fn umount(&self) -> VfsResult {
        Ok(())
    }

    /// Format the filesystem.
    ///
    /// This method formats the filesystem, erasing all existing data.
    /// The default implementation returns [`AxError::Unsupported`].
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if formatting succeeds, or an error otherwise.
    ///
    /// # Errors
    ///
    /// Returns [`AxError::Unsupported`] if the filesystem does not support formatting.
    fn format(&self) -> VfsResult {
        ax_err!(Unsupported)
    }

    /// Get the attributes of the filesystem.
    ///
    /// This method retrieves information about the filesystem, such as
    /// total size, available space, etc.
    /// The default implementation returns [`AxError::Unsupported`].
    ///
    /// # Returns
    ///
    /// Returns a [`FileSystemInfo`] containing filesystem attributes on success,
    /// or an error otherwise.
    ///
    /// # Errors
    ///
    /// Returns [`AxError::Unsupported`] if the filesystem does not support this operation.
    fn statfs(&self) -> VfsResult<FileSystemInfo> {
        ax_err!(Unsupported)
    }

    /// Get the root directory of the filesystem.
    ///
    /// This method returns a reference to the root directory node of the filesystem.
    /// This is the only required method that must be implemented.
    ///
    /// # Returns
    ///
    /// Returns a [`VfsNodeRef`] to the root directory.
    fn root_dir(&self) -> VfsNodeRef;
}

/// Node (file/directory) operations.
///
/// This trait defines the operations that can be performed on a VFS node,
/// which can be either a file or a directory. All operations are safe by
/// default and can be called concurrently from multiple threads.
///
/// # File Operations
///
/// The following methods are specific to files:
/// - [`read_at`](Self::read_at) - Read data from a file
/// - [`write_at`](Self::write_at) - Write data to a file
/// - [`fsync`](Self::fsync) - Synchronize file data to disk
/// - [`truncate`](Self::truncate) - Truncate a file
///
/// # Directory Operations
///
/// The following methods are specific to directories:
/// - [`parent`](Self::parent) - Get the parent directory
/// - [`lookup`](Self::lookup) - Look up a node by path
/// - [`create`](Self::create) - Create a new node
/// - [`remove`](Self::remove) - Remove a node
/// - [`read_dir`](Self::read_dir) - Read directory entries
/// - [`rename`](Self::rename) - Rename or move a node
pub trait VfsNodeOps: Send + Sync {
    /// Do something when the node is opened.
    ///
    /// This method is called when a node is opened for access.
    /// The default implementation does nothing.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the open operation succeeds, or an error otherwise.
    fn open(&self) -> VfsResult {
        Ok(())
    }

    /// Do something when the node is closed.
    ///
    /// This method is called when a node is closed after use.
    /// The default implementation does nothing.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the release operation succeeds, or an error otherwise.
    fn release(&self) -> VfsResult {
        Ok(())
    }

    /// Get the attributes of the node.
    ///
    /// This method retrieves metadata about the node, including its type,
    /// size, permissions, etc.
    /// The default implementation returns [`AxError::Unsupported`].
    ///
    /// # Returns
    ///
    /// Returns a [`VfsNodeAttr`] containing the node's attributes on success,
    /// or an error otherwise.
    ///
    /// # Errors
    ///
    /// Returns [`AxError::Unsupported`] if the node does not support this operation.
    fn get_attr(&self) -> VfsResult<VfsNodeAttr> {
        ax_err!(Unsupported)
    }

    // file operations:

    /// Read data from the file at the given offset.
    ///
    /// This method reads up to `buf.len()` bytes from the file starting at
    /// `offset`. The actual number of bytes read is returned.
    /// The default implementation returns [`AxError::InvalidInput`].
    ///
    /// # Arguments
    ///
    /// * `offset` - The offset in the file to start reading from
    /// * `buf` - The buffer to read data into
    ///
    /// # Returns
    ///
    /// Returns the number of bytes actually read on success, or an error otherwise.
    ///
    /// # Errors
    ///
    /// Returns [`AxError::InvalidInput`] if called on a non-file node.
    fn read_at(&self, _offset: u64, _buf: &mut [u8]) -> VfsResult<usize> {
        ax_err!(InvalidInput)
    }

    /// Write data to the file at the given offset.
    ///
    /// This method writes up to `buf.len()` bytes to the file starting at
    /// `offset`. The actual number of bytes written is returned.
    /// The default implementation returns [`AxError::InvalidInput`].
    ///
    /// # Arguments
    ///
    /// * `offset` - The offset in the file to start writing to
    /// * `buf` - The buffer containing the data to write
    ///
    /// # Returns
    ///
    /// Returns the number of bytes actually written on success, or an error otherwise.
    ///
    /// # Errors
    ///
    /// Returns [`AxError::InvalidInput`] if called on a non-file node.
    fn write_at(&self, _offset: u64, _buf: &[u8]) -> VfsResult<usize> {
        ax_err!(InvalidInput)
    }

    /// Flush the file, synchronize the data to disk.
    ///
    /// This method ensures that all data written to the file is persisted
    /// to the underlying storage device.
    /// The default implementation returns [`AxError::InvalidInput`].
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if synchronization succeeds, or an error otherwise.
    ///
    /// # Errors
    ///
    /// Returns [`AxError::InvalidInput`] if called on a non-file node.
    fn fsync(&self) -> VfsResult {
        ax_err!(InvalidInput)
    }

    /// Truncate the file to the given size.
    ///
    /// If `size` is larger than the current file size, the file is extended
    /// with zeros. If `size` is smaller, the file is truncated.
    /// The default implementation returns [`AxError::InvalidInput`].
    ///
    /// # Arguments
    ///
    /// * `size` - The new size of the file in bytes
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if truncation succeeds, or an error otherwise.
    ///
    /// # Errors
    ///
    /// Returns [`AxError::InvalidInput`] if called on a non-file node.
    fn truncate(&self, _size: u64) -> VfsResult {
        ax_err!(InvalidInput)
    }

    // directory operations:

    /// Get the parent directory of this directory.
    ///
    /// This method returns a reference to the parent directory of the current node.
    /// The default implementation returns `None`.
    ///
    /// # Returns
    ///
    /// Returns `Some(VfsNodeRef)` if the node has a parent directory,
    /// or `None` if the node has no parent (e.g., it's a file or root directory).
    fn parent(&self) -> Option<VfsNodeRef> {
        None
    }

    /// Lookup the node with given `path` in the directory.
    ///
    /// This method searches for a node with the specified relative path within
    /// the directory. If found, it returns a reference to the node.
    /// The default implementation returns [`AxError::Unsupported`].
    ///
    /// # Arguments
    ///
    /// * `path` - The relative path to look up (e.g., "subdir/file.txt")
    ///
    /// # Returns
    ///
    /// Returns a [`VfsNodeRef`] to the found node, or an error if not found.
    ///
    /// # Errors
    ///
    /// Returns [`AxError::Unsupported`] if the directory does not support lookup,
    /// or [`AxError::NotFound`] if the path does not exist.
    fn lookup(self: Arc<Self>, _path: &str) -> VfsResult<VfsNodeRef> {
        ax_err!(Unsupported)
    }

    /// Create a new node with the given `path` in the directory.
    ///
    /// This method creates a new file or directory with the specified path.
    /// If the node already exists, the method returns `Ok(())`.
    /// The default implementation returns [`AxError::Unsupported`].
    ///
    /// # Arguments
    ///
    /// * `path` - The path for the new node
    /// * `ty` - The type of node to create (file or directory)
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the node was created or already exists,
    /// or an error otherwise.
    ///
    /// # Errors
    ///
    /// Returns [`AxError::Unsupported`] if the directory does not support creation.
    fn create(&self, _path: &str, _ty: VfsNodeType) -> VfsResult {
        ax_err!(Unsupported)
    }

    /// Remove the node with the given `path` in the directory.
    ///
    /// This method removes a file or directory at the specified path.
    /// The default implementation returns [`AxError::Unsupported`].
    ///
    /// # Arguments
    ///
    /// * `path` - The path of the node to remove
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the node was removed successfully, or an error otherwise.
    ///
    /// # Errors
    ///
    /// Returns [`AxError::Unsupported`] if the directory does not support removal.
    fn remove(&self, _path: &str) -> VfsResult {
        ax_err!(Unsupported)
    }

    /// Read directory entries into `dirents`, starting from `start_idx`.
    ///
    /// This method reads directory entries (files and subdirectories) into
    /// the provided buffer, starting from the specified index. This allows
    /// for pagination of directory contents.
    /// The default implementation returns [`AxError::Unsupported`].
    ///
    /// # Arguments
    ///
    /// * `start_idx` - The starting index for reading entries (for pagination)
    /// * `dirents` - A mutable slice to store the directory entries
    ///
    /// # Returns
    ///
    /// Returns the number of entries read on success, or an error otherwise.
    ///
    /// # Errors
    ///
    /// Returns [`AxError::Unsupported`] if called on a non-directory node.
    fn read_dir(&self, _start_idx: usize, _dirents: &mut [VfsDirEntry]) -> VfsResult<usize> {
        ax_err!(Unsupported)
    }

    /// Renames or moves existing file or directory.
    ///
    /// This method renames or moves a node from `src_path` to `dst_path`.
    /// The operation can be within the same directory or across directories.
    /// The default implementation returns [`AxError::Unsupported`].
    ///
    /// # Arguments
    ///
    /// * `src_path` - The source path of the node to rename/move
    /// * `dst_path` - The destination path for the node
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the rename/move operation succeeds, or an error otherwise.
    ///
    /// # Errors
    ///
    /// Returns [`AxError::Unsupported`] if the directory does not support renaming.
    fn rename(&self, _src_path: &str, _dst_path: &str) -> VfsResult {
        ax_err!(Unsupported)
    }

    /// Convert `&self` to [`&dyn Any`][1] that can use
    /// [`Any::downcast_ref`][2].
    ///
    /// This method enables downcasting to the concrete type that implements
    /// this trait. This is useful for accessing implementation-specific methods.
    /// The default implementation returns `unimplemented!()` and must be
    /// implemented by all concrete types.
    ///
    /// # Returns
    ///
    /// Returns a reference to the node as `&dyn Any`.
    ///
    /// [1]: core::any::Any
    /// [2]: core::any::Any#method.downcast_ref
    fn as_any(&self) -> &dyn core::any::Any {
        unimplemented!()
    }
}

#[doc(hidden)]
pub mod __priv {
    //! Private module used internally by macros.
    //!
    //! This module re-exports types and functions that are needed by
    //! the declarative macros in this crate, but are not part of the
    //! public API.

    pub use alloc::sync::Arc;
    pub use axerrno::ax_err;
}
