# Changelog

All breaking changes after 1.0 will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## 2.0.0 [Unreleased]

### Added

- Alphabet constants are provided in the `alphabet` module.

### Changed

- `TextNonce::sized_configured()` has been replaced with `TextNonce::custom()`
  which takes an alphabet string rather than a `base64::Config`.
