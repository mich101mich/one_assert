# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).


## [0.2.0] - 2025-10-31

### Changed
- Overhauled several panic messages to make them less cluttered.
  - This also removes several steps of the condition analysis, so it might be less useful overall.
    Though arguably, assert conditions shouldn't reach that level of complexity to begin with...
- Allowed some conditions to only format their arguments on panic when it is known they won't be moved.
  - This will improve the performance in 95% of intended use cases, meaning this assert macro is now
    actually viable for production code rather than just test code.

[0.2.0]: https://github.com/mich101mich/one_assert/releases/tag/0.2.0

## [0.1.1] - 2025-02-14

### Added
- Added a changelog

### Fixed
- Removed unneeded dependencies

[0.1.1]: https://github.com/mich101mich/one_assert/releases/tag/0.1.1

## [0.1.0] - 2024-07-15

### Added
- [`assert`]: macro to make runtime assertions

[0.1.0]:    https://github.com/mich101mich/one_assert/releases/tag/0.1.0
[`assert`]: https://docs.rs/one_assert/0.1.0/one_assert/macro.assert.html
