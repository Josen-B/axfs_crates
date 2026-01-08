//! RAM filesystem used by [ArceOS](https://github.com/arceos-org/arceos).
//!
//! This crate provides a simple in-memory filesystem implementation based on
//! [`axfs_vfs`]. It is fully stored in RAM and does not persist to disk,
//! making it ideal for temporary files, testing, and scenarios where
//! fast access is needed.
//!
//! # Components
//!
//! - [`RamFileSystem`] - The main filesystem structure implementing filesystem operations
//! - [`DirNode`] - Directory node implementing directory operations
//! - [`FileNode`] - File node implementing file operations
//!
//! # Features
//!
//! - Full support for file and directory operations
//! - Hierarchical directory structure
//! - In-memory storage with fast access
//! - Compatible with the axfs_vfs interface
//!
//! [`axfs_vfs`]: axfs_vfs

#![cfg_attr(not(test), no_std)]

extern crate alloc;

mod dir;
mod file;

#[cfg(test)]
mod tests;

pub use self::dir::DirNode;
pub use self::file::FileNode;

use alloc::sync::Arc;
use axfs_vfs::{VfsNodeRef, VfsOps, VfsResult};
use spin::once::Once;

/// A RAM filesystem that implements VFS operations.
///
/// This is an in-memory filesystem that stores all data in RAM.
/// It provides fast access but does not persist data across reboots.
///
/// # Fields
///
/// - `parent` - The parent filesystem mount point
/// - `root` - The root directory of the RAM filesystem
pub struct RamFileSystem {
    parent: Once<VfsNodeRef>,
    root: Arc<DirNode>,
}

impl RamFileSystem {
    /// Create a new RAM filesystem instance.
    ///
    /// # Returns
    ///
    /// A new `RamFileSystem` with an empty root directory.
    pub fn new() -> Self {
        Self {
            parent: Once::new(),
            root: DirNode::new(None),
        }
    }

    /// Returns the root directory node.
    ///
    /// # Returns
    ///
    /// A reference to the root directory of the filesystem.
    pub fn root_dir_node(&self) -> Arc<DirNode> {
        self.root.clone()
    }
}

impl VfsOps for RamFileSystem {
    /// Mount the RAM filesystem at the specified path.
    ///
    /// This method sets up the parent reference for the root directory.
    ///
    /// # Arguments
    ///
    /// * `_path` - The mount path (not used in RAM filesystem)
    /// * `mount_point` - The mount point directory node
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success.
    fn mount(&self, _path: &str, mount_point: VfsNodeRef) -> VfsResult {
        if let Some(parent) = mount_point.parent() {
            self.root.set_parent(Some(self.parent.call_once(|| parent)));
        } else {
            self.root.set_parent(None);
        }
        Ok(())
    }

    /// Returns the root directory of the RAM filesystem.
    ///
    /// # Returns
    ///
    /// A reference to the root directory node.
    fn root_dir(&self) -> VfsNodeRef {
        self.root.clone()
    }
}

impl Default for RamFileSystem {
    /// Creates a default RAM filesystem instance.
    ///
    /// This is equivalent to calling [`RamFileSystem::new()`].
    fn default() -> Self {
        Self::new()
    }
}
