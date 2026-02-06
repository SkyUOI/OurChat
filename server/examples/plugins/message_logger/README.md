# Message Logger Plugin

A simple example WASM plugin for OurChat that logs all messages.

## Building

### Prerequisites

1. Add WASM target: `rustup target add wasm32-unknown-unknown`

### Build Steps

```bash
cd examples/plugins/message_logger
cargo build --release --target wasm32-unknown-unknown
```

The compiled `.wasm` file will be at:
`target/wasm32-unknown-unknown/release/message_logger_plugin.wasm` (name might be `libmessage_logger_plugin.wasm` depending on cargo version)

## Installation

1. Copy the `.wasm` file to your OurChat server's `plugins/` directory:

```bash
# Find the actual .wasm file name
ls target/wasm32-unknown-unknown/release/*.wasm

# Copy it (adjust the filename if needed)
cp target/wasm32-unknown-unknown/release/message_logger_plugin.wasm /path/to/ourchat/plugins/
```

2. Enable the plugin system in your OurChat config:

```toml
[plugin]
enabled = true
directory = "plugins"
max_memory_mb = 64
max_execution_time_ms = 100
```

3. Restart the OurChat server

## How It Works

The plugin exports the following functions that the host calls:

### `on_message_send(data_ptr: u32, data_len: u32) -> u32`

Called before a message is sent.
- **Parameters**:
  - `data_ptr`: Pointer to message data in WASM memory
  - `data_len`: Length of the message data
- **Returns**:
  - `0`: Allow the message to be sent
  - `1`: Block the message

### `on_plugin_load()`

Called when the plugin is first loaded. Use for initialization.

### `on_plugin_unload()`

Called when the plugin is unloaded. Use for cleanup.

## Plugin Development

### Function Signature

All hook functions follow this pattern:

```rust
#[no_mangle]
pub extern "C" fn hook_name(param1: u32, param2: u32) -> u32 {
    // Your code here
    0 // Return 0 to continue, 1 to stop
}
```

### Memory Access

The plugin can access its own linear memory using raw pointers:

```rust
#[no_mangle]
pub extern "C" fn on_message_send(data_ptr: u32, data_len: u32) -> u32 {
    // Access message data
    let data = unsafe {
        std::slice::from_raw_parts(data_ptr as *const u8, data_len as usize)
    };

    // Process the data
    // ...

    0 // Allow message
}
```

### Host Functions

Plugins can call host functions (future feature):

```rust
// Call host log function (will be implemented)
extern "C" {
    fn ourchat_log(level: u32, msg_ptr: u32, msg_len: u32);
}
```

## Testing

1. Start OurChat with the plugin enabled
2. Send a message through any OurChat client
3. Check the server logs for plugin activity
4. The plugin should log message sends

## Troubleshooting

### Plugin not loading

- Check the `.wasm` file is in the correct directory
- Check server logs for error messages
- Ensure the plugin exports the required functions

### Plugin crashes

- Check server logs for panic messages
- Ensure memory access is within bounds
- Use `unsafe` blocks carefully

### Build errors

- Ensure you have the `wasm32-unknown-unknown` target installed
- Check Rust version: `rustc --version`
