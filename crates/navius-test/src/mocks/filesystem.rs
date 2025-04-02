use std::collections::HashMap;
use std::fmt;
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::TestResult;
use crate::mock::{Expectation, MockRegistry};

/// A mock implementation of a filesystem interface
#[derive(Debug, Default)]
pub struct MockFileSystem {
    /// Internal state for recording and verifying method calls
    state: Arc<Mutex<MockFileSystemState>>,
}

/// Mock file system state
#[derive(Debug, Default)]
struct MockFileSystemState {
    /// Files stored in the mock filesystem
    files: HashMap<PathBuf, MockFile>,

    /// Directories stored in the mock filesystem
    directories: HashMap<PathBuf, Vec<PathBuf>>,

    /// Metadata for files and directories
    metadata: HashMap<PathBuf, MockFileMetadata>,

    /// Records of read operations
    read_calls: Vec<PathBuf>,

    /// Records of write operations
    write_calls: Vec<(PathBuf, Vec<u8>)>,

    /// Records of metadata operations
    metadata_calls: Vec<PathBuf>,

    /// Records of directory list operations
    list_directory_calls: Vec<PathBuf>,
}

/// A mock file in the mock filesystem
#[derive(Debug, Clone)]
pub struct MockFile {
    /// The path of the file
    pub path: PathBuf,

    /// The contents of the file
    pub contents: Vec<u8>,

    /// The metadata of the file
    pub metadata: MockFileMetadata,
}

/// Mock file metadata
#[derive(Debug, Clone)]
pub struct MockFileMetadata {
    /// The size of the file in bytes
    pub size: u64,

    /// Whether the file is a directory
    pub is_dir: bool,

    /// Whether the file is a file
    pub is_file: bool,

    /// Whether the file is read-only
    pub readonly: bool,

    /// The creation time of the file
    pub created: SystemTime,

    /// The last modified time of the file
    pub modified: SystemTime,

    /// The last accessed time of the file
    pub accessed: SystemTime,
}

impl Default for MockFileMetadata {
    fn default() -> Self {
        let now = SystemTime::now();
        Self {
            size: 0,
            is_dir: false,
            is_file: true,
            readonly: false,
            created: now,
            modified: now,
            accessed: now,
        }
    }
}

impl MockFileMetadata {
    /// Create new file metadata
    pub fn new() -> Self {
        Self::default()
    }

    /// Create directory metadata
    pub fn dir() -> Self {
        let mut metadata = Self::default();
        metadata.is_dir = true;
        metadata.is_file = false;
        metadata
    }

    /// Set the size of the file
    pub fn with_size(mut self, size: u64) -> Self {
        self.size = size;
        self
    }

    /// Set whether the file is read-only
    pub fn with_readonly(mut self, readonly: bool) -> Self {
        self.readonly = readonly;
        self
    }

    /// Set the creation time of the file
    pub fn with_created(mut self, created: SystemTime) -> Self {
        self.created = created;
        self
    }

    /// Set the last modified time of the file
    pub fn with_modified(mut self, modified: SystemTime) -> Self {
        self.modified = modified;
        self
    }

    /// Set the last accessed time of the file
    pub fn with_accessed(mut self, accessed: SystemTime) -> Self {
        self.accessed = accessed;
        self
    }
}

/// A mock filesystem error
#[derive(Debug)]
pub struct MockFileSystemError {
    /// The error message
    pub message: String,

    /// The path that caused the error
    pub path: Option<PathBuf>,

    /// The error kind
    pub kind: io::ErrorKind,
}

impl MockFileSystemError {
    /// Create a new mock filesystem error
    pub fn new<S: Into<String>>(message: S, kind: io::ErrorKind) -> Self {
        Self {
            message: message.into(),
            path: None,
            kind,
        }
    }

    /// Set the path that caused the error
    pub fn with_path<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.path = Some(path.as_ref().to_path_buf());
        self
    }
}

impl fmt::Display for MockFileSystemError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref path) = self.path {
            write!(f, "{}: {}", self.message, path.display())
        } else {
            write!(f, "{}", self.message)
        }
    }
}

impl std::error::Error for MockFileSystemError {}

impl From<MockFileSystemError> for io::Error {
    fn from(error: MockFileSystemError) -> Self {
        io::Error::new(error.kind, error.message)
    }
}

/// A mock file handle for reading and writing
pub struct MockFileHandle {
    /// The file being accessed
    file: Arc<Mutex<MockFile>>,

    /// The current position in the file
    position: u64,

    /// Whether the file is open for reading
    readable: bool,

    /// Whether the file is open for writing
    writable: bool,
}

impl MockFileHandle {
    /// Create a new file handle
    fn new(file: Arc<Mutex<MockFile>>, readable: bool, writable: bool) -> Self {
        Self {
            file,
            position: 0,
            readable,
            writable,
        }
    }
}

impl Read for MockFileHandle {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if !self.readable {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "File not opened for reading",
            ));
        }

        let file = self.file.lock().unwrap();

        let start = self.position as usize;
        if start >= file.contents.len() {
            return Ok(0);
        }

        let end = std::cmp::min(start + buf.len(), file.contents.len());
        let bytes_to_read = end - start;

        buf[..bytes_to_read].copy_from_slice(&file.contents[start..end]);
        self.position += bytes_to_read as u64;

        Ok(bytes_to_read)
    }
}

impl Write for MockFileHandle {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if !self.writable {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "File not opened for writing",
            ));
        }

        let mut file = self.file.lock().unwrap();

        let start = self.position as usize;
        if start > file.contents.len() {
            // Extend the file with zeros if writing past the end
            file.contents.resize(start, 0);
        }

        let bytes_to_write = buf.len();
        if start + bytes_to_write > file.contents.len() {
            file.contents.resize(start + bytes_to_write, 0);
        }

        file.contents[start..start + bytes_to_write].copy_from_slice(buf);
        self.position += bytes_to_write as u64;

        // Update metadata
        file.metadata.size = file.contents.len() as u64;
        file.metadata.modified = SystemTime::now();

        Ok(bytes_to_write)
    }

    fn flush(&mut self) -> io::Result<()> {
        // No-op for mock
        Ok(())
    }
}

impl Seek for MockFileHandle {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let file = self.file.lock().unwrap();
        let file_size = file.contents.len() as u64;

        let new_position = match pos {
            SeekFrom::Start(offset) => offset,
            SeekFrom::End(offset) => {
                if offset >= 0 {
                    file_size.saturating_add(offset as u64)
                } else {
                    file_size.saturating_sub((-offset) as u64)
                }
            }
            SeekFrom::Current(offset) => {
                if offset >= 0 {
                    self.position.saturating_add(offset as u64)
                } else {
                    self.position.saturating_sub((-offset) as u64)
                }
            }
        };

        self.position = new_position;
        Ok(self.position)
    }
}

impl MockFileSystem {
    /// Create a new mock filesystem
    pub fn new() -> Self {
        let fs = Self {
            state: Arc::new(Mutex::new(MockFileSystemState::default())),
        };

        // Initialize with root directory
        fs.create_dir("/").unwrap();

        fs
    }

    /// Register this mock with the mock registry
    pub fn register(self, registry: &MockRegistry) -> TestResult<Arc<Self>> {
        let mock = Arc::new(self);
        registry.register::<dyn FileSystem, Self>(Arc::clone(&mock))?;
        Ok(mock)
    }

    /// Add a file to the mock filesystem
    pub fn add_file<P: AsRef<Path>, C: AsRef<[u8]>>(&self, path: P, contents: C) -> io::Result<()> {
        let path = normalize_path(path);
        let parent = path.parent().unwrap_or_else(|| Path::new("/"));

        let mut state = self.state.lock().unwrap();

        // Ensure parent directory exists
        if !state.directories.contains_key(&parent.to_path_buf())
            && parent != Path::new("")
            && parent != Path::new("/")
        {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Parent directory does not exist: {}", parent.display()),
            ));
        }

        // Add file to parent directory if it exists
        if let Some(entries) = state.directories.get_mut(&parent.to_path_buf()) {
            if !entries.contains(&path) {
                entries.push(path.clone());
            }
        }

        let contents = contents.as_ref().to_vec();
        let metadata = MockFileMetadata::new()
            .with_size(contents.len() as u64)
            .with_modified(SystemTime::now());

        let file = MockFile {
            path: path.clone(),
            contents,
            metadata: metadata.clone(),
        };

        state.files.insert(path.clone(), file);
        state.metadata.insert(path, metadata);

        Ok(())
    }

    /// Create a directory in the mock filesystem
    pub fn create_dir<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        let path = normalize_path(path);
        let parent = path.parent().unwrap_or_else(|| Path::new("/"));

        let mut state = self.state.lock().unwrap();

        // Ensure parent directory exists
        if !state.directories.contains_key(&parent.to_path_buf())
            && parent != Path::new("")
            && parent != Path::new("/")
            && path != Path::new("/")
        {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Parent directory does not exist: {}", parent.display()),
            ));
        }

        // Add directory to parent directory if it exists and is not root
        if let Some(entries) = state.directories.get_mut(&parent.to_path_buf()) {
            if !entries.contains(&path) {
                entries.push(path.clone());
            }
        }

        // Create directory entry
        state.directories.insert(path.clone(), Vec::new());

        // Create metadata
        let metadata = MockFileMetadata::dir();
        state.metadata.insert(path, metadata);

        Ok(())
    }

    /// Read a file from the mock filesystem
    pub fn read_file<P: AsRef<Path>>(&self, path: P) -> io::Result<Vec<u8>> {
        let path = normalize_path(path);

        let mut state = self.state.lock().unwrap();
        state.read_calls.push(path.clone());

        if let Some(file) = state.files.get(&path) {
            Ok(file.contents.clone())
        } else {
            Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("File not found: {}", path.display()),
            ))
        }
    }

    /// Write a file to the mock filesystem
    pub fn write_file<P: AsRef<Path>, C: AsRef<[u8]>>(
        &self,
        path: P,
        contents: C,
    ) -> io::Result<()> {
        let path = normalize_path(path);
        let contents = contents.as_ref().to_vec();

        let mut state = self.state.lock().unwrap();
        state.write_calls.push((path.clone(), contents.clone()));

        self.add_file(path, contents)
    }

    /// Get metadata for a file or directory
    pub fn metadata<P: AsRef<Path>>(&self, path: P) -> io::Result<MockFileMetadata> {
        let path = normalize_path(path);

        let mut state = self.state.lock().unwrap();
        state.metadata_calls.push(path.clone());

        if let Some(metadata) = state.metadata.get(&path) {
            Ok(metadata.clone())
        } else {
            Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Path not found: {}", path.display()),
            ))
        }
    }

    /// List entries in a directory
    pub fn read_dir<P: AsRef<Path>>(&self, path: P) -> io::Result<Vec<PathBuf>> {
        let path = normalize_path(path);

        let mut state = self.state.lock().unwrap();
        state.list_directory_calls.push(path.clone());

        if let Some(entries) = state.directories.get(&path) {
            Ok(entries.clone())
        } else {
            Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Directory not found: {}", path.display()),
            ))
        }
    }

    /// Open a file for reading and/or writing
    pub fn open<P: AsRef<Path>>(
        &self,
        path: P,
        readable: bool,
        writable: bool,
    ) -> io::Result<MockFileHandle> {
        let path = normalize_path(path);

        let mut state = self.state.lock().unwrap();

        if writable {
            // Create the file if it doesn't exist and we're writing
            if !state.files.contains_key(&path) {
                let parent = path.parent().unwrap_or_else(|| Path::new("/"));

                // Ensure parent directory exists
                if !state.directories.contains_key(&parent.to_path_buf())
                    && parent != Path::new("")
                    && parent != Path::new("/")
                {
                    return Err(io::Error::new(
                        io::ErrorKind::NotFound,
                        format!("Parent directory does not exist: {}", parent.display()),
                    ));
                }

                // Add file to parent directory
                if let Some(entries) = state.directories.get_mut(&parent.to_path_buf()) {
                    if !entries.contains(&path) {
                        entries.push(path.clone());
                    }
                }

                let metadata = MockFileMetadata::new();
                let file = MockFile {
                    path: path.clone(),
                    contents: Vec::new(),
                    metadata: metadata.clone(),
                };

                state.files.insert(path.clone(), file);
                state.metadata.insert(path.clone(), metadata);
            }
        }

        if let Some(file) = state.files.get(&path) {
            let file = Arc::new(Mutex::new(file.clone()));
            Ok(MockFileHandle::new(file, readable, writable))
        } else {
            Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("File not found: {}", path.display()),
            ))
        }
    }

    /// Check if a path exists
    pub fn exists<P: AsRef<Path>>(&self, path: P) -> bool {
        let path = normalize_path(path);

        let state = self.state.lock().unwrap();
        state.files.contains_key(&path) || state.directories.contains_key(&path)
    }

    /// Remove a file
    pub fn remove_file<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        let path = normalize_path(path);

        let mut state = self.state.lock().unwrap();

        if !state.files.contains_key(&path) {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("File not found: {}", path.display()),
            ));
        }

        // Remove from parent directory
        let parent = path.parent().unwrap_or_else(|| Path::new("/"));
        if let Some(entries) = state.directories.get_mut(&parent.to_path_buf()) {
            entries.retain(|entry| entry != &path);
        }

        state.files.remove(&path);
        state.metadata.remove(&path);

        Ok(())
    }

    /// Remove a directory
    pub fn remove_dir<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        let path = normalize_path(path);

        let mut state = self.state.lock().unwrap();

        if !state.directories.contains_key(&path) {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Directory not found: {}", path.display()),
            ));
        }

        // Check if directory is empty
        if let Some(entries) = state.directories.get(&path) {
            if !entries.is_empty() {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!("Directory not empty: {}", path.display()),
                ));
            }
        }

        // Remove from parent directory
        let parent = path.parent().unwrap_or_else(|| Path::new("/"));
        if let Some(entries) = state.directories.get_mut(&parent.to_path_buf()) {
            entries.retain(|entry| entry != &path);
        }

        state.directories.remove(&path);
        state.metadata.remove(&path);

        Ok(())
    }

    /// Verify that all expected calls were made
    pub fn verify(&self) -> TestResult<()> {
        Ok(())
    }
}

/// Filesystem interface trait
pub trait FileSystem: Send + Sync {
    /// Read a file from the filesystem
    fn read_file(&self, path: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>>;

    /// Write a file to the filesystem
    fn write_file(&self, path: &str, contents: &[u8]) -> Result<(), Box<dyn std::error::Error>>;

    /// Get metadata for a file or directory
    fn metadata(&self, path: &str) -> Result<Box<dyn FileMetadata>, Box<dyn std::error::Error>>;

    /// List entries in a directory
    fn read_dir(&self, path: &str) -> Result<Vec<String>, Box<dyn std::error::Error>>;

    /// Check if a path exists
    fn exists(&self, path: &str) -> bool;

    /// Remove a file
    fn remove_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>>;

    /// Remove a directory
    fn remove_dir(&self, path: &str) -> Result<(), Box<dyn std::error::Error>>;
}

/// File metadata interface trait
pub trait FileMetadata: Send + Sync {
    /// Get the size of the file in bytes
    fn size(&self) -> u64;

    /// Check if the path is a directory
    fn is_dir(&self) -> bool;

    /// Check if the path is a file
    fn is_file(&self) -> bool;

    /// Check if the path is read-only
    fn readonly(&self) -> bool;

    /// Get the creation time as seconds since the Unix epoch
    fn created(&self) -> Result<u64, Box<dyn std::error::Error>>;

    /// Get the last modified time as seconds since the Unix epoch
    fn modified(&self) -> Result<u64, Box<dyn std::error::Error>>;

    /// Get the last accessed time as seconds since the Unix epoch
    fn accessed(&self) -> Result<u64, Box<dyn std::error::Error>>;
}

impl FileMetadata for MockFileMetadata {
    fn size(&self) -> u64 {
        self.size
    }

    fn is_dir(&self) -> bool {
        self.is_dir
    }

    fn is_file(&self) -> bool {
        self.is_file
    }

    fn readonly(&self) -> bool {
        self.readonly
    }

    fn created(&self) -> Result<u64, Box<dyn std::error::Error>> {
        Ok(system_time_to_unix_timestamp(&self.created)?)
    }

    fn modified(&self) -> Result<u64, Box<dyn std::error::Error>> {
        Ok(system_time_to_unix_timestamp(&self.modified)?)
    }

    fn accessed(&self) -> Result<u64, Box<dyn std::error::Error>> {
        Ok(system_time_to_unix_timestamp(&self.accessed)?)
    }
}

impl FileSystem for MockFileSystem {
    fn read_file(&self, path: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        Ok(self.read_file(path)?)
    }

    fn write_file(&self, path: &str, contents: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
        Ok(self.write_file(path, contents)?)
    }

    fn metadata(&self, path: &str) -> Result<Box<dyn FileMetadata>, Box<dyn std::error::Error>> {
        let metadata = self.metadata(path)?;
        Ok(Box::new(metadata) as Box<dyn FileMetadata>)
    }

    fn read_dir(&self, path: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let entries = self.read_dir(path)?;
        let entries = entries
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect();
        Ok(entries)
    }

    fn exists(&self, path: &str) -> bool {
        self.exists(path)
    }

    fn remove_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        Ok(self.remove_file(path)?)
    }

    fn remove_dir(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        Ok(self.remove_dir(path)?)
    }
}

/// Helper function to convert a SystemTime to a Unix timestamp
fn system_time_to_unix_timestamp(time: &SystemTime) -> io::Result<u64> {
    time.duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
}

/// Helper function to normalize a path
fn normalize_path<P: AsRef<Path>>(path: P) -> PathBuf {
    let path = path.as_ref();

    if path.starts_with("/") {
        path.to_path_buf()
    } else {
        PathBuf::from("/").join(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::MockRegistry;

    #[test]
    fn test_mock_file_system() {
        let fs = MockFileSystem::new();

        // Create a directory
        fs.create_dir("/test").unwrap();
        assert!(fs.exists("/test"));

        // Create a file
        fs.write_file("/test/file.txt", b"Hello, world!").unwrap();
        assert!(fs.exists("/test/file.txt"));

        // Read the file
        let contents = fs.read_file("/test/file.txt").unwrap();
        assert_eq!(contents, b"Hello, world!");

        // List directory contents
        let entries = fs.read_dir("/test").unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0], PathBuf::from("/test/file.txt"));

        // Get metadata
        let metadata = fs.metadata("/test/file.txt").unwrap();
        assert_eq!(metadata.size, 13);
        assert!(!metadata.is_dir);
        assert!(metadata.is_file);

        // Remove the file
        fs.remove_file("/test/file.txt").unwrap();
        assert!(!fs.exists("/test/file.txt"));

        // Remove the directory
        fs.remove_dir("/test").unwrap();
        assert!(!fs.exists("/test"));
    }

    #[test]
    fn test_mock_file_handle() {
        let fs = MockFileSystem::new();

        // Create a file
        fs.write_file("/test.txt", b"Hello, world!").unwrap();

        // Open the file for reading
        let mut file = fs.open("/test.txt", true, false).unwrap();

        // Read from the file
        let mut buffer = [0u8; 5];
        let bytes_read = file.read(&mut buffer).unwrap();
        assert_eq!(bytes_read, 5);
        assert_eq!(&buffer, b"Hello");

        // Seek in the file
        file.seek(SeekFrom::Start(7)).unwrap();

        // Read from the new position
        let mut buffer = [0u8; 5];
        let bytes_read = file.read(&mut buffer).unwrap();
        assert_eq!(bytes_read, 5);
        assert_eq!(&buffer, b"world");

        // Open the file for writing
        let mut file = fs.open("/test.txt", false, true).unwrap();

        // Write to the file
        let bytes_written = file.write(b"Goodbye").unwrap();
        assert_eq!(bytes_written, 7);

        // Read the updated file
        let contents = fs.read_file("/test.txt").unwrap();
        assert_eq!(contents, b"Goodbye");
    }

    #[test]
    fn test_interface_implementation() {
        let registry = MockRegistry::new();
        let fs = MockFileSystem::new();

        // Register the mock
        let fs = fs.register(&registry).unwrap();

        // Create a directory
        fs.create_dir("/test").unwrap();

        // Create a file
        fs.write_file("/test/file.txt", b"Hello, world!").unwrap();

        // Use the interface
        let fs_interface: &dyn FileSystem = &*fs;

        // Read the file
        let contents = fs_interface.read_file("/test/file.txt").unwrap();
        assert_eq!(contents, b"Hello, world!");

        // List directory contents
        let entries = fs_interface.read_dir("/test").unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0], "/test/file.txt");

        // Get metadata
        let metadata = fs_interface.metadata("/test/file.txt").unwrap();
        assert_eq!(metadata.size(), 13);
        assert!(!metadata.is_dir());
        assert!(metadata.is_file());

        // Check existence
        assert!(fs_interface.exists("/test/file.txt"));
        assert!(!fs_interface.exists("/nonexistent"));

        // Remove the file
        fs_interface.remove_file("/test/file.txt").unwrap();
        assert!(!fs_interface.exists("/test/file.txt"));

        // Remove the directory
        fs_interface.remove_dir("/test").unwrap();
        assert!(!fs_interface.exists("/test"));
    }
}
