# Changelog

## [0.1.5](https://github.com/mpecan/cargo-lint-extra/compare/cargo-lint-extra-v0.1.4...cargo-lint-extra-v0.1.5) (2026-03-14)


### Features

* add aarch64-unknown-linux-gnu release target ([7c8298b](https://github.com/mpecan/cargo-lint-extra/commit/7c8298b6b0319a7bd1bb10a96c371f65d5c66a48))
* add clone-density AST rule ([3b0dfde](https://github.com/mpecan/cargo-lint-extra/commit/3b0dfde1eb96b567ae06e25449451ca5722b1af4)), closes [#6](https://github.com/mpecan/cargo-lint-extra/issues/6)
* add clone-density AST rule ([#22](https://github.com/mpecan/cargo-lint-extra/issues/22)) ([1ad375d](https://github.com/mpecan/cargo-lint-extra/commit/1ad375d961a9b5fcf1dff26d6cad733515ff48de))
* add glob-imports AST rule ([#23](https://github.com/mpecan/cargo-lint-extra/issues/23)) ([bc28012](https://github.com/mpecan/cargo-lint-extra/commit/bc28012635273f9399de8fc391ef588613f60e97))
* add glob-imports AST rule ([#9](https://github.com/mpecan/cargo-lint-extra/issues/9)) ([f59ead5](https://github.com/mpecan/cargo-lint-extra/commit/f59ead5d18c1d0abd112643c9d4a5eb164d39188))
* add magic-numbers AST rule ([#8](https://github.com/mpecan/cargo-lint-extra/issues/8)) ([5187fc4](https://github.com/mpecan/cargo-lint-extra/commit/5187fc4b2523c7b3e5969bb8fa609b1adbd480e4))
* add magic-numbers AST rule ([#8](https://github.com/mpecan/cargo-lint-extra/issues/8)) ([#25](https://github.com/mpecan/cargo-lint-extra/issues/25)) ([dd1b8a6](https://github.com/mpecan/cargo-lint-extra/commit/dd1b8a6246d003edfc8ea522daba4bea47b561c4))
* add redundant-comments rule ([#10](https://github.com/mpecan/cargo-lint-extra/issues/10)) ([8c351c5](https://github.com/mpecan/cargo-lint-extra/commit/8c351c58ab6bd6170d5224c9717a6729fcb59532))
* add redundant-comments rule ([#21](https://github.com/mpecan/cargo-lint-extra/issues/21)) ([8cdcc23](https://github.com/mpecan/cargo-lint-extra/commit/8cdcc2335e5e3a8373f54943b0ecc70aa96b32df))
* add undocumented-panic AST rule ([#7](https://github.com/mpecan/cargo-lint-extra/issues/7)) ([ec04e91](https://github.com/mpecan/cargo-lint-extra/commit/ec04e9163242115257311961be85bc81bf171563))
* add undocumented-panic AST rule ([#7](https://github.com/mpecan/cargo-lint-extra/issues/7)) ([#26](https://github.com/mpecan/cargo-lint-extra/issues/26)) ([c83c29f](https://github.com/mpecan/cargo-lint-extra/commit/c83c29fd6c321723fdb8064efc1788c732606fc8))


### Bug Fixes

* address Copilot PR feedback for clone-density rule ([f41b96d](https://github.com/mpecan/cargo-lint-extra/commit/f41b96d6a855d6176a3f86bf36ba732e9cbfd7f9))
* address Copilot review feedback for magic-numbers rule ([ee5ac49](https://github.com/mpecan/cargo-lint-extra/commit/ee5ac49dc0f9e2cd810c7a5c767def888684e9ad))
* address PR review feedback for redundant-comments rule ([fee88cf](https://github.com/mpecan/cargo-lint-extra/commit/fee88cfac4dc79d350d13f02391ac7b4ca046d36))
* use tempfile::TempDir for race-free integration tests ([618de80](https://github.com/mpecan/cargo-lint-extra/commit/618de806febfe5663f6cb5e541aee45e96130745))


### Code Refactoring

* make lint rules independent via declare_rules! macro ([32c03da](https://github.com/mpecan/cargo-lint-extra/commit/32c03da5c922906a5ad48b49de8346450cebfc1b))
* make lint rules independent via declare_rules! macro ([#24](https://github.com/mpecan/cargo-lint-extra/issues/24)) ([9763174](https://github.com/mpecan/cargo-lint-extra/commit/9763174f46b5283dc175db97ed45ab85109de37d))

## [0.1.4](https://github.com/mpecan/cargo-lint-extra/compare/cargo-lint-extra-v0.1.3...cargo-lint-extra-v0.1.4) (2026-03-12)


### Features

* add cargo-binstall metadata for prebuilt binary installation ([34c7b44](https://github.com/mpecan/cargo-lint-extra/commit/34c7b4415d801ecff6fbfc97de5bbceb2cf4f393))
* exit 0 for warnings, exit 1 for errors, add -W flag ([72386f5](https://github.com/mpecan/cargo-lint-extra/commit/72386f52008fff52b1cb6c3d56203b1de9f52036))


### Bug Fixes

* level=deny promotes soft violations, deprecation warning, docs ([a4fb8d6](https://github.com/mpecan/cargo-lint-extra/commit/a4fb8d675d2561c40545e2b30c20fb74642fa0e2))

## [0.1.3](https://github.com/mpecan/cargo-lint-extra/compare/cargo-lint-extra-v0.1.2...cargo-lint-extra-v0.1.3) (2026-03-12)


### Features

* add soft/hard limits to file-length rule ([e9a6d80](https://github.com/mpecan/cargo-lint-extra/commit/e9a6d800d6f8b0505e0c56cd8a2ffb50e8c6c133))

## [0.1.2](https://github.com/mpecan/cargo-lint-extra/compare/cargo-lint-extra-v0.1.1...cargo-lint-extra-v0.1.2) (2026-03-12)


### Features

* add comment-based inline suppression for individual lines, blocks, and functions ([c26ee5d](https://github.com/mpecan/cargo-lint-extra/commit/c26ee5d4feaeff5e3d8f7221ecb748a6e144090c))
* add test-code-specific rule overrides via [test] config section ([0230309](https://github.com/mpecan/cargo-lint-extra/commit/02303095a5746761264b5913639963f8bc8cd923))


### Bug Fixes

* add cross-compilation targets to rust-toolchain.toml ([ffffb7d](https://github.com/mpecan/cargo-lint-extra/commit/ffffb7db9ccfb1bcb8bcb107d0f1187f8e5ec766))


### Documentation

* add CLAUDE.md with project constitution and conventions ([c51f9fe](https://github.com/mpecan/cargo-lint-extra/commit/c51f9fe357d178ce6d193138616149ee561e83d3))

## [0.1.1](https://github.com/mpecan/cargo-lint-extra/compare/cargo-lint-extra-v0.1.0...cargo-lint-extra-v0.1.1) (2026-03-06)


### Features

* add inline-comments rule ([878f24c](https://github.com/mpecan/cargo-lint-extra/commit/878f24c747f6646c9518c60235a8ad439fb4526c))
* initial implementation of cargo-lint-extra ([e173088](https://github.com/mpecan/cargo-lint-extra/commit/e173088b66bd553adb72699029a382db06f7d144))


### Documentation

* add LICENSE, README, and CONTRIBUTING guide ([317ffb7](https://github.com/mpecan/cargo-lint-extra/commit/317ffb771edfe8af7b0b770b8134418d09de4520))


### Code Refactoring

* always exclude target/ directory from linting ([9ac7bbb](https://github.com/mpecan/cargo-lint-extra/commit/9ac7bbb02e5df50d29977da006cf0fcc5630ee84))
