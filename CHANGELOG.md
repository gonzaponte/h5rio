# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0] - 2026-06-06

### Added

* Added `iter_array` for reading array datasets entry by entry along the
  leading dimension.
* Added validation for `ArrayHdf5Writer` item shapes, appended array shapes,
  and `write_chunked_array` chunk shapes.
* Added tests for array iteration, scalar dataset rejection, invalid shapes,
  and drop-time flush failure handling.

### Changed

* Changed writer `write()` and `flush()` methods to return
  `hdf5::Result<()>` consistently.
* Changed `#[h5type]` to derive through `::hdf5_metno::H5Type`, removing the
  need for an `hdf5` alias for the macro itself.
* Changed drop-time flushing to warn on pending buffered entries and flush
  failures instead of panicking.

### Fixed

* Fixed public panic paths in `read_array`, `iter_array`, and array writer
  cache dumping by returning `hdf5::Error::Internal` instead.
* Fixed `write_chunked_array` validation so invalid chunk rank and zero chunk
  dimensions are rejected before calling HDF5.
* Updated README documentation to reflect current validation behavior,
  iteration support, and macro expansion.

## [0.1.0] - 2026-06-01

### Added

* Initial public release of `h5rio`.
* Added appendable HDF5 table writer for typed row records.
* Added appendable HDF5 array writer for fixed-shape `ndarray` values.
* Added buffered writing with configurable chunk sizes.
* Added explicit `flush()` support and automatic flushing on drop.
* Added HDF5 table and array reader helpers.
* Added `#[h5type]` attribute macro for defining HDF5-compatible table records.
* Added tests for table writing, array writing, reader helpers, drop-time flushing, and macro expansion.
* Added Nix development environment.
* Added GitHub Actions workflow for building and testing the project.

[Unreleased]: https://github.com/gonzaponte/h5rio/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/gonzaponte/h5rio/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/gonzaponte/h5rio/releases/tag/v0.1.0
