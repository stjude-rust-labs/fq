# Release

  * [ ] Update version in `Cargo.toml`.
  * [ ] Update `CHANGELOG.md` with version and publication date.
  * [ ] Update `README.md` with version.
  * [ ] Run tests: `cargo test`.
  * [ ] Stage changes: `git add Cargo.lock Cargo.toml CHANGELOG.md README.md`.
  * [ ] Create git commit: `git commit -m "Bump version to $VERSION"`.
  * [ ] Create git tag: `git tag -m "" -a v$VERSION`.
  * [ ] Push release: `git push --follow-tags`.
