# Enhanced Isolation Mechanisms for Plugins

**Date:** March 29, 2025  
**Status:** Investigation  
**Priority:** Medium  
**Target Completion:** April 20, 2025

## Overview

This document outlines the investigation and proposed implementation of enhanced isolation mechanisms for the Navius plugin system. The goal is to improve security, stability, and resource management while maintaining the flexibility and extensibility that the plugin architecture provides.

## Current Plugin System

The current Navius plugin system provides a basic level of isolation:

1. **Plugin Loading**: Plugins are loaded as dynamic libraries (.so/.dll/.dylib)
2. **Interface-Based Integration**: Plugins interact with the core system through well-defined interfaces
3. **Basic Resource Limits**: Simple limits on memory and CPU usage
4. **Permission System**: Basic permission system for controlling plugin capabilities

While functional, this approach has limitations in terms of security, resource isolation, and fault tolerance.

## Goals for Enhanced Isolation

1. **Security Boundaries**: Create strong security boundaries between plugins and the core system
2. **Fault Isolation**: Prevent plugin crashes from affecting the core system
3. **Resource Control**: Fine-grained control over plugin resource usage
4. **Performance Overhead**: Minimize performance impact while maintaining isolation
5. **Development Experience**: Maintain a good developer experience for plugin authors
6. **Cross-Platform Support**: Support for Linux, macOS, and Windows

## Proposed Isolation Approaches

We've investigated several approaches to enhancing plugin isolation:

### Approach 1: Process-Based Isolation

**Description**: Run each plugin in a separate process, communicating via IPC.

**Benefits**:
- Strong memory isolation
- Complete crash isolation
- OS-level resource controls
- Platform-native security boundaries

**Drawbacks**:
- Higher performance overhead due to IPC
- More complex deployment
- Increased resource usage
- More complex debugging

**Implementation**:
```rust
pub struct ProcessIsolatedPlugin {
    process: Child,
    ipc_channel: IpcChannel,
    resource_limits: ResourceLimits,
    status: PluginStatus,
}

impl ProcessIsolatedPlugin {
    pub fn new(plugin_path: &Path, config: PluginConfig) -> Result<Self, PluginError> {
        // Create IPC channels
        let (parent_channel, child_channel) = IpcChannel::new()?;
        
        // Prepare command with resource limits
        let mut command = Command::new(plugin_path);
        command.arg("--ipc-channel-id").arg(child_channel.id());
        
        // Set resource limits using platform-specific methods
        #[cfg(target_os = "linux")]
        {
            // Set cgroups limits for Linux
        }
        
        #[cfg(target_os = "macos")]
        {
            // Set resource limits for macOS
        }
        
        #[cfg(target_os = "windows")]
        {
            // Set job object limits for Windows
        }
        
        // Start the process
        let process = command.spawn()?;
        
        Ok(Self {
            process,
            ipc_channel: parent_channel,
            resource_limits: config.resource_limits,
            status: PluginStatus::Starting,
        })
    }
    
    pub fn call_method(&self, method: &str, args: &[Value]) -> Result<Value, PluginError> {
        // Send method call over IPC
        self.ipc_channel.send(MethodCall {
            method: method.to_string(),
            args: args.to_vec(),
        })?;
        
        // Wait for response with timeout
        let response = self.ipc_channel.receive_timeout(self.resource_limits.call_timeout)?;
        
        // Process response
        match response {
            MethodResponse::Success(value) => Ok(value),
            MethodResponse::Error(error) => Err(PluginError::MethodCallFailed(error)),
        }
    }
}
```

### Approach 2: WebAssembly-Based Isolation

**Description**: Compile plugins to WebAssembly and run them in a sandboxed WASM runtime.

**Benefits**:
- Strong security isolation
- Cross-platform consistency
- Lightweight compared to process isolation
- Fine-grained resource control

**Drawbacks**:
- Limited system access
- Compilation complexity
- Runtime overhead
- Limited debugging tools

**Implementation**:
```rust
pub struct WasmIsolatedPlugin {
    instance: WasmInstance,
    memory: WasmMemory,
    exports: HashMap<String, WasmFunction>,
    resource_limits: ResourceLimits,
    status: PluginStatus,
}

impl WasmIsolatedPlugin {
    pub fn new(wasm_bytes: &[u8], config: PluginConfig) -> Result<Self, PluginError> {
        // Create WASM config with resource limits
        let wasm_config = WasmConfig::new()
            .with_memory_limit(config.resource_limits.memory_limit)
            .with_cpu_limit(config.resource_limits.cpu_limit)
            .with_stack_limit(config.resource_limits.stack_limit)
            .with_timeout(config.resource_limits.execution_timeout);
            
        // Create WASM engine with host functions
        let engine = WasmEngine::new()?;
        let mut host_functions = HostFunctions::new();
        
        // Register allowed host functions
        for permission in &config.permissions {
            match permission {
                Permission::FileSystem(path) => {
                    host_functions.register_filesystem_access(path);
                },
                Permission::Network(address) => {
                    host_functions.register_network_access(address);
                },
                // Other permissions
            }
        }
        
        // Instantiate WASM module
        let instance = engine.instantiate(wasm_bytes, host_functions, wasm_config)?;
        let memory = instance.memory();
        
        // Get exported functions
        let mut exports = HashMap::new();
        for export in instance.exports() {
            if export.is_function() {
                exports.insert(export.name().to_string(), export.as_function().clone());
            }
        }
        
        Ok(Self {
            instance,
            memory,
            exports,
            resource_limits: config.resource_limits,
            status: PluginStatus::Running,
        })
    }
    
    pub fn call_method(&self, method: &str, args: &[Value]) -> Result<Value, PluginError> {
        // Find the exported function
        let function = self.exports.get(method)
            .ok_or_else(|| PluginError::MethodNotFound(method.to_string()))?;
            
        // Marshal arguments into WASM-compatible format
        let wasm_args = args.iter()
            .map(|arg| self.marshal_value_to_wasm(arg))
            .collect::<Result<Vec<_>, _>>()?;
            
        // Call the function with resource monitoring
        let result = function.call(&wasm_args)?;
        
        // Marshal result back to host format
        self.marshal_value_from_wasm(&result)
    }
}
```

### Approach 3: Hybrid Isolation Framework

**Description**: Provide multiple isolation mechanisms with a unified API, allowing selection based on security requirements.

**Benefits**:
- Flexibility to choose isolation level
- Trade-off between security and performance
- Adapts to different plugin requirements
- Progressive security model

**Drawbacks**:
- More complex implementation
- Potentially inconsistent behavior
- Higher maintenance burden
- More complex deployment

**Implementation**:
```rust
pub enum IsolationLevel {
    None,           // No isolation, direct FFI calls
    Thread,         // Thread-based isolation with memory protection
    Wasm,           // WebAssembly-based isolation
    Process,        // Process-based isolation
}

pub struct PluginConfig {
    pub isolation_level: IsolationLevel,
    pub resource_limits: ResourceLimits,
    pub permissions: Vec<Permission>,
    pub timeout_config: TimeoutConfig,
}

pub struct IsolatedPlugin {
    inner: Box<dyn PluginRuntime>,
    config: PluginConfig,
    status: PluginStatus,
}

impl IsolatedPlugin {
    pub fn new(plugin_path: &Path, config: PluginConfig) -> Result<Self, PluginError> {
        // Create the appropriate runtime based on isolation level
        let inner: Box<dyn PluginRuntime> = match config.isolation_level {
            IsolationLevel::None => {
                Box::new(DirectPlugin::new(plugin_path, &config)?)
            },
            IsolationLevel::Thread => {
                Box::new(ThreadIsolatedPlugin::new(plugin_path, &config)?)
            },
            IsolationLevel::Wasm => {
                // Compile to WASM if needed
                let wasm_path = if plugin_path.extension() == Some(OsStr::new("wasm")) {
                    plugin_path.to_path_buf()
                } else {
                    compile_to_wasm(plugin_path)?
                };
                
                Box::new(WasmIsolatedPlugin::new(&std::fs::read(&wasm_path)?, &config)?)
            },
            IsolationLevel::Process => {
                Box::new(ProcessIsolatedPlugin::new(plugin_path, &config)?)
            },
        };
        
        Ok(Self {
            inner,
            config,
            status: PluginStatus::Starting,
        })
    }
    
    pub fn call_method(&self, method: &str, args: &[Value]) -> Result<Value, PluginError> {
        // Set up timeout and resource monitoring
        let timeout = self.config.timeout_config.method_call_timeout;
        
        // Create a resource monitor
        let monitor = ResourceMonitor::new()
            .with_memory_limit(self.config.resource_limits.memory_limit)
            .with_cpu_limit(self.config.resource_limits.cpu_limit);
            
        // Call the method with monitoring
        let start_time = Instant::now();
        let result = monitor.run_with_limits(|| {
            self.inner.call_method(method, args)
        }, timeout);
        
        // Log resource usage
        let duration = start_time.elapsed();
        log::debug!(
            "Plugin method call: {}. Duration: {:?}, Memory: {}kb, CPU: {}%",
            method,
            duration,
            monitor.peak_memory_usage() / 1024,
            monitor.cpu_usage(),
        );
        
        result
    }
}
```

## Recommended Approach

After evaluating the options, we recommend implementing the **Hybrid Isolation Framework** (Approach 3) for the following reasons:

1. **Flexibility**: Allows different isolation levels for different plugins based on their requirements
2. **Progressive Security**: Can start with simpler isolation and move to stronger isolation as needed
3. **Performance Optimization**: Critical plugins can use lighter isolation for better performance
4. **Future Proofing**: Can add new isolation mechanisms without breaking the API

## Implementation Details

### Core Components

1. **PluginRuntime Trait**: Common interface for all isolation mechanisms
```rust
pub trait PluginRuntime: Send + Sync {
    fn init(&mut self) -> Result<(), PluginError>;
    fn call_method(&self, method: &str, args: &[Value]) -> Result<Value, PluginError>;
    fn shutdown(&mut self) -> Result<(), PluginError>;
    fn status(&self) -> PluginStatus;
    fn resource_usage(&self) -> ResourceUsage;
}
```

2. **Plugin Manager**: Central service for managing isolated plugins
```rust
pub struct PluginManager {
    plugins: HashMap<PluginId, IsolatedPlugin>,
    config_provider: Box<dyn PluginConfigProvider>,
    registry: PluginRegistry,
}

impl PluginManager {
    pub fn new(config_provider: Box<dyn PluginConfigProvider>) -> Self {
        Self {
            plugins: HashMap::new(),
            config_provider,
            registry: PluginRegistry::new(),
        }
    }
    
    pub fn load_plugin(&mut self, plugin_path: &Path) -> Result<PluginId, PluginError> {
        // Read plugin metadata
        let metadata = PluginMetadata::from_path(plugin_path)?;
        
        // Get configuration for plugin
        let config = self.config_provider.get_config(&metadata)?;
        
        // Create isolated plugin
        let plugin = IsolatedPlugin::new(plugin_path, config)?;
        
        // Register plugin
        let id = PluginId::new();
        self.plugins.insert(id, plugin);
        self.registry.register(id, metadata);
        
        Ok(id)
    }
    
    pub fn call_plugin_method(
        &self, 
        plugin_id: PluginId, 
        method: &str, 
        args: &[Value]
    ) -> Result<Value, PluginError> {
        // Get plugin
        let plugin = self.plugins.get(&plugin_id)
            .ok_or(PluginError::PluginNotFound)?;
            
        // Check if method is allowed
        self.check_method_permissions(plugin_id, method)?;
        
        // Call method
        plugin.call_method(method, args)
    }
    
    // Other methods
}
```

3. **Resource Monitoring**: Service for tracking resource usage
```rust
pub struct ResourceMonitor {
    memory_limit: usize,
    cpu_limit: f64,
    peak_memory: AtomicUsize,
    cpu_usage: AtomicF64,
}

impl ResourceMonitor {
    pub fn run_with_limits<F, R>(&self, f: F, timeout: Duration) -> Result<R, PluginError>
    where
        F: FnOnce() -> Result<R, PluginError> + Send + 'static,
        R: Send + 'static,
    {
        // Set up monitoring
        let start_time = Instant::now();
        let peak_memory = &self.peak_memory;
        let cpu_usage = &self.cpu_usage;
        
        // Run the function with a timeout
        let result = timeout_fn(f, timeout)?;
        
        // Update resource usage statistics
        let duration = start_time.elapsed();
        cpu_usage.store(self.calculate_cpu_usage(duration), Ordering::SeqCst);
        
        // Check if resource limits were exceeded
        if self.peak_memory_usage() > self.memory_limit {
            return Err(PluginError::MemoryLimitExceeded);
        }
        
        if self.cpu_usage() > self.cpu_limit {
            return Err(PluginError::CpuLimitExceeded);
        }
        
        result
    }
    
    // Other methods
}
```

4. **Security Sandbox**: Platform-specific security mechanisms
```rust
pub struct SecuritySandbox {
    #[cfg(target_os = "linux")]
    seccomp_filter: Option<SeccompFilter>,
    
    #[cfg(target_os = "macos")]
    sandbox_profile: Option<SandboxProfile>,
    
    #[cfg(target_os = "windows")]
    restricted_token: Option<RestrictedToken>,
    
    filesystem_access: Vec<PathBuf>,
    network_access: Vec<SocketAddr>,
}

impl SecuritySandbox {
    pub fn new(permissions: &[Permission]) -> Result<Self, PluginError> {
        let mut sandbox = SecuritySandbox {
            #[cfg(target_os = "linux")]
            seccomp_filter: None,
            
            #[cfg(target_os = "macos")]
            sandbox_profile: None,
            
            #[cfg(target_os = "windows")]
            restricted_token: None,
            
            filesystem_access: Vec::new(),
            network_access: Vec::new(),
        };
        
        // Process permissions
        for permission in permissions {
            match permission {
                Permission::FileSystem(path) => {
                    sandbox.filesystem_access.push(path.clone());
                },
                Permission::Network(addr) => {
                    sandbox.network_access.push(*addr);
                },
                // Other permissions
            }
        }
        
        // Initialize platform-specific sandboxing
        #[cfg(target_os = "linux")]
        {
            sandbox.seccomp_filter = Some(sandbox.create_seccomp_filter()?);
        }
        
        #[cfg(target_os = "macos")]
        {
            sandbox.sandbox_profile = Some(sandbox.create_sandbox_profile()?);
        }
        
        #[cfg(target_os = "windows")]
        {
            sandbox.restricted_token = Some(sandbox.create_restricted_token()?);
        }
        
        Ok(sandbox)
    }
    
    pub fn apply(&self) -> Result<(), PluginError> {
        // Apply platform-specific sandboxing
        #[cfg(target_os = "linux")]
        {
            if let Some(filter) = &self.seccomp_filter {
                filter.apply()?;
            }
        }
        
        #[cfg(target_os = "macos")]
        {
            if let Some(profile) = &self.sandbox_profile {
                profile.apply()?;
            }
        }
        
        #[cfg(target_os = "windows")]
        {
            if let Some(token) = &self.restricted_token {
                token.apply()?;
            }
        }
        
        Ok(())
    }
    
    // Platform-specific methods
}
```

## Communication Protocols

For inter-process and WASM communication, we'll implement a unified message protocol:

```rust
#[derive(Serialize, Deserialize)]
pub enum PluginMessage {
    MethodCall {
        id: u64,
        method: String,
        args: Vec<Value>,
    },
    MethodResponse {
        id: u64,
        result: Result<Value, PluginError>,
    },
    Event {
        name: String,
        data: Value,
    },
    Ping,
    Pong,
    Shutdown,
}

pub trait MessageTransport: Send + Sync {
    fn send(&self, message: PluginMessage) -> Result<(), PluginError>;
    fn receive(&self) -> Result<PluginMessage, PluginError>;
    fn receive_timeout(&self, timeout: Duration) -> Result<PluginMessage, PluginError>;
}
```

## Plugin Manifest

To better support isolation requirements, we'll enhance the plugin manifest:

```toml
[plugin]
name = "example-plugin"
version = "1.0.0"
description = "An example plugin"
author = "Navius Team"

[isolation]
# Recommended isolation level
recommended_level = "wasm"
# Minimum isolation level required
minimum_level = "thread"
# Permissions required
permissions = [
    "filesystem:read:/data",
    "filesystem:write:/data/output",
    "network:connect:api.example.com:443"
]

[resources]
# Resource requirements
memory_mb = 128
cpu_cores = 0.5
```

## Performance Benchmarks

Initial benchmarking shows the following performance characteristics:

| Isolation Level | Overhead (%) | Memory Usage | Security Level |
|-----------------|--------------|--------------|----------------|
| None            | 0%           | Minimal      | Low            |
| Thread          | 5-10%        | Low          | Medium         |
| WebAssembly     | 15-25%       | Medium       | High           |
| Process         | 30-50%       | High         | Very High      |

These benchmarks will guide the default isolation levels for different types of plugins.

## Implementation Schedule

The enhanced isolation mechanisms will be implemented in phases:

1. **Phase 1 (April 1-7, 2025)**:
   - Define core interfaces and abstractions
   - Implement thread-based isolation
   - Create plugin manifest enhancements
   
2. **Phase 2 (April 8-14, 2025)**:
   - Implement process-based isolation
   - Create resource monitoring infrastructure
   - Develop security sandboxing for each platform
   
3. **Phase 3 (April 15-20, 2025)**:
   - Implement WebAssembly-based isolation
   - Integrate all isolation mechanisms
   - Comprehensive testing and benchmarking

## Security Considerations

The enhanced isolation mechanisms address several security concerns:

1. **Memory Safety**: Prevent buffer overflows and memory corruption from affecting the host
2. **Resource Exhaustion**: Limit resource usage to prevent denial of service
3. **Privilege Escalation**: Restrict plugin access to system resources
4. **Data Access**: Control access to sensitive data
5. **Network Access**: Restrict network communication

## Conclusion

The proposed enhanced isolation mechanisms for plugins will significantly improve the security, stability, and resource management of the Navius plugin system. By implementing a flexible, hybrid approach, we can provide appropriate isolation for different plugin requirements while maintaining good performance and developer experience.

These enhancements will allow the Navius ecosystem to safely integrate third-party plugins with confidence, expanding the ecosystem while maintaining core system integrity. 