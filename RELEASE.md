# Release

  * [ ] Update version in `Cargo.toml`.
  * [ ] Update `CHANGELOG.md` with version and publication date.
  * [ ] Run tests: `cargo test`.
  * [ ] Stage changes: `git add Cargo.toml CHANGELOG.md`.
  * [ ] Create git commit: `git commit -m "Bump version to $VERSION"`.
  * [ ] Create git tag: `git tag -m "" -a v$VERSION`.
  * [ ] Push release: `git push --follow-tags`.
