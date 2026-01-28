use alloc::collections::BTreeMap;
use alloc::sync::{Arc, Weak};
use alloc::{string::String, vec::Vec};

use axfs_vfs::{VfsDirEntry, VfsNodeAttr, VfsNodeOps, VfsNodeRef, VfsNodeType};
use axfs_vfs::{VfsError, VfsResult};
use spin::RwLock;

use crate::file::FileNode;

/// The directory node in RAM filesystem.
///
/// This represents a directory that can contain files and subdirectories.
/// It implements the VFS node operations trait to provide directory operations.
///
/// # Fields
///
/// - `this` - Weak reference to self for creating child directories
/// - `parent` - Weak reference to parent directory
/// - `children` - Map of child node names to their references
pub struct DirNode {
    this: Weak<DirNode>,
    parent: RwLock<Weak<dyn VfsNodeOps>>,
    children: RwLock<BTreeMap<String, VfsNodeRef>>,
}

impl DirNode {
    /// Creates a new directory node.
    ///
    /// # Arguments
    ///
    /// * `parent` - Optional weak reference to parent directory
    ///
    /// # Returns
    ///
    /// A new directory node wrapped in an Arc.
    pub(super) fn new(parent: Option<Weak<dyn VfsNodeOps>>) -> Arc<Self> {
        Arc::new_cyclic(|this| Self {
            this: this.clone(),
            parent: RwLock::new(parent.unwrap_or_else(|| Weak::<Self>::new())),
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

    /// Returns a list of all entry names in this directory.
    ///
    /// # Returns
    ///
    /// A vector containing the names of all child nodes.
    pub fn get_entries(&self) -> Vec<String> {
        self.children.read().keys().cloned().collect()
    }

    /// Checks whether a node with the given name exists in this directory.
    ///
    /// # Arguments
    ///
    /// * `name` - The name to check for
    ///
    /// # Returns
    ///
    /// `true` if a node with the given name exists, `false` otherwise.
    pub fn exist(&self, name: &str) -> bool {
        self.children.read().contains_key(name)
    }

    /// Creates a new node with the given name and type in this directory.
    ///
    /// # Arguments
    ///
    /// * `name` - The name for the new node
    /// * `ty` - The type of node to create (file or directory)
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the node was created, or an error if creation failed.
    ///
    /// # Errors
    ///
    /// Returns [`VfsError::AlreadyExists`] if a node with the same name exists.
    /// Returns [`VfsError::Unsupported`] if the node type is not supported.
    pub fn create_node(&self, name: &str, ty: VfsNodeType) -> VfsResult {
        if self.exist(name) {
            log::error!("AlreadyExists {name}");
            return Err(VfsError::AlreadyExists);
        }
        let node: VfsNodeRef = match ty {
            VfsNodeType::File => Arc::new(FileNode::new()),
            VfsNodeType::Dir => Self::new(Some(self.this.clone())),
            _ => return Err(VfsError::Unsupported),
        };
        self.children.write().insert(name.into(), node);
        Ok(())
    }

    /// Removes a node by the given name in this directory.
    ///
    /// Directories can only be removed if they are empty.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the node to remove
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the node was removed, or an error otherwise.
    ///
    /// # Errors
    ///
    /// Returns [`VfsError::NotFound`] if the node does not exist.
    /// Returns [`VfsError::DirectoryNotEmpty`] if attempting to remove a non-empty directory.
    pub fn remove_node(&self, name: &str) -> VfsResult {
        let mut children = self.children.write();
        let node = children.get(name).ok_or(VfsError::NotFound)?;
        if let Some(dir) = node.as_any().downcast_ref::<DirNode>() {
            if !dir.children.read().is_empty() {
                return Err(VfsError::DirectoryNotEmpty);
            }
        }
        children.remove(name);
        Ok(())
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

    /// Lookups a node with the given path relative to this directory.
    ///
    /// This method supports path components including `.` (current directory)
    /// and `..` (parent directory).
    ///
    /// # Arguments
    ///
    /// * `path` - The relative path to lookup
    ///
    /// # Returns
    ///
    /// Returns a reference to the found node, or an error if not found.
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
    /// The first two entries are always `.` (current directory) and
    /// `..` (parent directory), followed by the actual child nodes.
    ///
    /// # Arguments
    ///
    /// * `start_idx` - The starting index for reading entries (for pagination)
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

    /// Creates a new node with the given path and type.
    ///
    /// This method recursively creates directories if needed.
    ///
    /// # Arguments
    ///
    /// * `path` - The path where the node should be created
    /// * `ty` - The type of node to create
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if creation succeeds, or an error otherwise.
    fn create(&self, path: &str, ty: VfsNodeType) -> VfsResult {
        log::debug!("create {ty:?} at ramfs: {path}");
        let (name, rest) = split_path(path);
        if let Some(rest) = rest {
            match name {
                "" | "." => self.create(rest, ty),
                ".." => self.parent().ok_or(VfsError::NotFound)?.create(rest, ty),
                _ => {
                    let subdir = self
                        .children
                        .read()
                        .get(name)
                        .ok_or(VfsError::NotFound)?
                        .clone();
                    subdir.create(rest, ty)
                }
            }
        } else if name.is_empty() || name == "." || name == ".." {
            Ok(()) // already exists
        } else {
            self.create_node(name, ty)
        }
    }

    /// Removes a node at the given path.
    ///
    /// This method recursively removes nodes along the path.
    /// Directories must be empty to be removed.
    ///
    /// # Arguments
    ///
    /// * `path` - The path of the node to remove
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if removal succeeds, or an error otherwise.
    fn remove(&self, path: &str) -> VfsResult {
        log::debug!("remove at ramfs: {path}");
        let (name, rest) = split_path(path);
        if let Some(rest) = rest {
            match name {
                "" | "." => self.remove(rest),
                ".." => self.parent().ok_or(VfsError::NotFound)?.remove(rest),
                _ => {
                    let subdir = self
                        .children
                        .read()
                        .get(name)
                        .ok_or(VfsError::NotFound)?
                        .clone();
                    subdir.remove(rest)
                }
            }
        } else if name.is_empty() || name == "." || name == ".." {
            Err(VfsError::InvalidInput) // remove '.' or '..
        } else {
            self.remove_node(name)
        }
    }

    axfs_vfs::impl_vfs_dir_default! {}
}

/// Splits a path into the first component and the rest.
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

    #[test]
    fn test_split_path() {
        assert_eq!(split_path("foo/bar"), ("foo", Some("bar")));
        assert_eq!(split_path("foo"), ("foo", None));
        assert_eq!(split_path("/foo/bar"), ("foo", Some("bar")));
        assert_eq!(split_path("///foo/bar"), ("foo", Some("bar")));
        assert_eq!(split_path(""), ("", None));
        assert_eq!(split_path("/"), ("", None));
        assert_eq!(split_path("///"), ("", None));
    }

    #[test]
    fn test_dir_node_new() {
        let dir = DirNode::new(None);
        assert!(dir.get_entries().is_empty());
        assert!(!dir.exist("test"));
    }

    #[test]
    fn test_dir_node_exist() {
        let dir = DirNode::new(None);
        assert!(!dir.exist("test"));
        assert!(!dir.exist("foo"));
    }

    #[test]
    fn test_dir_node_create_file() {
        let dir = DirNode::new(None);
        assert!(dir.create_node("test.txt", VfsNodeType::File).is_ok());
        assert!(dir.exist("test.txt"));
    }

    #[test]
    fn test_dir_node_create_dir() {
        let dir = DirNode::new(None);
        assert!(dir.create_node("testdir", VfsNodeType::Dir).is_ok());
        assert!(dir.exist("testdir"));
    }

    #[test]
    fn test_dir_node_create_duplicate() {
        let dir = DirNode::new(None);
        assert!(dir.create_node("test", VfsNodeType::File).is_ok());
        assert_eq!(
            dir.create_node("test", VfsNodeType::File).err(),
            Some(VfsError::AlreadyExists)
        );
    }

    #[test]
    fn test_dir_node_create_unsupported() {
        let dir = DirNode::new(None);
        assert_eq!(
            dir.create_node("test", VfsNodeType::SymLink).err(),
            Some(VfsError::Unsupported)
        );
    }

    #[test]
    fn test_dir_node_remove_file() {
        let dir = DirNode::new(None);
        assert!(dir.create_node("test.txt", VfsNodeType::File).is_ok());
        assert!(dir.remove_node("test.txt").is_ok());
        assert!(!dir.exist("test.txt"));
    }

    #[test]
    fn test_dir_node_remove_empty_dir() {
        let dir = DirNode::new(None);
        assert!(dir.create_node("testdir", VfsNodeType::Dir).is_ok());
        assert!(dir.remove_node("testdir").is_ok());
        assert!(!dir.exist("testdir"));
    }

    #[test]
    fn test_dir_node_remove_not_empty_dir() {
        let dir = DirNode::new(None);
        assert!(dir.create_node("testdir", VfsNodeType::Dir).is_ok());
        let subdir = dir.clone().lookup("testdir").unwrap();
        assert!(subdir.create("nested.txt", VfsNodeType::File).is_ok());
        assert_eq!(
            dir.remove_node("testdir").err(),
            Some(VfsError::DirectoryNotEmpty)
        );
    }

    #[test]
    fn test_dir_node_remove_not_found() {
        let dir = DirNode::new(None);
        assert_eq!(dir.remove_node("nonexistent").err(), Some(VfsError::NotFound));
    }

    #[test]
    fn test_dir_node_get_entries() {
        let dir = DirNode::new(None);
        assert!(dir.create_node("f1", VfsNodeType::File).is_ok());
        assert!(dir.create_node("f2", VfsNodeType::File).is_ok());
        assert!(dir.create_node("d1", VfsNodeType::Dir).is_ok());
        
        let entries = dir.get_entries();
        assert_eq!(entries.len(), 3);
        assert!(entries.contains(&"f1".to_string()));
        assert!(entries.contains(&"f2".to_string()));
        assert!(entries.contains(&"d1".to_string()));
    }

    #[test]
    fn test_dir_node_lookup_current() {
        let dir = DirNode::new(None);
        let current = dir.clone().lookup(".").unwrap();
        assert!(current.get_attr().unwrap().is_dir());
    }

    #[test]
    fn test_dir_node_lookup_file() {
        let dir = DirNode::new(None);
        assert!(dir.create_node("test.txt", VfsNodeType::File).is_ok());
        let file = dir.lookup("test.txt").unwrap();
        assert!(file.get_attr().unwrap().is_file());
    }

    #[test]
    fn test_dir_node_lookup_not_found() {
        let dir = DirNode::new(None);
        assert_eq!(dir.lookup("nonexistent").err(), Some(VfsError::NotFound));
    }

    #[test]
    fn test_dir_node_parent_none() {
        let dir = DirNode::new(None);
        assert!(dir.parent().is_none());
    }

    #[test]
    fn test_dir_node_get_attr() {
        let dir = DirNode::new(None);
        let attr = dir.get_attr().unwrap();
        assert!(attr.is_dir());
        assert_eq!(attr.size(), 4096);
    }

    #[test]
    fn test_dir_node_create_with_path() {
        let dir = DirNode::new(None);
        // Create intermediate directory first
        assert!(dir.create("subdir", VfsNodeType::Dir).is_ok());
        assert!(dir.create("subdir/nested", VfsNodeType::Dir).is_ok());
        assert!(dir.create("subdir/nested/file.txt", VfsNodeType::File).is_ok());
        assert!(dir.exist("subdir"));
    }

    #[test]
    fn test_dir_node_read_dir() {
        let dir = DirNode::new(None);
        assert!(dir.create_node("f1", VfsNodeType::File).is_ok());
        assert!(dir.create_node("f2", VfsNodeType::Dir).is_ok());
        
        let mut entries: Vec<VfsDirEntry> = (0..10).map(|_| VfsDirEntry::default()).collect();
        let count = dir.read_dir(0, &mut entries).unwrap();
        assert!(count >= 3); // ., .., f1, f2
        
        // First entry should be "."
        assert_eq!(entries[0].name_as_bytes(), b".");
        assert_eq!(entries[0].entry_type(), VfsNodeType::Dir);
        
        // Second entry should be ".."
        assert_eq!(entries[1].name_as_bytes(), b"..");
        assert_eq!(entries[1].entry_type(), VfsNodeType::Dir);
    }
}
