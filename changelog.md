# Changelog


## 2026-07-27

### Fixed
- read_until_delimiter uses a buffer correctly and only deserializes the required slice, tests pass now 

### Changed
- command_to_string requires a flag for if the command will return zero bytes 

### Added
- framework for networking code testing
- subtrait of read + write: IOStream. made for allowing generics for duplexes 

## 2026-07-27

### Added
- subslice_contains utility function added to aide in finding delimiters for networking
- cmd_to_string utility 
- abstracted networking code 
- machine fingerprinting in init.rs
