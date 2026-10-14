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
