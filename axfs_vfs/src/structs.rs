/// Filesystem attributes.
///
/// This structure contains information about the filesystem, such as
/// total size, available space, block size, etc.
///
/// # Note
///
/// Currently this struct is not used and is reserved for future use.
#[non_exhaustive]
pub struct FileSystemInfo;

/// Node (file/directory) attributes.
///
/// This structure contains metadata about a VFS node, including its
/// permissions, type, size, and the number of blocks allocated.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct VfsNodeAttr {
    /// File permission mode.
    mode: VfsNodePerm,
    /// File type.
    ty: VfsNodeType,
    /// Total size, in bytes.
    size: u64,
    /// Number of 512B blocks allocated.
    blocks: u64,
}

bitflags::bitflags! {
    /// Node (file/directory) permission mode.
    ///
    /// This bitflag represents the Unix-style permission bits for a file or directory.
    /// Permissions are divided into three categories: owner, group, and others.
    /// Each category has read, write, and execute permissions.
    #[derive(Debug, Clone, Copy)]
    pub struct VfsNodePerm: u16 {
        /// Owner has read permission.
        const OWNER_READ = 0o400;
        /// Owner has write permission.
        const OWNER_WRITE = 0o200;
        /// Owner has execute permission.
        const OWNER_EXEC = 0o100;

        /// Group has read permission.
        const GROUP_READ = 0o40;
        /// Group has write permission.
        const GROUP_WRITE = 0o20;
        /// Group has execute permission.
        const GROUP_EXEC = 0o10;

        /// Others have read permission.
        const OTHER_READ = 0o4;
        /// Others have write permission.
        const OTHER_WRITE = 0o2;
        /// Others have execute permission.
        const OTHER_EXEC = 0o1;
    }
}

/// Node (file/directory) type.
///
/// This enumeration represents the type of a VFS node. It includes standard
/// Unix file types such as regular files, directories, symbolic links, devices,
/// and more.
#[repr(u8)]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum VfsNodeType {
    /// FIFO (named pipe)
    Fifo = 0o1,
    /// Character device
    CharDevice = 0o2,
    /// Directory
    Dir = 0o4,
    /// Block device
    BlockDevice = 0o6,
    /// Regular file
    File = 0o10,
    /// Symbolic link
    SymLink = 0o12,
    /// Socket
    Socket = 0o14,
}

/// Directory entry.
///
/// This structure represents a single entry in a directory, containing
/// the entry's name and type. The name is limited to 63 bytes.
pub struct VfsDirEntry {
    d_type: VfsNodeType,
    d_name: [u8; 63],
}

impl VfsNodePerm {
    /// Returns the default permission for a file.
    ///
    /// The default permission is `0o666` (rw-rw-rw-), which means
    /// owner, group, and others can all read and write.
    ///
    /// # Returns
    ///
    /// A `VfsNodePerm` with the default file permission bits set.
    pub const fn default_file() -> Self {
        Self::from_bits_truncate(0o666)
    }

    /// Returns the default permission for a directory.
    ///
    /// The default permission is `0o755` (rwxr-xr-x), which means
    /// owner can read, write, and execute, while group and others can
    /// read and execute only.
    ///
    /// # Returns
    ///
    /// A `VfsNodePerm` with the default directory permission bits set.
    pub const fn default_dir() -> Self {
        Self::from_bits_truncate(0o755)
    }

    /// Returns the underlying raw `st_mode` bits that contain the standard
    /// Unix permissions for this file.
    ///
    /// This returns a 32-bit integer containing the permission bits,
    /// compatible with Unix `st_mode` format.
    ///
    /// # Returns
    ///
    /// The raw permission bits as a `u32`.
    pub const fn mode(&self) -> u32 {
        self.bits() as u32
    }

    /// Returns a 9-bytes string representation of the permission.
    ///
    /// The representation follows the standard Unix `ls -l` format:
    /// - First three characters: owner permissions (r/w/x)
    /// - Next three characters: group permissions (r/w/x)
    /// - Last three characters: other permissions (r/w/x)
    ///
    /// Each permission is represented as the character `r`, `w`, `x`
    /// if granted, or `-` if not granted.
    ///
    /// # Returns
    ///
    /// A 9-byte array containing the string representation.
    pub const fn rwx_buf(&self) -> [u8; 9] {
        let mut perm = [b'-'; 9];
        if self.contains(Self::OWNER_READ) {
            perm[0] = b'r';
        }
        if self.contains(Self::OWNER_WRITE) {
            perm[1] = b'w';
        }
        if self.contains(Self::OWNER_EXEC) {
            perm[2] = b'x';
        }
        if self.contains(Self::GROUP_READ) {
            perm[3] = b'r';
        }
        if self.contains(Self::GROUP_WRITE) {
            perm[4] = b'w';
        }
        if self.contains(Self::GROUP_EXEC) {
            perm[5] = b'x';
        }
        if self.contains(Self::OTHER_READ) {
            perm[6] = b'r';
        }
        if self.contains(Self::OTHER_WRITE) {
            perm[7] = b'w';
        }
        if self.contains(Self::OTHER_EXEC) {
            perm[8] = b'x';
        }
        perm
    }

    /// Whether the owner has read permission.
    ///
    /// # Returns
    ///
    /// `true` if the owner has read permission, `false` otherwise.
    pub const fn owner_readable(&self) -> bool {
        self.contains(Self::OWNER_READ)
    }

    /// Whether the owner has write permission.
    ///
    /// # Returns
    ///
    /// `true` if the owner has write permission, `false` otherwise.
    pub const fn owner_writable(&self) -> bool {
        self.contains(Self::OWNER_WRITE)
    }

    /// Whether the owner has execute permission.
    ///
    /// # Returns
    ///
    /// `true` if the owner has execute permission, `false` otherwise.
    pub const fn owner_executable(&self) -> bool {
        self.contains(Self::OWNER_EXEC)
    }
}

impl VfsNodeType {
    /// Tests whether this node type represents a regular file.
    ///
    /// # Returns
    ///
    /// `true` if the node type is [`File`](Self::File), `false` otherwise.
    pub const fn is_file(self) -> bool {
        matches!(self, Self::File)
    }

    /// Tests whether this node type represents a directory.
    ///
    /// # Returns
    ///
    /// `true` if the node type is [`Dir`](Self::Dir), `false` otherwise.
    pub const fn is_dir(self) -> bool {
        matches!(self, Self::Dir)
    }

    /// Tests whether this node type represents a symbolic link.
    ///
    /// # Returns
    ///
    /// `true` if the node type is [`SymLink`](Self::SymLink), `false` otherwise.
    pub const fn is_symlink(self) -> bool {
        matches!(self, Self::SymLink)
    }

    /// Returns `true` if this node type is a block device.
    ///
    /// # Returns
    ///
    /// `true` if the node type is [`BlockDevice`](Self::BlockDevice), `false` otherwise.
    pub const fn is_block_device(self) -> bool {
        matches!(self, Self::BlockDevice)
    }

    /// Returns `true` if this node type is a char device.
    ///
    /// # Returns
    ///
    /// `true` if the node type is [`CharDevice`](Self::CharDevice), `false` otherwise.
    pub const fn is_char_device(self) -> bool {
        matches!(self, Self::CharDevice)
    }

    /// Returns `true` if this node type is a fifo.
    ///
    /// # Returns
    ///
    /// `true` if the node type is [`Fifo`](Self::Fifo), `false` otherwise.
    pub const fn is_fifo(self) -> bool {
        matches!(self, Self::Fifo)
    }

    /// Returns `true` if this node type is a socket.
    ///
    /// # Returns
    ///
    /// `true` if the node type is [`Socket`](Self::Socket), `false` otherwise.
    pub const fn is_socket(self) -> bool {
        matches!(self, Self::Socket)
    }

    /// Returns a character representation of the node type.
    ///
    /// This follows the standard Unix `ls -l` format:
    /// - `d` for directory
    /// - `-` for regular file
    /// - `l` for symbolic link
    /// - `p` for FIFO (named pipe)
    /// - `c` for character device
    /// - `b` for block device
    /// - `s` for socket
    ///
    /// # Returns
    ///
    /// A single character representing the node type.
    pub const fn as_char(self) -> char {
        match self {
            Self::Fifo => 'p',
            Self::CharDevice => 'c',
            Self::Dir => 'd',
            Self::BlockDevice => 'b',
            Self::File => '-',
            Self::SymLink => 'l',
            Self::Socket => 's',
        }
    }
}

impl VfsNodeAttr {
    /// Creates a new `VfsNodeAttr` with the given permission mode, type, size
    /// and number of blocks.
    ///
    /// # Arguments
    ///
    /// * `mode` - The permission mode for the node
    /// * `ty` - The type of the node (file, directory, etc.)
    /// * `size` - The size of the node in bytes
    /// * `blocks` - The number of 512-byte blocks allocated
    ///
    /// # Returns
    ///
    /// A new `VfsNodeAttr` instance.
    pub const fn new(mode: VfsNodePerm, ty: VfsNodeType, size: u64, blocks: u64) -> Self {
        Self {
            mode,
            ty,
            size,
            blocks,
        }
    }

    /// Creates a new `VfsNodeAttr` for a file, with the default file permission.
    ///
    /// This is a convenience constructor that uses `VfsNodePerm::default_file()`
    /// and `VfsNodeType::File`.
    ///
    /// # Arguments
    ///
    /// * `size` - The size of the file in bytes
    /// * `blocks` - The number of 512-byte blocks allocated
    ///
    /// # Returns
    ///
    /// A new `VfsNodeAttr` instance for a file.
    pub const fn new_file(size: u64, blocks: u64) -> Self {
        Self {
            mode: VfsNodePerm::default_file(),
            ty: VfsNodeType::File,
            size,
            blocks,
        }
    }

    /// Creates a new `VfsNodeAttr` for a directory, with the default directory
    /// permission.
    ///
    /// This is a convenience constructor that uses `VfsNodePerm::default_dir()`
    /// and `VfsNodeType::Dir`.
    ///
    /// # Arguments
    ///
    /// * `size` - The size of the directory in bytes
    /// * `blocks` - The number of 512-byte blocks allocated
    ///
    /// # Returns
    ///
    /// A new `VfsNodeAttr` instance for a directory.
    pub const fn new_dir(size: u64, blocks: u64) -> Self {
        Self {
            mode: VfsNodePerm::default_dir(),
            ty: VfsNodeType::Dir,
            size,
            blocks,
        }
    }

    /// Returns the size of the node.
    ///
    /// # Returns
    ///
    /// The size in bytes.
    pub const fn size(&self) -> u64 {
        self.size
    }

    /// Returns the number of blocks the node occupies on the disk.
    ///
    /// Each block is 512 bytes.
    ///
    /// # Returns
    ///
    /// The number of 512-byte blocks allocated.
    pub const fn blocks(&self) -> u64 {
        self.blocks
    }

    /// Returns the permission of the node.
    ///
    /// # Returns
    ///
    /// A `VfsNodePerm` representing the permission bits.
    pub const fn perm(&self) -> VfsNodePerm {
        self.mode
    }

    /// Sets the permission of the node.
    ///
    /// # Arguments
    ///
    /// * `perm` - The new permission mode to set
    pub fn set_perm(&mut self, perm: VfsNodePerm) {
        self.mode = perm
    }

    /// Returns the type of the node.
    ///
    /// # Returns
    ///
    /// A `VfsNodeType` representing the node type.
    pub const fn file_type(&self) -> VfsNodeType {
        self.ty
    }

    /// Whether the node is a file.
    ///
    /// # Returns
    ///
    /// `true` if the node is a regular file, `false` otherwise.
    pub const fn is_file(&self) -> bool {
        self.ty.is_file()
    }

    /// Whether the node is a directory.
    ///
    /// # Returns
    ///
    /// `true` if the node is a directory, `false` otherwise.
    pub const fn is_dir(&self) -> bool {
        self.ty.is_dir()
    }
}

impl VfsDirEntry {
    /// Creates an empty `VfsDirEntry`.
    ///
    /// The default entry has type `VfsNodeType::File` and an empty name.
    ///
    /// # Returns
    ///
    /// A new `VfsDirEntry` with default values.
    pub const fn default() -> Self {
        Self {
            d_type: VfsNodeType::File,
            d_name: [0; 63],
        }
    }

    /// Creates a new `VfsDirEntry` with the given name and type.
    ///
    /// The name is truncated to 63 bytes if it exceeds that length.
    /// A warning is logged if truncation occurs.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the entry
    /// * `ty` - The type of the entry (file, directory, etc.)
    ///
    /// # Returns
    ///
    /// A new `VfsDirEntry` instance.
    pub fn new(name: &str, ty: VfsNodeType) -> Self {
        let mut d_name = [0; 63];
        if name.len() > d_name.len() {
            log::warn!(
                "directory entry name too long: {} > {}",
                name.len(),
                d_name.len()
            );
        }
        d_name[..name.len()].copy_from_slice(name.as_bytes());
        Self { d_type: ty, d_name }
    }

    /// Returns the type of the entry.
    ///
    /// # Returns
    ///
    /// A `VfsNodeType` indicating the type of the entry.
    pub fn entry_type(&self) -> VfsNodeType {
        self.d_type
    }

    /// Converts the name of the entry to a byte slice.
    ///
    /// The returned slice contains only the name up to the first null terminator.
    ///
    /// # Returns
    ///
    /// A byte slice containing the entry name.
    pub fn name_as_bytes(&self) -> &[u8] {
        let len = self
            .d_name
            .iter()
            .position(|&c| c == 0)
            .unwrap_or(self.d_name.len());
        &self.d_name[..len]
    }
}
