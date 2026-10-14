# Explicit retained-checkpoint branching

Sean requested a fixed warmed parent with independent sibling jobs, rather than
repeated parent disk pivots and periodic reconstruction. Root chose a minimal
explicit CLI option on 2026-09-21; Kepler implements it against pinned runtime
9504e94e3581a1f52c414247edcbcd6d6b49a71a. James owns native compilation and Root
owns independent review and live verification.

`machine branch --reuse-checkpoint` (including the existing `fork` alias) uses
the existing retained-generation preparation/boot path. Plain, indexed batch
and held branches preserve their names, environments, secrets, identity reset,
forkpoint handling and existing release/gate semantics. Omission preserves
the current direct-fork behavior; pool/API optional reuse is unchanged.

The explicit option requires a persisted retained checkpoint for the same live
source PID and kernel birth time, with the existing real owned snapshot path
and running-source marker validation. Missing, stale, dead, changed-path or
incompatible RAM-policy checkpoints return errors before new capture, disk
pivot or recovery of uncommitted generations. There is no fresh-capture
fallback and no new API server, pool controller, lease owner or worker naming.

The existing source lock remains held over preparation/boot. The flag reuses
the already captured state, not changes the source made afterward. It does not
flatten existing chains, raise the depth limit, delete ancestors or qualify
the new CLI binary for installed fleets automatically.

Verification: the original pinned CLI causally rejects --reuse-checkpoint
with exit2 before VM lookup. New Rust parser tests cover explicit single,
batch, alias and held forms plus unchanged default. Selection tests cover
missing checkpoint, changed PID/birth, missing running-source marker,
foreign path, valid paused/running reuse, and unchanged direct/pool defaults.
Native compilation and live repeated-fork depth proof remain pending.

## Provenance: the deployed fork CLI is reproducible (verified 2026-10-14)

The fleet's retained fork CLI `5bcd38d1ed7e7e99a7a91cecdf199e4753b97ee027a2d50741db6d7dae3c4082`
is rebuilt byte-for-byte from 9504e94 plus this patch (commit "machine branch
--reuse-checkpoint ... (recovered 2026-09-21 fleet patch)") with:

- toolchain: rustc/cargo 1.93.1 (01f6ddf75), ld-1230.1, macOS SDK 26.2;
- exact paths (absolute source paths are embedded): source at
  `/Volumes/extssd/shd18/reuse-checkpoint-20260921/smolvm-9504e94e3581a1f52c414247edcbcd6d6b49a71a`
  (a physical directory: cargo canonicalizes the workspace root),
  `CARGO_HOME=/Volumes/extssd/shd18/cargo`,
  `CARGO_TARGET_DIR=/Volumes/extssd/shd18/reuse-checkpoint-20260921/native-target`;
- `LIBKRUN_DIR` holding the runtime's libkrun `404127e4...` (install ID
  `libkrun.2.dylib`), `RUSTFLAGS='-C link-arg=-Wl,-rpath,@executable_path/lib
  -C link-arg=-Wl,-rpath,@executable_path/../lib'`, no `LIBKRUN_BUNDLE`/`_STATIC`/`_BUILD`;
- `cargo build --release --locked --offline -j 2 -p smolvm --bin smolvm`;
- utoipa-swagger-ui's rust-embed records each swagger-ui file's mtime and
  birthtime: the original extraction is 1789997639 for the first twelve files
  in sorted order (swagger-initializer.js modified 1789997640) and 1789997640
  for the last six; set them, then recompile that crate;
- `codesign --force --sign - --entitlements smolvm.entitlements` (gives the
  intermediate `ed7ac009...`), copy to `tiny-cli-v2/smolvm-bin`,
  `install_name_tool -change libkrun.2.dylib @rpath/libkrun.dylib`, and the
  same `codesign` again: `5bcd38d1...`.
