# Layer 4 WASM Runtime Integration Plan

**Date**: October 21, 2025  
**Status**: 🚧 **CRITICAL PATH ITEM**  
**Priority**: P0 - Blocks 4 security tests

---

## 🎯 Objective

Integrate Wasmtime WASM runtime with Layer 4 Executor to enable:
1. ✅ Resource quota enforcement (CPU, memory, time)
2. ✅ Sandbox isolation validation
3. ✅ Security test completion (9/9 passing)

**Current Blocker**: Security tests 6-9 fail because `execute_task_with_quota()` returns stub error

---

## 📋 Implementation Checklist

### Phase 1: Basic WASM Execution ⏳

#### 1.1 Create WASM Executor Module
```rust
// src/layer4/src/wasm_executor.rs

use wasmtime::*;
use crate::types::*;

pub struct WasmExecutor {
    engine: Engine,
    linker: Linker<()>,
    store: Store<()>,
}

impl WasmExecutor {
    pub fn new() -> Layer4Result<Self> {
        let mut config = Config::new();
        config.wasm_component_model(true);
        config.async_support(true);
        
        let engine = Engine::new(&config)?;
        let linker = Linker::new(&engine);
        let store = Store::new(&engine, ());
        
        Ok(Self { engine, linker, store })
    }
    
    pub async fn execute_wasm(
        &mut self,
        wasm_bytes: &[u8],
        task: Task,
    ) -> Layer4Result<ExecutionResult> {
        // 1. Compile WASM module
        let module = Module::new(&self.engine, wasm_bytes)?;
        
        // 2. Instantiate with linker
        let instance = self.linker.instantiate_async(&mut self.store, &module).await?;
        
        // 3. Get main function
        let main_func = instance
            .get_typed_func::<(), ()>(&mut self.store, "execute_task")?;
        
        // 4. Execute
        main_func.call_async(&mut self.store, ()).await?;
        
        Ok(ExecutionResult {
            task_id: task.id,
            success: true,
            output: serde_json::json!({"status": "completed"}),
            execution_time_ms: 0,
            resource_usage: ResourceUsage::default(),
            error: None,
            completed_at: SystemTime::now(),
        })
    }
}
```

**Estimated Time**: 2-3 hours

---

### Phase 2: Resource Quota Enforcement 🔒

#### 2.1 CPU Time Limits
```rust
impl WasmExecutor {
    pub fn set_cpu_limit(&mut self, max_cpu_seconds: f64) {
        self.store.set_epoch_deadline(
            (max_cpu_seconds * 1_000_000.0) as u64
        );
    }
}
```

#### 2.2 Memory Limits
```rust
let mut config = Config::new();
config.max_wasm_stack(1024 * 1024); // 1MB stack
config.static_memory_maximum_size(64 * 1024 * 1024); // 64MB max

// Per-instance limit
store.limiter(|_| {
    StoreLimits::new()
        .memory_size(quota.max_memory_mb as usize * 1024 * 1024)
});
```

#### 2.3 Execution Timeout
```rust
use tokio::time::timeout;

let result = timeout(
    Duration::from_secs(quota.max_execution_time_secs),
    main_func.call_async(&mut store, ())
).await;

match result {
    Ok(Ok(_)) => { /* success */ },
    Ok(Err(e)) => { /* WASM error */ },
    Err(_) => Err(Layer4Error::AgentTimeout(quota.max_execution_time_secs)),
}
```

**Estimated Time**: 3-4 hours

---

### Phase 3: Sandbox Isolation 🔐

#### 3.1 WASI Configuration (Restrictive)
```rust
use wasmtime_wasi::WasiCtxBuilder;

let wasi = WasiCtxBuilder::new()
    .inherit_stdio()
    // NO filesystem access
    // NO network access
    // NO environment variables
    .build();

linker.wasi()?;
store.data_mut().wasi = wasi;
```

#### 3.2 Import Restrictions
```rust
// Only allow specific imports
linker.func_wrap("env", "log", |msg: i32| {
    println!("WASM log: {}", msg);
})?;

// Block dangerous imports
// - fs::open, fs::read, fs::write
// - net::connect, net::send
// - process::spawn
```

**Estimated Time**: 2-3 hours

---

### Phase 4: Security Test Integration ✅

#### 4.1 Update Helper Function
```rust
// tests/security_tests.rs

async fn execute_task_with_quota(task: Task) -> Layer4Result<ExecutionResult> {
    let mut executor = WasmExecutor::new()?;
    
    // Apply resource quotas
    executor.set_cpu_limit(task.resource_quota.max_cpu_cores as f64);
    executor.set_memory_limit(task.resource_quota.max_memory_mb);
    
    // Load WASM from test artifacts
    let wasm_bytes = load_test_wasm_for_task(&task)?;
    
    // Execute with timeout
    executor.execute_with_timeout(wasm_bytes, task).await
}
```

#### 4.2 Create Test WASM Artifacts
```wat
;; cpu_bomber.wat - Infinite loop test
(module
  (func $execute_task (export "execute_task")
    (loop $infinite
      (br $infinite)
    )
  )
)

;; memory_bomber.wat - Excessive allocation
(module
  (memory (export "memory") 1000)  ;; 1000 pages = 64MB
  (func $execute_task (export "execute_task")
    ;; Attempt to grow memory beyond limits
    (drop (memory.grow (i32.const 1000)))
  )
)

;; filesystem_probe.wat - Filesystem access attempt
(module
  (import "wasi_snapshot_preview1" "path_open" 
    (func $path_open (param i32 i32 i32 i32 i32 i64 i64 i32 i32) (result i32)))
  (func $execute_task (export "execute_task")
    ;; Try to open /etc/passwd
    (call $path_open 
      (i32.const 0)   ;; dirfd
      (i32.const 0)   ;; dirflags
      (i32.const 0)   ;; path
      (i32.const 12)  ;; path_len
      (i32.const 0)   ;; oflags
      (i64.const 0)   ;; fs_rights_base
      (i64.const 0)   ;; fs_rights_inheriting
      (i32.const 0)   ;; fdflags
      (i32.const 0)   ;; fd (output)
    )
    drop
  )
)
```

**Compile**:
```bash
# Install wasm tools
cargo install wasm-tools

# Compile WAT to WASM
wasm-tools parse cpu_bomber.wat -o tests/wasm/cpu_bomber.wasm
wasm-tools parse memory_bomber.wat -o tests/wasm/memory_bomber.wasm
wasm-tools parse filesystem_probe.wat -o tests/wasm/filesystem_probe.wasm
```

**Estimated Time**: 4-5 hours

---

## 📊 Expected Outcomes

### Before Integration
```
Security Tests: 5/9 passing (55%)
- ✅ Validation tests passing
- ❌ Enforcement tests failing (no runtime)
```

### After Integration
```
Security Tests: 9/9 passing (100%)
- ✅ All validation tests passing
- ✅ All enforcement tests passing
- ✅ Resource quotas enforced
- ✅ Sandbox isolation verified
```

---

## ⏱️ Time Estimate

| Phase | Task | Time | Dependencies |
|-------|------|------|--------------|
| 1 | Basic WASM execution | 2-3h | None |
| 2 | Resource quotas | 3-4h | Phase 1 |
| 3 | Sandbox isolation | 2-3h | Phase 1 |
| 4 | Test integration | 4-5h | Phases 1-3 |
| **Total** | | **11-15 hours** | **~2 days** |

---

## 🚀 Implementation Steps

### Day 1: Core Runtime (6-8 hours)
1. Create `wasm_executor.rs` module
2. Implement basic WASM loading and execution
3. Add resource quota configuration
4. Test with simple WASM module

### Day 2: Security & Testing (5-7 hours)
1. Implement sandbox restrictions
2. Create malicious WASM test artifacts
3. Update security test helpers
4. Run and validate all security tests
5. Document findings and edge cases

---

## 🔍 Testing Strategy

### Unit Tests
```rust
#[test]
fn test_wasm_executor_creation() {
    let executor = WasmExecutor::new();
    assert!(executor.is_ok());
}

#[test]
fn test_cpu_limit_configuration() {
    let mut executor = WasmExecutor::new().unwrap();
    executor.set_cpu_limit(1.0);
    // Verify limit is set
}
```

### Integration Tests
```rust
#[tokio::test]
async fn test_execute_simple_wasm() {
    let mut executor = WasmExecutor::new().unwrap();
    let wasm = include_bytes!("../tests/wasm/hello.wasm");
    let task = create_test_task();
    
    let result = executor.execute_wasm(wasm, task).await;
    assert!(result.is_ok());
}
```

### Security Tests (Existing)
- All 9 tests should pass after integration
- Focus on enforcement verification

---

## 🎯 Success Criteria

- [ ] WasmExecutor module implemented and tested
- [ ] CPU quota enforcement working
- [ ] Memory quota enforcement working
- [ ] Execution timeout working
- [ ] Filesystem isolation verified
- [ ] Network isolation verified
- [ ] All 9 security tests passing
- [ ] Documentation updated
- [ ] Performance benchmarks run successfully

---

## 📝 Implementation Notes

### Wasmtime Configuration Best Practices
```rust
// Recommended production configuration
let mut config = Config::new();
config.wasm_component_model(true);
config.async_support(true);
config.consume_fuel(true);  // For CPU limiting
config.epoch_interruption(true);  // For timeouts
config.max_wasm_stack(1024 * 1024);  // 1MB
config.static_memory_maximum_size(64 * 1024 * 1024);  // 64MB
config.dynamic_memory_guard_size(0);  // No guard pages
config.static_memory_guard_size(0);
```

### Error Handling
```rust
match wasm_error {
    wasmtime::Error::Timeout => Layer4Error::AgentTimeout(duration),
    wasmtime::Error::ResourceLimitExceeded => Layer4Error::ResourceQuotaExceeded(msg),
    wasmtime::Error::Trap(trap) => Layer4Error::WasmRuntime(trap.into()),
    _ => Layer4Error::Internal(format!("WASM error: {}", e)),
}
```

### Performance Considerations
- Compile modules once, instantiate many times
- Use module caching for frequently-used WASM
- Pre-warm WASM store for faster startups
- Monitor JIT compilation overhead

---

## 🔗 References

- [Wasmtime Documentation](https://docs.wasmtime.dev/)
- [WASI Security Model](https://github.com/WebAssembly/WASI/blob/main/legacy/security.md)
- [Wasmtime Resource Limits](https://docs.rs/wasmtime/latest/wasmtime/struct.ResourceLimiter.html)
- [WebAssembly Security](https://webassembly.org/docs/security/)

---

## 📞 Next Actions

### Immediate
1. Create `src/layer4/src/wasm_executor.rs`
2. Implement basic runtime
3. Test with hello world WASM

### Short-term
1. Add resource quotas
2. Configure sandbox
3. Create test WASM artifacts

### Validation
1. Run security tests
2. Verify 9/9 passing
3. Document results

---

**Estimated Completion**: 2 days  
**Blocker Priority**: P0 - Critical  
**Team**: 1 engineer  
**Risk**: Low - well-documented APIs

---

**Status**: Ready to implement  
**Prerequisites**: All met  
**Next**: Begin Phase 1 implementation
