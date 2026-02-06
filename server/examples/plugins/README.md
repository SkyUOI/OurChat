# OurChat Plugin System

A secure, sandboxed plugin system using WebAssembly (WASM) for extending OurChat server functionality.

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                    OurChat Server                           │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  ┌──────────────┐        ┌─────────────────────────┐       │
│  │ RpcServer    │───────▶│ PluginManager           │       │
│  │              │        │  - Load plugins         │       │
│  │ plugin_mgr   │        │  - Execute hooks        │       │
│  └──────────────┘        │  - Registry             │       │
│                          └───────────┬─────────────┘       │
│                                      │                      │
│                                      ▼                      │
│                          ┌──────────────────────┐          │
│                          │   WasmEngine         │          │
│                          │   - Wasmtime runtime  │          │
│                          │   - Sandboxing        │          │
│                          └───────────┬──────────┘          │
│                                      │                      │
│                          ┌───────────▼──────────┐          │
│                          │  Loaded Plugins      │          │
│                          │  ┌────────────────┐  │          │
│                          │  │ plugin1.wasm   │  │          │
│                          │  │ plugin2.wasm   │  │          │
│                          │  └────────────────┘  │          │
│                          └──────────────────────┘          │
│                                                               │
└─────────────────────────────────────────────────────────────┘
```

## Features

- **WASM-based**: Plugins compiled to WebAssembly run in a secure sandbox
- **Language Agnostic**: Write plugins in Rust, JavaScript/TypeScript, Go, or any WASM-compatible language
- **Hook System**: Plugins can hook into message sending, user events, and more
- **Resource Limits**: Configurable memory and execution time limits prevent abuse
- **Hot Loading**: Load/unload plugins without restarting the server (planned)

## Hook Types

### Message Hooks

- **`pre_message_send`**: Called before a message is sent. Can:
  - Validate and block messages
  - Modify message content (planned)
  - Add metadata/headers

- **`post_message_send`**: Called after successful message delivery. Can:
  - Log to external systems
  - Trigger notifications
  - Update statistics

### Event Hooks (Planned)

- `on_user_created`: Called when a new user registers
- `on_user_login`: Called when a user authenticates
- `on_friend_added`: Called when a friendship is created
- `on_session_created`: Called when a new chat session is created

## Host API

Plugins can call these host functions:

| Function | Description |
|----------|-------------|
| `ourchat.log(level, msg_ptr, msg_len)` | Log messages (0=trace, 1=debug, 2=info, 3=warn, 4=error) |
| `ourchat.get_config(key_ptr, key_len)` | Get plugin configuration value |
| `ourchat.set_config(key_ptr, key_len, val_ptr, val_len)` | Set plugin configuration value |
| `ourchat.emit_event(event_type, data_ptr, data_len)` | Emit plugin event |

## Example: Message Filter Plugin

```rust
// Block messages containing spam keywords
#[no_mangle]
pub extern "C" fn on_message_send(
    ctx_ptr: *mut Context,
    msg_ptr: u32,
    msg_len: u32
) -> u32 {
    // Read message from WASM memory
    let msg = read_message(ctx_ptr, msg_ptr, msg_len);

    // Check for spam
    if contains_spam(&msg) {
        // Log the blocked message
        host_log(2, b"Blocked spam message");
        return 1; // Block the message
    }

    0 // Allow the message
}
```

## Example: Message Logging Plugin

```rust
// Log all messages to external service
#[no_mangle]
pub extern "C" fn on_message_sent(
    ctx_ptr: *mut Context,
    msg_ptr: u32,
    msg_len: u32
) {
    let msg = read_message(ctx_ptr, msg_ptr, msg_len);

    // Send to external logging service
    host_log(2, format!("Message sent: {}", msg).as_bytes());

    // Optionally call external API
    // host_http_post("https://logs.example.com/api", msg);
}
```

## Building a Plugin

### Using Rust

1. Create a new library project:
```bash
cargo new --lib my_plugin
cd my_plugin
```

2. Configure `Cargo.toml`:
```toml
[package]
name = "my_plugin"
version = "0.1.0"
edition = "2024"

[lib]
crate-type = ["cdylib"]

[dependencies]
# Add dependencies as needed
```

3. Write your plugin in `src/lib.rs` (see examples above)

4. Build for WASM:
```bash
rustup target add wasm32-unknown-unknown
cargo build --release --target wasm32-unknown-unknown
```

5. Copy the output:
```bash
cp target/wasm32-unknown-unknown/release/my_plugin.wasm /path/to/ourchat/plugins/
```

For Docker deployments, copy to `docker/plugins/` instead:
```bash
cp target/wasm32-unknown-unknown/release/my_plugin.wasm /path/to/ourchat/docker/plugins/
```

## Plugin Discovery

The server automatically discovers `.wasm` files in the configured `plugin.directory`. Each plugin is:

1. Loaded into memory
2. Validated (WASM signature, metadata)
3. Initialized (optional `on_plugin_load` function)
4. Registered in the plugin registry

## Security

- **Sandboxing**: Each plugin runs in an isolated WASM sandbox
- **No Direct Memory Access**: Plugins cannot access host memory directly
- **Controlled API**: Only approved host functions are available
- **Resource Limits**: Memory and execution time are strictly limited
- **Fail-Safe**: Plugin errors don't crash the server

## Troubleshooting

### Plugin Not Loading

Check the server logs for error messages:
```
Failed to load plugin plugin_name.wasm: ...
```

Common issues:
- Invalid WASM format
- Missing required exports
- Exceeds resource limits

### Plugin Hook Not Called

Ensure:
1. Plugin is loaded successfully (check logs)
2. Plugin exports the expected hook functions
3. Plugin state is `Enabled` (not `Disabled` or `Failed`)

### Plugin Performance Issues

Adjust resource limits in config:
```toml
[plugin]
max_memory_mb = 128              # Increase if needed
max_execution_time_ms = 200      # Increase for slow plugins
```

## Current Limitations

- No hot-reload (requires server restart to load new plugins)
- Limited host API (database, Redis, HTTP client planned)
- No inter-plugin communication

## Future Enhancements

- [ ] Web UI for plugin management
- [ ] Plugin marketplace
- [ ] Hot-reload support
- [ ] Database/Redis access from plugins
- [ ] HTTP client for external API calls
- [ ] Plugin permissions system
- [ ] Inter-plugin communication
- [ ] Plugin development SDK

## Contributing

Plugin contributions are welcome! Please submit:

1. Source code
2. Compiled `.wasm` binary
3. Documentation
4. Example usage

See `examples/plugins/` for reference implementations.
