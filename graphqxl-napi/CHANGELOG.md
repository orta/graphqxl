# Changelog

## [Unreleased]

### Added

- Initial implementation of Node.js bindings for GraphQXL
- Support for Virtual File System (VFS) based compilation
- `graphqxlToSdl` function that accepts:
  - VFS object (key-value pairs of file paths and contents)
  - Entry path
  - Optional indent spaces (default: 2)
  - Optional private prefix (default: "\_")
- Full support for GraphQXL features:
  - Basic GraphQL types (type, input, enum, interface, union, scalar)
  - Import system for modular schemas
  - Generic types with type parameters
  - Type inheritance and composition
  - Schema extensions
  - Private fields
  - Directives
- TypeScript definitions
- Comprehensive test suite
- Examples in both JavaScript and TypeScript

### Technical Details

- Built using NAPI-RS for Rust/Node.js bindings
- Implements the VirtualFileSystem trait for in-memory file operations
- No file system access required - all operations are in-memory
- Cross-platform support for multiple architectures
