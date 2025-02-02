# xelf

A versatile Rust toolkit providing extensive utilities for async programming, database operations, FFI, and more.

## Features

This crate provides several feature-gated modules:

### Minimal Features (Default)

- `datetime` - Chronological utilities using `chrono`
- `collections` - Enhanced collection types using `smallvec` and more
- `io` - I/O utilities
- `derive` - Helpful derive macros

### Common Features

- Async programming utilities (tokio-based)
- Byte handling and manipulation
- File system operations
- JSON processing
- Network programming
- Signal handling
- String manipulation
- Synchronization primitives

### Full Features

- Database support (sea-orm, sqlx)
- FFI utilities
- All common features

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
xelf = "0.4.6"  # Default features
```

With specific features:

```toml
[dependencies]
xelf = { version = "0.4.6", features = ["full"] }  # All features
# or
xelf = { version = "0.4.6", features = ["async", "db"] }  # Selected features
```

## Feature Flags

- `minimal` (default): Basic utilities
- `common`: Most commonly used features
- `full`: All features including database and FFI
- Individual features like `async`, `db`, `ffi`, etc.

See Cargo.toml for complete feature list.

## Requirements

- Rust 1.77 or later

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Author

Sprite Tong (<spritetong@gmail.com>)
