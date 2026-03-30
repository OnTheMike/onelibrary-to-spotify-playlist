---
name: onelibrary-to-spotify-playlist
description: "Use when: Working on the onelibrary-to-spotify-playlist Rust CLI application that syncs OneLibrary XML exports to Spotify playlists. Apply best practices from code review including robust error handling, proper logging, and Rust 2021 edition conventions. Reference the docs/ folder for implementation details, before/after comparisons, and issue analysis."
---

# onelibrary-to-spotify-playlist Development Guidelines

## Project Overview

This is a Rust CLI application that converts OneLibrary XML music library exports into Spotify playlists using the Spotify Web API. The application was recently refactored to fix critical issues and improve code quality.

**Status:** ✅ Production Ready (all 11 issues resolved)

## Key Implementation Standards

### ✅ Error Handling

**Never use `.unwrap()` in production code.** Instead:

```rust
// ❌ WRONG - panics on error
let track_id = TrackId::from_id(&id).unwrap();

// ✅ RIGHT - propagates error gracefully
let track_id = TrackId::from_id(&id)
    .ok_or_else(|| TrackParseError::InvalidSpotifyId(id.clone()))?;
```

Use the custom `TrackParseError` enum defined in `src/onelibrary.rs` for all parsing errors.

### 🎯 Logging

Use `log` crate with `env_logger` for all diagnostic output:

```rust
// ✅ Use structured logging, not println!
log::info!("Processing playlist: {}", playlist_name);
log::warn!("Skipping track with invalid ID: {}", spotify_id);
log::error!("Failed to get Spotify user: {}", err);
```

### 📦 Dependencies

- **roxmltree**: XML parsing
- **rspotify**: Spotify API integration
- **tokio**: Async runtime
- **axum**: Web framework (if needed)
- **chrono**: Date handling

Do NOT add unused dependencies to Cargo.toml. Verify all imports are actually used in code.

### 🧪 Testing

Maintain test coverage around 60%. Test patterns are in `src/lib.rs`:

- Unit tests for parsing logic
- Error case validation
- Date parsing edge cases
- Duplicate detection performance (O(1) lookup with HashSet)

### 📋 Code Organization

From `src/`:
- **main.rs**: CLI entry point, argument parsing
- **lib.rs**: Core library with public API and tests
- **onelibrary.rs**: OneLibrary XML parsing and Track struct (with custom error handling)
- **spotify_auth.rs**: Spotify authentication flow

## Performance Improvements

- Duplicate detection changed from O(n²) to O(1) using HashSet
- Improves from ~80 lines to ~40 lines of core logic
- No external lookups or nested iterations

## Important References

See `docs/` folder for detailed information:

- **EXECUTIVE_SUMMARY.md**: Quick status overview and project metrics
- **CODE_REVIEW.md**: Details on all 11 issues identified and severity levels
- **IMPLEMENTATION_GUIDE.md**: Full refactored code examples
- **BEFORE_AND_AFTER.md**: Side-by-side code comparisons showing improvements
- **INDEX.md**: Navigation guide to all documentation

## Rust Edition & Setup

- **Edition:** 2021
- **MSRV:** 1.70+ (verify with `cargo +1.70 check`)
- **Build:** `cargo build --release`
- **Test:** `cargo test`
- **Run:** `./target/release/onelibrary-to-spotify-playlist -f export.xml`

## When Suggesting Changes

1. **Reference the code review findings** from `docs/CODE_REVIEW.md` to maintain consistency with previous decisions
2. **Check before/after patterns** in `docs/BEFORE_AND_AFTER.md` to understand what improvements have been vetted
3. **Suggest pattern improvements** that align with the Rust 2021 edition and async/await conventions
4. **Avoid regressions** — if the docs explain why something was changed, don't suggest reverting it
5. **Use proper error types** — lean on `TrackParseError` and Result types rather than panics

## Quick Checks Before Committing

- [ ] No `.unwrap()` calls outside of tests
- [ ] All compile with `cargo build --release` (0 warnings)
- [ ] All tests pass with `cargo test`
- [ ] Uses structured logging (no raw `println!`)
- [ ] Dependencies in Cargo.toml are all used
