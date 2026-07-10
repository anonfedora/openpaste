//! Plugin manager — loads WASM plugins and applies them to clipboard items.
//!
//! Plugin contract (guest side):
//!   - Export: `process_item(ptr: i32, len: i32) -> i32`
//!     Receives a UTF-8 string (the clipboard text content) at [ptr..ptr+len]
//!     in the guest's linear memory.
//!     Return value: byte length of the result string written at ptr.
//!     Return 0 or negative to leave the content unchanged.
//!
//!   - The host provides one import:
//!       `env::log(ptr: i32, len: i32)` — write a log message.
//!
//! Plugins receive ONLY plain-text content. Encrypted and image items are
//! skipped automatically.

use crate::PluginError;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tracing::{info, warn};
use wasmtime::{Engine, Linker, Module, Store};

// ── Plugin record ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct PluginInfo {
    pub name: String,
    pub path: PathBuf,
    pub enabled: bool,
}

// ── Per-call state threaded through Store ─────────────────────────────────────

struct HostState {
    plugin_name: String,
}

// ── Manager ───────────────────────────────────────────────────────────────────

pub struct PluginManager {
    engine: Engine,
    plugins: Arc<Mutex<HashMap<String, (PluginInfo, Module)>>>,
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            engine: Engine::default(),
            plugins: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Load (or reload) a `.wasm` plugin from disk.
    /// The plugin name is the file stem (e.g. `"dedupe"` for `dedupe.wasm`).
    pub fn load(&self, path: &Path) -> Result<String, PluginError> {
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| PluginError::LoadFailed("Invalid file name".to_string()))?
            .to_string();

        let module = Module::from_file(&self.engine, path)
            .map_err(|e| PluginError::LoadFailed(e.to_string()))?;

        let info = PluginInfo {
            name: name.clone(),
            path: path.to_path_buf(),
            enabled: true,
        };

        self.plugins
            .lock()
            .unwrap()
            .insert(name.clone(), (info, module));

        info!("Plugin loaded: {}", name);
        Ok(name)
    }

    /// Unload a plugin by name.
    pub fn unload(&self, name: &str) -> Result<(), PluginError> {
        self.plugins.lock().unwrap().remove(name);
        info!("Plugin unloaded: {}", name);
        Ok(())
    }

    /// List all loaded plugins.
    pub fn list(&self) -> Vec<PluginInfo> {
        self.plugins
            .lock()
            .unwrap()
            .values()
            .map(|(info, _)| info.clone())
            .collect()
    }

    /// Run all enabled plugins against a text clipboard item.
    /// Returns the (possibly transformed) content.
    /// If any plugin errors, it is logged and skipped — the original (or last
    /// successful) content is passed to the next plugin.
    pub fn process(&self, content: &str, content_type: &str) -> String {
        // Only process plain text — skip images, encrypted items, etc.
        if content_type != "text" && content_type != "code" && content_type != "html" {
            return content.to_string();
        }

        let plugins: Vec<(String, Module)> = {
            let guard = self.plugins.lock().unwrap();
            guard
                .values()
                .filter(|(info, _)| info.enabled)
                .map(|(info, module)| (info.name.clone(), module.clone()))
                .collect()
        };

        let mut current = content.to_string();

        for (name, module) in plugins {
            match self.run_plugin(&name, &module, &current) {
                Ok(Some(transformed)) => {
                    info!("Plugin '{}' transformed content ({} → {} bytes)", name, current.len(), transformed.len());
                    current = transformed;
                }
                Ok(None) => {} // plugin chose not to transform
                Err(e) => {
                    warn!("Plugin '{}' failed: {}", name, e);
                }
            }
        }

        current
    }

    // ── Internal: run one plugin ──────────────────────────────────────────────

    fn run_plugin(
        &self,
        name: &str,
        module: &Module,
        content: &str,
    ) -> Result<Option<String>, PluginError> {
        let mut linker: Linker<HostState> = Linker::new(&self.engine);
        let plugin_name = name.to_string();

        // Provide `env::log` import
        // plugin_name is captured by reference via an Arc-like clone so the
        // closure stays Fn (not FnOnce).
        let plugin_name_log = plugin_name.to_string();
        linker
            .func_wrap(
                "env",
                "log",
                move |mut caller: wasmtime::Caller<'_, HostState>, ptr: i32, len: i32| {
                    let _ = &plugin_name_log; // keep capture alive without moving out
                    if let Some(mem) = caller.get_export("memory") {
                        if let Some(mem) = mem.into_memory() {
                            let data = mem.data(&caller);
                            let ptr = ptr as usize;
                            let len = len as usize;
                            if ptr + len <= data.len() {
                                if let Ok(msg) = std::str::from_utf8(&data[ptr..ptr + len]) {
                                    info!("[plugin:{}] {}", caller.data().plugin_name, msg);
                                }
                            }
                        }
                    }
                },
            )
            .map_err(|e| PluginError::ExecutionFailed(e.to_string()))?;

        let mut store = Store::new(
            &self.engine,
            HostState {
                plugin_name: name.to_string(),
            },
        );

        let instance = linker
            .instantiate(&mut store, module)
            .map_err(|e| PluginError::ExecutionFailed(format!("instantiate: {}", e)))?;

        // Get memory export
        let memory = instance
            .get_memory(&mut store, "memory")
            .ok_or_else(|| PluginError::ExecutionFailed("No memory export".to_string()))?;

        // Get process_item export
        let process_fn = instance
            .get_typed_func::<(i32, i32), i32>(&mut store, "process_item")
            .map_err(|e| PluginError::ExecutionFailed(format!("process_item: {}", e)))?;

        // Write content into guest memory at offset 0
        let content_bytes = content.as_bytes();
        if content_bytes.len() > 64 * 1024 {
            return Err(PluginError::ExecutionFailed(
                "Content too large for plugin (>64 KB)".to_string(),
            ));
        }

        // Grow memory if needed (1 page = 64 KB minimum)
        let needed_pages = (content_bytes.len() / 65536) + 1;
        let current_pages = memory.size(&store) as usize;
        if needed_pages > current_pages {
            memory
                .grow(&mut store, (needed_pages - current_pages) as u64)
                .map_err(|e| PluginError::ExecutionFailed(format!("grow: {}", e)))?;
        }

        memory
            .write(&mut store, 0, content_bytes)
            .map_err(|e| PluginError::ExecutionFailed(format!("write: {}", e)))?;

        let result_len = process_fn
            .call(&mut store, (0, content_bytes.len() as i32))
            .map_err(|e| PluginError::ExecutionFailed(format!("call: {}", e)))?;

        if result_len <= 0 {
            return Ok(None); // no transformation
        }

        // Read back the result from memory offset 0
        let result_bytes: Vec<u8> = memory.data(&store)[..result_len as usize].to_vec();
        let result = String::from_utf8(result_bytes)
            .map_err(|e| PluginError::ExecutionFailed(format!("utf8: {}", e)))?;

        Ok(Some(result))
    }
}
