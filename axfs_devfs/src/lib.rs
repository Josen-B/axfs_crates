//! Device filesystem used by [ArceOS](https://github.com/arceos-org/arceos).
//!
//! This crate provides a filesystem for managing device nodes, similar to
//! `/dev` in Unix-like systems. It includes special device files such as
//! null, zero, and urandom devices.
//!
//! The implementation is based on [`axfs_vfs`].
//!
//! # Components
//!
//! - [`DeviceFileSystem`] - The main device filesystem structure
//! - [`DirNode`] - Directory node for device organization
//! - [`NullDev`] - Null device (like `/dev/null`)
//! - [`UrandomDev`] - Random number generator device (like `/dev/urandom`)
//! - [`ZeroDev`] - Zero device (like `/dev/zero`)
//!
//! # Features
//!
//! - Static device registration (devices must be added at creation time)
//! - Read-only directory structure (cannot create or remove devices dynamically)
//! - Special device behaviors for null, zero, and random data

#![cfg_attr(not(test), no_std)]

extern crate alloc;

mod dir;
mod null;
mod urandom;
mod zero;

pub use self::dir::DirNode;
pub use self::null::NullDev;
pub use self::urandom::UrandomDev;
pub use self::zero::ZeroDev;

use alloc::sync::Arc;
use axfs_vfs::{VfsNodeRef, VfsOps, VfsResult};
use spin::once::Once;

/// A device filesystem that manages device nodes.
///
/// This filesystem provides access to special device files similar to
/// `/dev` directory in Unix-like systems. Devices must be
/// registered at filesystem creation time.
///
/// # Fields
///
/// - `parent` - The parent filesystem mount point
/// - `root` - The root directory containing device nodes
pub struct DeviceFileSystem {
    parent: Once<VfsNodeRef>,
    root: Arc<DirNode>,
}

impl DeviceFileSystem {
    /// Creates a new device filesystem instance.
    ///
    /// # Returns
    ///
    /// A new device filesystem with an empty root directory.
    pub fn new() -> Self {
        Self {
            parent: Once::new(),
            root: DirNode::new(None),
        }
    }

    /// Creates a subdirectory at the root directory.
    ///
    /// This method creates a new directory node and adds it to the root.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the subdirectory to create
    ///
    /// # Returns
    ///
    /// A reference to the created directory node.
    pub fn mkdir(&self, name: &'static str) -> Arc<DirNode> {
        self.root.mkdir(name)
    }

    /// Adds a device node to the root directory.
    ///
    /// This method registers a device node in the filesystem.
    /// The node must implement the VFS node operations trait.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the device node
    /// * `node` - The device node reference to add
    pub fn add(&self, name: &'static str, node: VfsNodeRef) {
        self.root.add(name, node);
    }
}

impl VfsOps for DeviceFileSystem {
    /// Mounts the device filesystem at the specified path.
    ///
    /// This method sets up the parent reference for the root directory.
    ///
    /// # Arguments
    ///
    /// * `_path` - The mount path (not used in device filesystem)
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

    /// Returns the root directory of the device filesystem.
    ///
    /// # Returns
    ///
    /// A reference to the root directory node.
    fn root_dir(&self) -> VfsNodeRef {
        self.root.clone()
    }
}

impl Default for DeviceFileSystem {
    /// Creates a default device filesystem instance.
    ///
    /// This is equivalent to calling [`DeviceFileSystem::new()`].
    fn default() -> Self {
        Self::new()
    }
}
