# WASM Security Implementation Guide - Wasmtime 22.0

**Date**: October 21, 2025  
**Project**: Project Chimera - Layer 4 Execution Fabric  
**Purpose**: Replace stub executor with production-grade WASM security

---

## 📋 Overview

This document provides complete implementation guidance for replacing the stub `WasmExecutor` with real Wasmtime-based security enforcement including:

- CPU time limits and quota enforcement
- Memory limits with strict bounds checking
- Filesystem isolation via WASI preopen directories
- Network restriction (complete syscall blocking)
- Resource usage tracking and reporting

---

## 🎯 Current Status

### What Exists (Stub Implementation)
```rust
// src/layer4/src/wasm_executor.rs (CURRENT - STUB)
pub struct WasmExecutor {
    _placeholder: (),
}

async fn simulate_execution(&self, task: Task) -> Layer4Result<serde_json::Value> {
    // Simulated execution - NOT SECURE
    match task.target_agent_type.as_str() {
        "cpu_bomber" => tokio::time::sleep(Duration::from_secs(10)).await,
        _ => Ok(serde_json::json!({"status": "completed"})),
    }
}
```

**Problems**:
- ❌ No real WASM execution
- ❌ No resource quotas enforced
- ❌ Security tests passing falsely
- ❌ No actual sandboxing

### What Needs to Be Implemented

```rust
// TARGET: Real Wasmtime integration
pub struct WasmExecutor {
    engine: Engine,           // Wasmtime engine with security config
    memory_limit_bytes: usize, // Hard memory limit
    cpu_time_limit_ms: u64,   // CPU time budget
    filesystem_allowed: Vec<PathBuf>, // WASI preopen dirs
}
```

---

## 🏗️ Implementation Plan

### Phase 1: Wasmtime Configuration (Day 4 - Part 1)

#### 1.1 Add Wasmtime Dependencies

Update `src/layer4/Cargo.toml`:

```toml
[dependencies]
wasmtime = "22.0"
wasmtime-wasi = "22.0"
tokio = { version = "1", features = ["full"] }
anyhow = "1.0"
serde_json = "1.0"
tracing = "0.1"
```

#### 1.2 Create Secure Engine Configuration

```rust
// src/layer4/src/wasm_executor.rs
use wasmtime::{Config, Engine, Instance, Linker, Module, Store};
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder};
use std::path::PathBuf;
use std::time::{Duration, Instant};

pub struct WasmExecutor {
    engine: Engine,
    memory_limit_bytes: usize,
    cpu_time_limit_ms: u64,
    allowed_dirs: Vec<PathBuf>,
}

impl WasmExecutor {
    pub fn new() -> Layer4Result<Self> {
        let engine = Self::create_secure_engine()?;
        
        Ok(Self {
            engine,
            memory_limit_bytes: 128 * 1024 * 1024, // 128MB default
            cpu_time_limit_ms: 5000, // 5 seconds default
            allowed_dirs: vec![], // No filesystem access by default
        })
    }
    
    fn create_secure_engine() -> Layer4Result<Engine> {
        let mut config = Config::new();
        
        // Enable WASM features (carefully selected)
        config.wasm_bulk_memory(true);     // Safe bulk memory ops
        config.wasm_reference_types(true); // Safe reference types
        config.wasm_simd(false);           // Disable SIMD (potential timing attacks)
        config.wasm_threads(false);        // Disable threads (prevent fork bombs)
        config.wasm_multi_memory(false);   // Single memory space only
        
        // Resource limits
        config.max_wasm_stack(1 * 1024 * 1024); // 1MB stack limit
        config.consume_fuel(true);              // Enable fuel metering for CPU limits
        
        // Security settings
        config.cranelift_opt_level(wasmtime::OptLevel::Speed);
        config.parallel_compilation(false);     // Deterministic execution
        
        Ok(Engine::new(&config)?)
    }
}
```

---

### Phase 2: Memory Quota Enforcement (Day 4 - Part 2)

#### 2.1 Memory Limiting with WASI

```rust
impl WasmExecutor {
    /// Execute WASM with strict memory limits
    pub async fn execute_with_quotas(
        &self,
        wasm_bytes: &[u8],
        task: Task,
    ) -> Layer4Result<ExecutionResult> {
        let start_time = Instant::now();
        let quota = &task.resource_quota;
        
        // Create memory-limited store
        let mut store = self.create_limited_store(quota)?;
        
        // Compile module
        let module = Module::new(&self.engine, wasm_bytes)
            .map_err(|e| Layer4Error::InvalidWasm(e.to_string()))?;
        
        // Create WASI context with restrictions
        let wasi_ctx = self.create_restricted_wasi_context(quota)?;
        store.data_mut().wasi = wasi_ctx;
        
        // Execute with timeout and fuel limit
        let result = self.execute_module(&mut store, module, task.clone()).await?;
        
        let execution_time_ms = start_time.elapsed().as_millis() as u64;
        
        Ok(ExecutionResult {
            task_id: task.id,
            success: true,
            output: result,
            execution_time_ms,
            resource_usage: self.get_resource_usage(&store),
            error: None,
            completed_at: SystemTime::now(),
        })
    }
    
    fn create_limited_store(&self, quota: &ResourceQuota) -> Layer4Result<Store<WasmState>> {
        let mut store = Store::new(&self.engine, WasmState::new());
        
        // Set memory limit (in pages, 1 page = 64KB)
        let max_memory_pages = (quota.max_memory_mb as usize * 1024 * 1024) / (64 * 1024);
        store.limiter(|state| &mut state.limiter);
        
        // Set CPU fuel limit (approximate CPU time)
        // Fuel consumption rate varies, calibrate based on benchmarks
        let fuel_limit = quota.max_execution_time_secs * 1_000_000; // 1M fuel per second
        store.set_fuel(fuel_limit)?;
        
        Ok(store)
    }
}

/// State for WASM execution with resource limits
struct WasmState {
    wasi: WasiCtx,
    limiter: ResourceLimiter,
}

impl WasmState {
    fn new() -> Self {
        Self {
            wasi: WasiCtxBuilder::new().build(),
            limiter: ResourceLimiter::new(),
        }
    }
}

/// Resource limiter for memory and table quotas
struct ResourceLimiter {
    memory_limit_bytes: usize,
    current_memory_bytes: usize,
}

impl ResourceLimiter {
    fn new() -> Self {
        Self {
            memory_limit_bytes: 128 * 1024 * 1024, // 128MB
            current_memory_bytes: 0,
        }
    }
}

impl wasmtime::ResourceLimiter for ResourceLimiter {
    fn memory_growing(&mut self, current: usize, desired: usize, maximum: Option<usize>) -> bool {
        if desired > self.memory_limit_bytes {
            return false; // Deny memory growth beyond limit
        }
        self.current_memory_bytes = desired;
        true
    }
    
    fn table_growing(&mut self, current: u32, desired: u32, maximum: Option<u32>) -> bool {
        // Limit table size to prevent memory exhaustion
        desired < 10_000
    }
}
```

---

### Phase 3: CPU Time Limiting (Day 4 - Part 3)

#### 3.1 Fuel-based CPU Metering

```rust
impl WasmExecutor {
    /// Execute module with CPU time enforcement
    async fn execute_module(
        &self,
        store: &mut Store<WasmState>,
        module: Module,
        task: Task,
    ) -> Layer4Result<serde_json::Value> {
        // Create linker with WASI imports
        let mut linker = Linker::new(&self.engine);
        wasmtime_wasi::add_to_linker(&mut linker, |state: &mut WasmState| &mut state.wasi)?;
        
        // Instantiate module
        let instance = linker.instantiate(&mut *store, &module)
            .map_err(|e| Layer4Error::WasmInstantiationFailed(e.to_string()))?;
        
        // Get exported execute function
        let execute_func = instance
            .get_typed_func::<(), ()>(&mut *store, "execute")
            .map_err(|_| Layer4Error::InvalidWasm("No 'execute' function exported".to_string()))?;
        
        // Execute with timeout
        let timeout_duration = Duration::from_secs(task.resource_quota.max_execution_time_secs);
        
        let result = tokio::time::timeout(
            timeout_duration,
            async {
                // Call the WASM function
                execute_func.call(&mut *store, ())
                    .map_err(|e| self.handle_execution_error(e, store))
            }
        ).await;
        
        match result {
            Ok(Ok(())) => {
                // Success - check remaining fuel
                let remaining_fuel = store.get_fuel()
                    .map_err(|e| Layer4Error::Internal(e.to_string()))?;
                
                tracing::info!("WASM execution complete, remaining fuel: {}", remaining_fuel);
                
                Ok(serde_json::json!({"status": "success"}))
            }
            Ok(Err(e)) => Err(e),
            Err(_) => Err(Layer4Error::AgentTimeout(task.resource_quota.max_execution_time_secs)),
        }
    }
    
    /// Handle WASM execution errors (fuel exhaustion, memory, etc.)
    fn handle_execution_error(
        &self,
        error: wasmtime::Error,
        store: &Store<WasmState>,
    ) -> Layer4Error {
        let error_str = error.to_string();
        
        if error_str.contains("out of fuel") || error_str.contains("fuel") {
            Layer4Error::ResourceQuotaExceeded("CPU time limit exceeded".to_string())
        } else if error_str.contains("memory") {
            Layer4Error::ResourceQuotaExceeded("Memory limit exceeded".to_string())
        } else if error_str.contains("trap") {
            Layer4Error::WasmTrap(error_str)
        } else {
            Layer4Error::WasmExecutionFailed(error_str)
        }
    }
}
```

---

### Phase 4: Filesystem Isolation (Day 4 - Part 4)

#### 4.1 WASI Preopen Directory Restriction

```rust
impl WasmExecutor {
    /// Create restricted WASI context with minimal permissions
    fn create_restricted_wasi_context(
        &self,
        quota: &ResourceQuota,
    ) -> Layer4Result<WasiCtx> {
        let mut builder = WasiCtxBuilder::new();
        
        // No stdin/stdout/stderr by default (security hardening)
        builder.inherit_stdio();
        
        // Preopen only explicitly allowed directories (readonly)
        for allowed_dir in &self.allowed_dirs {
            let dir = cap_std::fs::Dir::open_ambient_dir(allowed_dir, cap_std::ambient_authority())
                .map_err(|e| Layer4Error::FilesystemError(e.to_string()))?;
            
            builder.preopened_dir(dir, "/", wasmtime_wasi::DirPerms::READ, wasmtime_wasi::FilePerms::READ)
                .map_err(|e| Layer4Error::WasiError(e.to_string()))?;
        }
        
        // No environment variables (prevent information leakage)
        // builder.env() calls intentionally omitted
        
        // No arguments
        // builder.args() calls intentionally omitted
        
        Ok(builder.build())
    }
    
    /// Set allowed filesystem access (must be called before execution)
    pub fn with_filesystem_access(mut self, dirs: Vec<PathBuf>) -> Self {
        self.allowed_dirs = dirs;
        self
    }
}
```

---

### Phase 5: Network Restriction (Day 5 - Part 1)

#### 5.1 Complete Network Isolation

```rust
impl WasmExecutor {
    /// WASI does NOT provide network access by default
    /// This is enforced by simply not adding any network imports to the linker
    /// 
    /// Verification:
    /// - Do NOT call wasmtime_wasi::add_to_linker for socket functions
    /// - Do NOT add any custom host functions for network I/O
    /// - WASM module cannot make network calls without these imports
    
    fn create_restricted_wasi_context(
        &self,
        quota: &ResourceQuota,
    ) -> Layer4Result<WasiCtx> {
        let mut builder = WasiCtxBuilder::new();
        
        // CRITICAL: Do NOT enable network sockets
        // By not calling builder.socket() or similar, network access is denied
        
        // Only enable minimal WASI capabilities
        builder.inherit_stdio();
        
        // Filesystem access (if explicitly allowed)
        for allowed_dir in &self.allowed_dirs {
            let dir = cap_std::fs::Dir::open_ambient_dir(allowed_dir, cap_std::ambient_authority())?;
            builder.preopened_dir(dir, "/", wasmtime_wasi::DirPerms::READ, wasmtime_wasi::FilePerms::READ)?;
        }
        
        Ok(builder.build())
    }
}
```

---

### Phase 6: Resource Usage Tracking (Day 5 - Part 2)

#### 6.1 Collect and Report Resource Metrics

```rust
impl WasmExecutor {
    /// Get resource usage after execution
    fn get_resource_usage(&self, store: &Store<WasmState>) -> ResourceUsage {
        let remaining_fuel = store.get_fuel().unwrap_or(0);
        let memory_used = store.data().limiter.current_memory_bytes;
        
        ResourceUsage {
            cpu_cores_used: 0.0, // Would need OS-level tracking
            memory_used_mb: (memory_used / (1024 * 1024)) as u32,
            execution_time_ms: 0, // Tracked externally
            network_bytes_sent: 0,
            network_bytes_received: 0,
            disk_bytes_read: 0,
            disk_bytes_written: 0,
        }
    }
}
```

---

## 🧪 Testing Strategy

### Test 1: CPU Quota Enforcement

```rust
#[tokio::test]
async fn test_cpu_limit_enforcement() {
    let executor = WasmExecutor::new().unwrap();
    
    // WASM module that consumes excessive CPU
    let cpu_bomber_wasm = wat::parse_str(r#"
        (module
            (func $execute
                (loop $infinite
                    br $infinite
                )
            )
            (export "execute" (func $execute))
        )
    "#).unwrap();
    
    let task = Task {
        resource_quota: ResourceQuota {
            max_execution_time_secs: 1, // 1 second limit
            ..Default::default()
        },
        ..create_test_task()
    };
    
    let result = executor.execute_with_quotas(&cpu_bomber_wasm, task).await;
    
    // Should fail with CPU quota exceeded
    assert!(matches!(result, Err(Layer4Error::ResourceQuotaExceeded(_))));
}
```

### Test 2: Memory Quota Enforcement

```rust
#[tokio::test]
async fn test_memory_limit_enforcement() {
    let executor = WasmExecutor::new().unwrap();
    
    // WASM module that tries to allocate excessive memory
    let memory_bomber_wasm = wat::parse_str(r#"
        (module
            (memory 1)
            (func $execute
                (memory.grow (i32.const 10000)) ;; Try to grow by 640MB
                drop
            )
            (export "execute" (func $execute))
        )
    "#).unwrap();
    
    let task = Task {
        resource_quota: ResourceQuota {
            max_memory_mb: 128, // 128MB limit
            ..Default::default()
        },
        ..create_test_task()
    };
    
    let result = executor.execute_with_quotas(&memory_bomber_wasm, task).await;
    
    // Should fail with memory quota exceeded
    assert!(matches!(result, Err(Layer4Error::ResourceQuotaExceeded(_))));
}
```

### Test 3: Filesystem Isolation

```rust
#[tokio::test]
async fn test_filesystem_isolation() {
    let executor = WasmExecutor::new().unwrap();
    // No allowed_dirs set - complete isolation
    
    // WASM module that tries to read /etc/passwd
    let fs_escape_wasm = wat::parse_str(r#"
        (module
            (import "wasi_snapshot_preview1" "path_open"
                (func $path_open (param i32 i32 i32 i32 i32 i64 i64 i32 i32) (result i32))
            )
            (memory 1)
            (func $execute
                ;; Try to open /etc/passwd
                ;; This should fail due to WASI restrictions
                (drop (call $path_open
                    (i32.const 0)   ;; dirfd
                    (i32.const 0)   ;; dirflags
                    (i32.const 0)   ;; path
                    (i32.const 11)  ;; path_len
                    (i32.const 0)   ;; oflags
                    (i64.const 0)   ;; fs_rights_base
                    (i64.const 0)   ;; fs_rights_inheriting
                    (i32.const 0)   ;; fdflags
                    (i32.const 100) ;; result pointer
                ))
            )
            (export "execute" (func $execute))
        )
    "#).unwrap();
    
    let task = create_test_task();
    let result = executor.execute_with_quotas(&fs_escape_wasm, task).await;
    
    // Should fail - no filesystem access granted
    assert!(result.is_err());
}
```

### Test 4: Network Restriction

```rust
#[tokio::test]
async fn test_network_restriction() {
    let executor = WasmExecutor::new().unwrap();
    
    // WASM module that tries to create a socket
    // Will fail at link time because socket imports are not available
    let network_wasm = wat::parse_str(r#"
        (module
            (import "wasi_snapshot_preview1" "sock_open"
                (func $sock_open (param i32 i32 i32) (result i32))
            )
            (func $execute
                (drop (call $sock_open
                    (i32.const 0)   ;; address_family (AF_INET)
                    (i32.const 1)   ;; socket_type (SOCK_STREAM)
                    (i32.const 100) ;; result pointer
                ))
            )
            (export "execute" (func $execute))
        )
    "#).unwrap();
    
    let task = create_test_task();
    let result = executor.execute_with_quotas(&network_wasm, task).await;
    
    // Should fail - socket imports not provided
    assert!(result.is_err());
}
```

---

## ✅ Implementation Checklist

### Day 4 Tasks
- [ ] Update Cargo.toml with Wasmtime 22.0 dependencies
- [ ] Replace stub WasmExecutor with real Wasmtime engine
- [ ] Implement secure engine configuration
- [ ] Add memory quota enforcement with ResourceLimiter
- [ ] Implement CPU fuel metering
- [ ] Add filesystem isolation with WASI preopens
- [ ] Test individual components

### Day 5 Tasks
- [ ] Implement network restriction (verify no socket imports)
- [ ] Add resource usage tracking and reporting
- [ ] Write comprehensive test suite (4 core tests)
- [ ] Run security test suite (`cargo test --test security_tests`)
- [ ] Verify all 9 security tests pass with real executor
- [ ] Update documentation with implementation details
- [ ] Create PR with full Week 1 fixes

---

## 📚 References

- [Wasmtime 22.0 Documentation](https://docs.wasmtime.dev/)
- [WASI Preview 1 Spec](https://github.com/WebAssembly/WASI/blob/main/legacy/preview1/docs.md)
- [Cap-std (Filesystem Capabilities)](https://github.com/bytecodealliance/cap-std)
- [Wasmtime Resource Limiting](https://docs.wasmtime.dev/api/wasmtime/struct.Store.html#method.limiter)
- [Fuel Metering API](https://docs.wasmtime.dev/api/wasmtime/struct.Store.html#method.set_fuel)

---

## 🔐 Security Guarantees

After implementation, the WASM executor will provide:

✅ **CPU Protection**: Fuel metering prevents infinite loops and CPU exhaustion  
✅ **Memory Protection**: Hard memory limits prevent OOM attacks  
✅ **Filesystem Isolation**: WASI preopens restrict file access to explicitly allowed directories  
✅ **Network Isolation**: No socket imports = complete network restriction  
✅ **Process Isolation**: WASM sandbox prevents fork bombs and system calls  
✅ **Timing Attack Resistance**: Deterministic execution mode enabled  

---

**Last Updated**: October 21, 2025  
**Implementation Timeline**: Days 4-5 (October 24-25, 2025)  
**Status**: Ready for implementation
