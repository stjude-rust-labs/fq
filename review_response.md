# Review Response

## Fixed

**#6 — Skip-ahead always starts at record 0 (biased sampling)**
Fixed in PR #56. `skip_ahead_sample()` now draws the first exponential jump before selecting any record, so the starting position is randomized.
Commit: `34b771b` on `feat/indexed-reader`

**#8 — `.bgz` extension inconsistency**
Fixed in both PRs. Removed `.bgz` from `is_gzipped()` so it matches `fastq::fs` which only recognizes `.gz`.
Commits: `34b771b` on `feat/indexed-reader`, `b9ad3c6` on `feat/subsample-by-tile`

**#10 — `--seed` does not guarantee reproducible tile-binned output**
Fixed in both PRs. Tiles are sorted by `bin_key` before processing. Per-tile RNG is derived from `base_seed + bin_key` instead of drawn sequentially from a shared mutex. Output is now deterministic regardless of HashMap iteration order or thread scheduling.
Commits: `34b771b` on `feat/indexed-reader`, `b9ad3c6` on `feat/subsample-by-tile`

**#11 — Temp file creation panics via `unwrap()`**
Fixed in both PRs. `File::create()` errors in `write_tile_temp_files()` now propagate as `SubsampleError::CreateFile` instead of panicking.
Commits: `34b771b` on `feat/indexed-reader`, `b9ad3c6` on `feat/subsample-by-tile`

## Already fixed in prior round

**#6 (original) — Empty-input crashes `Uniform::new(0, 0)`**
Fixed previously. `subsample_exact` returns early with empty output when `actual_record_count == 0`.

**#9 (original) — Stale `.fai` index trusted blindly**
Fixed previously for uncompressed input (cross-checked against line count). Gzipped input trusts the index with an explicit log message — cross-checking would require full decompression.

## Already filed as GitHub issues (pre-existing, out of scope)

| Finding | Issue |
|---------|-------|
| #1 — Single-end lint skips S007 | [#47](https://github.com/stjude-rust-labs/fq/issues/47) |
| #2 — Paired lint S007 only checks r1 | [#48](https://github.com/stjude-rust-labs/fq/issues/48) |
| #3 — `filter --names` normalization | [#49](https://github.com/stjude-rust-labs/fq/issues/49) |
| #4 — `filter` mismatched src/dst truncation | [#50](https://github.com/stjude-rust-labs/fq/issues/50) |
| #5 — `filter` pass-through bypasses gzip | [#51](https://github.com/stjude-rust-labs/fq/issues/51) |
| #12 — `describe` sentinel on empty input | [#52](https://github.com/stjude-rust-labs/fq/issues/52) |
| #13 — `AsciiChar` accepts invalid args | [#53](https://github.com/stjude-rust-labs/fq/issues/53) |
| #14 — Validators invert expected/got | [#55](https://github.com/stjude-rust-labs/fq/issues/55) |

## Not fixing

**#7 — EOF-without-trailing-newline inconsistency**
Valid but pre-existing inconsistency between the streaming reader and `RecordIndex`. Both behaviors are wrong but fixing the streaming reader is a separate change. Covered by [#54](https://github.com/stjude-rust-labs/fq/issues/54).

**#9 (new) — Gzipped `.fai` indexes still trusted blindly**
Known tradeoff. Cross-checking gzipped input requires full decompression, negating the index benefit. The current behavior is logged explicitly. Adding a file-size heuristic check could be a future improvement.

**#15 — `subsample_by_tile()` complexity (9-argument signature)**
Valid readability concern but not a correctness issue. A config struct refactor would help but is better done as a standalone cleanup.
