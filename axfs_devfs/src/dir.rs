use alloc::collections::BTreeMap;
use alloc::sync::{Arc, Weak};
use axfs_vfs::{VfsDirEntry, VfsNodeAttr, VfsNodeOps, VfsNodeRef, VfsNodeType};
use axfs_vfs::{VfsError, VfsResult};
use spin::RwLock;

#[cfg(test)]
use crate::NullDev;

/// The directory node in device filesystem.
///
/// This represents a directory that can contain device nodes.
/// It implements VFS node operations trait to provide directory operations.
///
/// Device filesystem directories are read-only at runtime - devices
/// must be registered at filesystem creation time.
///
/// # Fields
///
/// - `parent` - Weak reference to parent directory
/// - `children` - Map of child node names to their references
pub struct DirNode {
    parent: RwLock<Weak<dyn VfsNodeOps>>,
    children: RwLock<BTreeMap<&'static str, VfsNodeRef>>,
}

impl DirNode {
    /// Creates a new directory node.
    ///
    /// # Arguments
    ///
    /// * `parent` - Optional reference to parent directory
    ///
    /// # Returns
    ///
    /// A new directory node wrapped in an Arc.
    pub(super) fn new(parent: Option<&VfsNodeRef>) -> Arc<Self> {
        let parent = parent.map_or(Weak::<Self>::new() as _, Arc::downgrade);
        Arc::new(Self {
            parent: RwLock::new(parent),
            children: RwLock::new(BTreeMap::new()),
        })
    }

    /// Sets the parent directory for this directory.
    ///
    /// # Arguments
    ///
    /// * `parent` - Optional reference to parent directory node
    pub(super) fn set_parent(&self, parent: Option<&VfsNodeRef>) {
        *self.parent.write() = parent.map_or(Weak::<Self>::new() as _, Arc::downgrade);
    }

    /// Creates a subdirectory at this directory.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the subdirectory to create
    ///
    /// # Returns
    ///
    /// A reference to the created directory node.
    pub fn mkdir(self: &Arc<Self>, name: &'static str) -> Arc<Self> {
        let parent = self.clone() as VfsNodeRef;
        let node = Self::new(Some(&parent));
        self.children.write().insert(name, node.clone());
        node
    }

    /// Adds a device node to this directory.
    ///
    /// This is the primary method for registering devices in the filesystem.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the device node
    /// * `node` - The device node reference to add
    pub fn add(&self, name: &'static str, node: VfsNodeRef) {
        self.children.write().insert(name, node);
    }
}

impl VfsNodeOps for DirNode {
    /// Returns the attributes of this directory.
    ///
    /// # Returns
    ///
    /// Returns directory attributes with a fixed size of 4096 bytes.
    fn get_attr(&self) -> VfsResult<VfsNodeAttr> {
        Ok(VfsNodeAttr::new_dir(4096, 0))
    }

    /// Returns the parent directory of this directory.
    ///
    /// # Returns
    ///
    /// Returns `Some(VfsNodeRef)` if a parent exists, `None` otherwise.
    fn parent(&self) -> Option<VfsNodeRef> {
        self.parent.read().upgrade()
    }

    /// Lookups a device node with the given path.
    ///
    /// This method supports path components including `.` and `..`.
    ///
    /// # Arguments
    ///
    /// * `path` - The relative path to lookup
    ///
    /// # Returns
    ///
    /// Returns a reference to the found device node, or an error if not found.
    fn lookup(self: Arc<Self>, path: &str) -> VfsResult<VfsNodeRef> {
        let (name, rest) = split_path(path);
        let node = match name {
            "" | "." => Ok(self.clone() as VfsNodeRef),
            ".." => self.parent().ok_or(VfsError::NotFound),
            _ => self
                .children
                .read()
                .get(name)
                .cloned()
                .ok_or(VfsError::NotFound),
        }?;

        if let Some(rest) = rest {
            node.lookup(rest)
        } else {
            Ok(node)
        }
    }

    /// Reads directory entries into the provided buffer.
    ///
    /// The first two entries are always `.` and `..`, followed by
    /// the actual device nodes.
    ///
    /// # Arguments
    ///
    /// * `start_idx` - The starting index for reading entries
    /// * `dirents` - A mutable slice to store the directory entries
    ///
    /// # Returns
    ///
    /// Returns the number of entries read on success.
    fn read_dir(&self, start_idx: usize, dirents: &mut [VfsDirEntry]) -> VfsResult<usize> {
        let children = self.children.read();
        let mut children = children.iter().skip(start_idx.max(2) - 2);
        for (i, ent) in dirents.iter_mut().enumerate() {
            match i + start_idx {
                0 => *ent = VfsDirEntry::new(".", VfsNodeType::Dir),
                1 => *ent = VfsDirEntry::new("..", VfsNodeType::Dir),
                _ => {
                    if let Some((name, node)) = children.next() {
                        *ent = VfsDirEntry::new(name, node.get_attr().unwrap().file_type());
                    } else {
                        return Ok(i);
                    }
                }
            }
        }
        Ok(dirents.len())
    }

    /// Creates a new node (not supported).
    ///
    /// This method is not supported in device filesystem as devices
    /// must be registered at filesystem creation time.
    ///
    /// # Arguments
    ///
    /// * `path` - The path where node would be created
    /// * `ty` - The type of node to create
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the node already exists, or an error otherwise.
    ///
    /// # Errors
    ///
    /// Returns [`VfsError::PermissionDenied`] as dynamic creation is not supported.
    fn create(&self, path: &str, ty: VfsNodeType) -> VfsResult {
        log::debug!("create {ty:?} at devfs: {path}");
        let (name, rest) = split_path(path);
        if let Some(rest) = rest {
            match name {
                "" | "." => self.create(rest, ty),
                ".." => self.parent().ok_or(VfsError::NotFound)?.create(rest, ty),
                _ => self
                    .children
                    .read()
                    .get(name)
                    .ok_or(VfsError::NotFound)?
                    .create(rest, ty),
            }
        } else if name.is_empty() || name == "." || name == ".." {
            Ok(()) // already exists
        } else {
            Err(VfsError::PermissionDenied) // do not support to create nodes dynamically
        }
    }

    /// Removes a node at the given path (not supported).
    ///
    /// This method is not supported in device filesystem as devices
    /// cannot be removed at runtime.
    ///
    /// # Arguments
    ///
    /// * `path` - The path of the node to remove
    ///
    /// # Returns
    ///
    /// Returns an error as dynamic removal is not supported.
    ///
    /// # Errors
    ///
    /// Returns [`VfsError::PermissionDenied`] as dynamic removal is not supported.
    fn remove(&self, path: &str) -> VfsResult {
        log::debug!("remove at devfs: {path}");
        let (name, rest) = split_path(path);
        if let Some(rest) = rest {
            match name {
                "" | "." => self.remove(rest),
                ".." => self.parent().ok_or(VfsError::NotFound)?.remove(rest),
                _ => self
                    .children
                    .read()
                    .get(name)
                    .ok_or(VfsError::NotFound)?
                    .remove(rest),
            }
        } else {
            Err(VfsError::PermissionDenied) // do not support to remove nodes dynamically
        }
    }

    axfs_vfs::impl_vfs_dir_default! {}
}

/// Splits a path into first component and the rest.
///
/// This helper function is used for path resolution.
///
/// # Arguments
///
/// * `path` - The path to split
///
/// # Returns
///
/// A tuple containing the first path component and an optional remainder.
fn split_path(path: &str) -> (&str, Option<&str>) {
    let trimmed_path = path.trim_start_matches('/');
    trimmed_path.find('/').map_or((trimmed_path, None), |n| {
        (&trimmed_path[..n], Some(&trimmed_path[n + 1..]))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::NullDev;

    #[test]
    fn test_split_path() {
        assert_eq!(split_path("foo/bar"), ("foo", Some("bar")));
        assert_eq!(split_path("foo"), ("foo", None));
        assert_eq!(split_path("/foo/bar"), ("foo", Some("bar")));
        assert_eq!(split_path(""), ("", None));
        assert_eq!(split_path("/"), ("", None));
        assert_eq!(split_path("///"), ("", None));
    }

    #[test]
    fn test_dir_node_new() {
        let dir = DirNode::new(None);
        assert!(dir.parent().is_none());
    }

    #[test]
    fn test_dir_node_mkdir() {
        let dir = DirNode::new(None);
        let subdir = dir.mkdir("subdir");
        assert!(subdir.parent().is_some());
        
        // Check if subdir was added
        let entries = dir.children.read();
        assert!(entries.contains_key("subdir"));
    }

    #[test]
    fn test_dir_node_add() {
        let dir = DirNode::new(None);
        let null_device: VfsNodeRef = Arc::new(NullDev);
        dir.add("null", null_device);
        
        // Check if device was added
        let entries = dir.children.read();
        assert!(entries.contains_key("null"));
    }

    #[test]
    fn test_dir_node_get_attr() {
        let dir = DirNode::new(None);
        let attr = dir.get_attr().unwrap();
        assert!(attr.is_dir());
        assert_eq!(attr.size(), 4096);
    }

    #[test]
    fn test_dir_node_lookup_current() {
        let dir = DirNode::new(None);
        let current = dir.clone().lookup(".").unwrap();
        assert!(current.get_attr().unwrap().is_dir());
    }

    #[test]
    fn test_dir_node_lookup_device() {
        let dir = DirNode::new(None);
        let null_device: VfsNodeRef = Arc::new(NullDev);
        dir.add("null", null_device);
        
        let device = dir.lookup("null").unwrap();
        assert_eq!(device.get_attr().unwrap().file_type(), VfsNodeType::CharDevice);
    }

    #[test]
    fn test_dir_node_lookup_not_found() {
        let dir = DirNode::new(None);
        assert_eq!(dir.lookup("nonexistent").err(), Some(VfsError::NotFound));
    }

    #[test]
    fn test_dir_node_lookup_subdirectory() {
        let dir = DirNode::new(None);
        let subdir = dir.mkdir("subdir");
        let null_device: VfsNodeRef = Arc::new(NullDev);
        subdir.add("null", null_device);
        
        let device = dir.lookup("subdir/null").unwrap();
        assert_eq!(device.get_attr().unwrap().file_type(), VfsNodeType::CharDevice);
    }

    #[test]
    fn test_dir_node_read_dir_empty() {
        let dir = DirNode::new(None);
        let mut entries: Vec<VfsDirEntry> = (0..10).map(|_| VfsDirEntry::default()).collect();
        let count = dir.read_dir(0, &mut entries).unwrap();
        assert_eq!(count, 2); // . and ..
        
        assert_eq!(entries[0].name_as_bytes(), b".");
        assert_eq!(entries[1].name_as_bytes(), b"..");
    }

    #[test]
    fn test_dir_node_read_dir_with_devices() {
        let dir = DirNode::new(None);
        let null_device: VfsNodeRef = Arc::new(NullDev);
        dir.add("null", null_device);
        
        let mut entries: Vec<VfsDirEntry> = (0..10).map(|_| VfsDirEntry::default()).collect();
        let count = dir.read_dir(0, &mut entries).unwrap();
        assert_eq!(count, 3); // ., .., null
        
        assert_eq!(entries[0].name_as_bytes(), b".");
        assert_eq!(entries[1].name_as_bytes(), b"..");
        assert_eq!(entries[2].name_as_bytes(), b"null");
    }

    #[test]
    fn test_dir_node_create_not_supported() {
        let dir = DirNode::new(None);
        assert_eq!(
            dir.create("newfile", VfsNodeType::File).err(),
            Some(VfsError::PermissionDenied)
        );
    }

    #[test]
    fn test_dir_node_create_already_exists() {
        let dir = DirNode::new(None);
        let null_device: VfsNodeRef = Arc::new(NullDev);
        dir.add("null", null_device);
        
        // Creating an existing node should return PermissionDenied
        assert_eq!(
            dir.create("null", VfsNodeType::File).err(),
            Some(VfsError::PermissionDenied)
        );
    }

    #[test]
    fn test_dir_node_remove_not_supported() {
        let dir = DirNode::new(None);
        assert_eq!(
            dir.remove("null").err(),
            Some(VfsError::PermissionDenied)
        );
    }
}
