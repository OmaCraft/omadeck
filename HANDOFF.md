# OmaDeck engineering handoff

## Development lifecycle

OmaCraft coordinates exact revisions, independent reviews and cross-platform evidence at
https://github.com/ustaaa/omacraft. This repository keeps its own upstream history.
Use origin for the Oma fork and upstream for the original project. Work on a topic branch,
review the diff, run relevant gates, and open a draft pull request. Do not automatically
publish releases or overwrite another device's dirty checkout.

## Current pilot (2026-10-30)

Linux ARM64 and x86_64 pilot binaries have been built. Native Wayland launch and sample
rendering were checked at display scales 1.6 and 1.5 respectively. This is not full
keyboard, IME, clipboard, accessibility or document-compatibility certification.
The ribbon and command layout are retained. Appearance must not modify document state.
macOS build workflows are experimental and manual; no Oma macOS distribution is certified.

Full Linux quality gates passed during pilot preparation. Fresh synthetic corpus comparisons
passed for Grid (3 cases), Word (2) and Deck (2); Notes CLI smoke passed 20 checks.
A previous x86 Notes UI test process terminated with SIGSEGV at four test threads.
All 121 UI tests then passed serially and under a debugger; full serial CI passed.
The cause remains unresolved. Do not describe reruns as a fix or hide this risk.

## Next review

Review downstream identity, theme isolation, font reload, package/updater separation,
and the declared support matrix. Revalidate at the exact reviewed commit and record
compiler, target, commands, result, skipped checks and logs. Keep private machine paths,
notebooks, credentials and infrastructure inventory out of tracked reports.

### OmaCraft brand polish (2026-10-30)
Suite links and documentation now use ustaaa/omacraft. Original application icon
regenerated in the shared dark-tile/paper style, with a distinct application accent.
PNG/SVG/ICO structure (where applicable), deterministic regeneration and whitespace
checks passed. No engine, document, command, dependency or version changes.
Installed-app validation of these new assets is still pending; previous runtime
evidence does not cover this polish. Existing CI limitations remain open.

### Suite review follow-up (2026-10-31)
Linux pilot policy documents native Wayland and `OMADECK_X11=1`; existing
`DECKCRAFT_WAYLAND` remains accepted. Recovery now uses the OmaDeck XDG data path;
legacy config recovery entries are copied without overwriting, with uid collisions
assigned a fresh recoverable id while the old files remain. Added a focused migration
test. Formatting and diff checks pass; Rust test execution could not access the Cargo
git checkout outside the writable workspace.

### Candidate 2 review fixes (2026-10-31)
Theme root priority/recovery and exact `OMADECK_X11=1` handling now have focused tests.
Legacy Deck recovery migration skips entries with a held owner lock in the old folder;
engine and app tests cover real lock files and migration. Cargo could not start tests
because the shared Cargo git DB is unwritable from this workspace; formatting passed.

### Upstream variable isolation by Claude (2026-10-31; branch claude/identity-env)
Preferences, recovery and the control port now read only `OMADECK_NO_PREFS`,
`OMADECK_NO_RECOVERY` and `OMADECK_CONTROL_PORT`. The `DECKCRAFT_` fallbacks are gone,
so values exported for an upstream DeckCraft install cannot switch off OmaDeck
preferences or recovery or bind the same control port. `DECKCRAFT_WAYLAND` stays as the
documented backend alias. A unit test proves upstream names are ignored.

### Alpha version reporting (2026-11-01)
Downstream label `0.12.0-oma.1.alpha.1` is shown by GUI About and
`omadeck --version` / `omadeck-cli --version`; output names upstream DeckCraft 0.12.0
separately. Linux packages use the downstream label and include `VERSION.txt` with both
versions. Cargo package versions and lockfile are unchanged. Rust builds/runtime and
package artifact verification are delegated to Halo; local formatting, shell syntax and
diff whitespace checks passed.

### DeckCraft v0.13.1 integration candidate (2026-11-01)

Prepared in isolated checkout `/Users/irtek/Development/omacraft-integration-20261101/omadeck` on `codex/upstream-deck-0.13.1`; canonical checkout was not edited. Base is OmaDeck alpha 1 `cd1aacb136baec379e05a95262dcfbc1aaf6a802`; exact upstream tag `v0.13.1` resolves to commit `4ce2944630b04188dee8504c14151994d06c416a` (common ancestor `29f0f7772c47aa2bfb74fcbb0c4d3b1503300849`).

Resolved nine textual conflicts: `AGENTS.md`, `Cargo.lock`, CLI help/main, desktop main, UI updater/lib, and both Oma Flatpak manifests. Kept the Oma CLI/app names, startup/version reporting, Omarchy appearance hooks, installer-managed update notice, and offline/no-network Flatpak policy. Integrated upstream CLI command-file and closed-stdout behavior plus multi-root automation reporting. Upstream self-updater startup/UI and its `craft-update` application dependency were excluded to keep update channels isolated. Retained upstream release content elsewhere, including ribbon and command changes.

Cargo workspace package version is `0.13.1`; downstream `OMA_VERSION` is `0.13.1-oma.1.alpha.1`. No new direct app dependencies were added beyond the release; `craft-update` was removed from the workspace dependency table because Oma updates remain installation-managed. The lock keeps the release's dependency versions, removes updater-only dependency edges and retains Oma's pinned `craft-platform` appearance dependency. Rust minimum remains 1.95. Upstream changed font database code/tests only; no font assets, font manifest, or `CRAFT_FONTS_DIR` input changed. Actual font input revision was not checked in this checkout.

`git diff --check` passed. `cargo metadata --offline --locked` resolved the graph but exited when it tried to download uncached `accesskit 0.25.1`; no compile or tests were run here. `cargo fmt --all -- --check` and merge-diff review remain pending before commit. Halo owns build/test validation. This branch is not pushed, merged to pilot/main, tagged, or deployed.

## 2026-11-01 Claude upstream review corrections
Restored upstream SlideDrag, presenter metadata/keepCursor command and presenter/reflection test modules lost in merge conflict resolution. Adapted new CLI binary/error-prefix tests and three presenter preview harnesses to Oma identity/font initialization. Production font paths unchanged. Halo four-thread full cargo xtask ci passes all8 steps on reviewed source patch; exact clean-head rerun pending. Original failures retained privately. Archived unadapted FreeBSD packager behind exit2 stub. Claude Opus5.5 independently confirmed fixes and production font bootstrap. No deployment.

## Alpha 3 upstream integration in progress (2026-11-05)

On `codex/alpha-3`, merged upstream DeckCraft v0.15.1, exact commit
`9bac60ff53029039c82083ee574c22d74c1541f1`, over alpha2 base
`d54dae3824f0526ab06cfba51fd987086befa1d7` with `--no-commit`. Downstream version is
`0.15.1-oma.1.alpha.1`. The merge is resolved and staged for review; it is not committed or pushed.

Conflict decisions: retained the OmaDeck README and disabled installation-managed updater;
removed `craft-update` from manifest/lock. Kept the private-name bundled PDF font test fixture
and attribution/licence behavior, preserved the real recovery lock regression test alongside the
upstream instance-ID test, and kept OmaDeck About copy and app identity. Retained the release's
SlideDrag, presenter/keepCursor behavior and presenter/reflection tests. Release-facing
`craft-media`, `craft-pdf`, `craft-ui`, and `krilla` remain on the release-compatible `c5a1cbf`
revision; the separately pinned `craft-platform` appearance integration stays at `92d392c`.
OmaDeck font bootstrap paths remain unchanged.

The added OCI packaging helper now uses `omadeck-$OMA_VERSION`, defaults to
`ghcr.io/ustaaa/omadeck`, and publishes only with explicit `--push`; no push was run. Active
upstream updater references are absent. The upstream release workflow remains `.upstream` and is
not enabled.

Checks on this macOS host: `cargo fmt --all -- --check` passed;
`cargo metadata --offline --locked --no-deps --format-version 1` passed;
`bash -n packaging/linux/flatpak-oci.sh packaging/linux/flatpak-bundle.sh` passed;
`git diff --check` passed. Full offline Cargo metadata could not check the pinned `krilla` git
source because it is not cached; no dependency fetch/install, compile, or tests were run.
Full `cargo +1.98.0 xtask ci` and `cargo +1.98.0 build --locked --release -p deckcraft -p deckcraft-cli` remain pending on the designated Linux x86_64 build host.
No NUC contact or runtime validation was performed.
