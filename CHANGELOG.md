# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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

[Unreleased]: https://github.com/gonzaponte/h5rio/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/gonzaponte/h5rio/releases/tag/v0.1.0
