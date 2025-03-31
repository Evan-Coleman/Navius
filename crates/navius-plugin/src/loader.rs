use std::collections::HashSet;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::error::{PluginError, PluginResult};
use crate::plugin::Plugin;

/// Symbol name for the plugin creation function
const PLUGIN_CREATE_FN: &str = "create_plugin";

/// Type for the plugin creation function
type PluginCreateFn = unsafe fn() -> *mut dyn Plugin;

/// Loads plugins from dynamic libraries
pub struct PluginLoader {
    /// Search paths for plugin libraries
    search_paths: Vec<PathBuf>,

    /// Loaded libraries (to keep them in memory)
    #[allow(dead_code)]
    loaded_libs: Vec<libloading::Library>,
}

impl PluginLoader {
    /// Create a new plugin loader
    pub fn new() -> Self {
        Self {
            search_paths: vec![PathBuf::from("./plugins")],
            loaded_libs: Vec::new(),
        }
    }

    /// Add a search path for plugin libraries
    pub fn add_search_path(&mut self, path: impl AsRef<Path>) -> &mut Self {
        self.search_paths.push(PathBuf::from(path.as_ref()));
        self
    }

    /// Find all plugin libraries in the search paths
    pub fn find_plugins(&self) -> PluginResult<Vec<PathBuf>> {
        let mut result = Vec::new();
        let mut seen = HashSet::new();

        for search_path in &self.search_paths {
            if !search_path.exists() || !search_path.is_dir() {
                continue;
            }

            let entries = match std::fs::read_dir(search_path) {
                Ok(entries) => entries,
                Err(e) => {
                    return Err(PluginError::IoError(e));
                }
            };

            for entry in entries {
                let entry = match entry {
                    Ok(entry) => entry,
                    Err(e) => {
                        return Err(PluginError::IoError(e));
                    }
                };

                let path = entry.path();

                // Skip directories and non-library files
                if path.is_dir() || !Self::is_library_file(&path) {
                    continue;
                }

                // Avoid duplicates
                let canonical = match path.canonicalize() {
                    Ok(path) => path,
                    Err(e) => {
                        return Err(PluginError::IoError(e));
                    }
                };

                if seen.insert(canonical.clone()) {
                    result.push(canonical);
                }
            }
        }

        Ok(result)
    }

    /// Load a plugin from a dynamic library
    pub fn load_plugin(&mut self, path: impl AsRef<Path>) -> PluginResult<Box<dyn Plugin>> {
        let path = path.as_ref();

        // Load the library
        let lib = unsafe {
            match libloading::Library::new(path) {
                Ok(lib) => lib,
                Err(e) => {
                    return Err(PluginError::LibraryError(format!(
                        "Failed to load library {}: {}",
                        path.display(),
                        e
                    )));
                }
            }
        };

        // Get the plugin creation function
        let create_plugin: libloading::Symbol<PluginCreateFn> = unsafe {
            match lib.get(PLUGIN_CREATE_FN.as_bytes()) {
                Ok(sym) => sym,
                Err(e) => {
                    return Err(PluginError::LibraryError(format!(
                        "Failed to find {} function in {}: {}",
                        PLUGIN_CREATE_FN,
                        path.display(),
                        e
                    )));
                }
            }
        };

        // Create the plugin
        let plugin = unsafe { Box::from_raw(create_plugin()) };

        // Store the library to keep it loaded
        self.loaded_libs.push(lib);

        Ok(plugin)
    }

    /// Scan all search paths and load all plugins
    pub fn scan_and_load(&mut self) -> PluginResult<Vec<Box<dyn Plugin>>> {
        let plugin_paths = self.find_plugins()?;
        let mut plugins = Vec::new();

        for path in plugin_paths {
            match self.load_plugin(&path) {
                Ok(plugin) => {
                    plugins.push(plugin);
                }
                Err(e) => {
                    eprintln!("Error loading plugin {}: {}", path.display(), e);
                    // Continue loading other plugins on error
                }
            }
        }

        Ok(plugins)
    }

    /// Check if a file is a dynamic library
    fn is_library_file(path: &Path) -> bool {
        if let Some(ext) = path.extension().and_then(OsStr::to_str) {
            match ext {
                "dll" | "so" | "dylib" => return true,
                _ => {}
            }
        }
        false
    }
}

impl Default for PluginLoader {
    fn default() -> Self {
        Self::new()
    }
}

/// Shared plugin loader
pub struct SharedPluginLoader {
    /// Internal plugin loader
    loader: Arc<std::sync::Mutex<PluginLoader>>,
}

impl SharedPluginLoader {
    /// Create a new shared plugin loader
    pub fn new() -> Self {
        Self {
            loader: Arc::new(std::sync::Mutex::new(PluginLoader::new())),
        }
    }

    /// Add a search path for plugin libraries
    pub fn add_search_path(&self, path: impl AsRef<Path>) -> PluginResult<()> {
        let mut loader = match self.loader.lock() {
            Ok(loader) => loader,
            Err(e) => {
                return Err(PluginError::Other(format!(
                    "Failed to lock plugin loader: {}",
                    e
                )));
            }
        };

        loader.add_search_path(path);
        Ok(())
    }

    /// Find all plugin libraries in the search paths
    pub fn find_plugins(&self) -> PluginResult<Vec<PathBuf>> {
        let loader = match self.loader.lock() {
            Ok(loader) => loader,
            Err(e) => {
                return Err(PluginError::Other(format!(
                    "Failed to lock plugin loader: {}",
                    e
                )));
            }
        };

        loader.find_plugins()
    }

    /// Load a plugin from a dynamic library
    pub fn load_plugin(&self, path: impl AsRef<Path>) -> PluginResult<Box<dyn Plugin>> {
        let mut loader = match self.loader.lock() {
            Ok(loader) => loader,
            Err(e) => {
                return Err(PluginError::Other(format!(
                    "Failed to lock plugin loader: {}",
                    e
                )));
            }
        };

        loader.load_plugin(path)
    }

    /// Scan all search paths and load all plugins
    pub fn scan_and_load(&self) -> PluginResult<Vec<Box<dyn Plugin>>> {
        let mut loader = match self.loader.lock() {
            Ok(loader) => loader,
            Err(e) => {
                return Err(PluginError::Other(format!(
                    "Failed to lock plugin loader: {}",
                    e
                )));
            }
        };

        loader.scan_and_load()
    }
}

impl Default for SharedPluginLoader {
    fn default() -> Self {
        Self::new()
    }
}

/// Macro for implementing the plugin creation function in a dynamic library
#[macro_export]
macro_rules! export_plugin {
    ($plugin_type:ty) => {
        #[no_mangle]
        pub unsafe extern "C" fn create_plugin() -> *mut dyn $crate::plugin::Plugin {
            let plugin = Box::new(<$plugin_type>::new());
            Box::into_raw(plugin)
        }
    };
}

/// In-memory plugin provider for testing and development
pub struct InMemoryPluginProvider {
    /// Stored plugins
    plugins: Vec<Box<dyn Plugin>>,
}

impl InMemoryPluginProvider {
    /// Create a new in-memory plugin provider
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
        }
    }

    /// Add a plugin to the provider
    pub fn add_plugin<P: Plugin + 'static>(&mut self, plugin: P) -> &mut Self {
        self.plugins.push(Box::new(plugin));
        self
    }

    /// Add multiple plugins to the provider
    pub fn add_plugins<P: Plugin + 'static>(&mut self, plugins: Vec<P>) -> &mut Self {
        for plugin in plugins {
            self.add_plugin(plugin);
        }
        self
    }

    /// Get all plugins
    pub fn get_plugins(&self) -> &[Box<dyn Plugin>] {
        &self.plugins
    }

    /// Take ownership of all plugins
    pub fn take_plugins(&mut self) -> Vec<Box<dyn Plugin>> {
        std::mem::take(&mut self.plugins)
    }
}

impl Default for InMemoryPluginProvider {
    fn default() -> Self {
        Self::new()
    }
}
