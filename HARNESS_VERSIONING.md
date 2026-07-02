# HARNESS_VERSIONING.md

**Spec versioning, artifact compatibility, upgrade paths, and bootstrap.**

The harness has specs (these `.md` files) and artifacts (the data produced by runs). Both need versioning, and both need an upgrade path. Without it, the hash chain breaks on the first edit, old episodes become unreadable, and there is no protocol for the very first agents to prove the harness works before any `CHANGELOG.agent.md` entry exists.

Paired with:
- `HARNESS_PRIMITIVES.md` — defines `schema_version` fields whose values are governed here.
- `HARNESS_SECURITY.md` — defines integrity layers whose continuity is preserved by upgrade rules here.
- `HARNESS_METRICS.md` — sets version-bump gates.

---

## 0. Non-negotiable mandates

1. **Versions are data.** Every spec file and every persisted artifact carries a `spec_version` or `schema_version`. Unversioned data is rejected.
2. **Compatibility is explicit.** A new version declares what it reads from prior versions and what it produces. "Should work" is not compatibility.
3. **No silent upgrades.** Upgrading the harness is an out-of-band human action that produces a version-bump entry in the harness integrity log. A run never upgrades the harness it is running inside.
4. **Old artifacts stay readable.** A reader at spec version N can read artifacts produced at any version M where the compatibility matrix allows; readers must refuse artifacts they cannot validate, not silently misinterpret them.
5. **Bootstrap is a first-class phase.** Before `CHANGELOG.agent.md` Entry 1, the harness must prove its own invariants against itself — that is the bootstrap protocol in §6.

---

## 1. Spec version field

Every harness spec markdown file declares a version inline at the top of its body, in a blockquote or header immediately under the title:

```
> spec_version: 1
> status: stable | draft | deprecated
> supersedes: <prior_spec_version or null>
> effective: <ISO-8601 date>
```

`spec_version` is an integer per file, monotonically increasing. Two-digit versions are for major rewrites; we start at 1. A `draft` file may be merged into `stable` only after §4 release gates pass.

### 1.1 File registry

| File | spec_version | status | supersedes |
|---|---|---|---|
| `HARNESS_ENGINEERING.md` | 1 | stable | null |
| `readharness.md` | 1 | stable | null |
| `HARNESS_PRIMITIVES.md` | 1 | stable | null |
| `HARNESS_SECURITY.md` | 1 | stable | null |
| `HARNESS_METRICS.md` | 1 | stable | null |
| `HARNESS_VERSIONING.md` | 1 | stable | null |

(The registry above is the bootstrap content; future bumps append rows. Never edit a row in place except for `status: stable → deprecated`.)

### 1.2 Cross-file references

Spec files reference each other by `<filename>#<section>`. A spec version bump that moves or renames a section MUST keep a redirect entry in this file's §7. References that point at deprecated sections must be updated in the same version bump that deprecates them; dangling references fail the release gates.

---

## 2. Artifact schema versions

Every persisted artifact carries `schema_version`. The harness reader refuses unknown majors; tolerates unknown minors by ignoring unknown fields; and never rewrites an old artifact in place (forward-write pattern: produce a new artifact file at the new schema, leave the old one in place).

### 2.1 Compatibility matrix — artifact readers

| Artifact | schema_version 1 reader accepts | Producing new major | Notes |
|---|---|---|---|
| `CHANGELOG.agent.md` entry | v1 | v2 only after dual-write period (§3.2) | Hash is computed over canonical text, so any format change requires a new major. |
| `logs/episodes/<run-id>.md` | v1 | v2 with new sections appended after existing ones | Existing section names never change meaning; new sections are additions. |
| `spend/*.jsonl` rows | v1 | v2 must include all v1 fields verbatim | New fields additive only. |
| `tool-calls.jsonl` rows | v1 | v2 must include all v1 fields verbatim | New fields additive only. |
| Snapshots | v1 | v2 requires explicit migration script (`HARNESS_VERSIONING.md` §3.3) | Snapshots cannot be lazily migrated because regression gates depend on them. |
| Approvals, interventions, incidents | v1 | v2 required new-migration script | These are audit-critical. |
| Memory JSONL rows | v1 | v2 forward-write; tombstones stay v1 | Forgetting changes nothing about prior rows. |
| `.harness/config.toml` & sibling TOMLs | 2 (config.toml), 1 (others) | Bump with full migration script | See §3.3. |

### 2.2 Reader-discipline rules

- A reader that encounters `schema_version = N` it does not understand MUST halt with a `schema-rejected` event in `harness.log`. It must NOT silently coerce.
- A reader that encounters a missing `schema_version` in a persisted artifact from after the effective date of this spec (2026-06-29) MUST halt. Artifacts produced before that date are grandfathered as v1. (See §6 for the only legitimate unversioned artifact: the bootstrap-time harness integrity log seed.)
- Unknown fields in a known major are dropped silently. Known fields of unknown value are NOT coerced (e.g., a `verification: "excellent"` value is rejected as out-of-enum, not coerced to `"unknown"`).

### 2.3 Hash stability across versions

The hash chain in `CHANGELOG.agent.md` is computed over the canonical text **of an entry at its own schema_version**. A schema v2 entry has a different canonical form than v1; the hash chain accepts the cross-version jump because:

- The first v2 entry's `prev_hash` equals the last v1 entry's `this_hash` (computed at v1 canonical form).
- The first v2 entry's `this_hash` is computed at v2 canonical form.
- The chain's continuity is preserved by the `prev_hash` reference, not by hash equality of formats.

A reader verifying the chain must therefore know both v1 and v2 canonical forms; refusing v2 means refusing the whole tail of the ledger. This is intentional: downgrading readers is forbidden; upgrade-only.

---

## 3. Upgrade protocol

### 3.1 Spec version bump procedure (out-of-run, by the human)

1. Draft the new spec content in place, bumping the affected file's `spec_version` and adding a row to §1.1.
2. Update cross-reference targets in other spec files (no dangling refs allowed).
3. Update the harness implementation to read the new spec_version. Implementation MUST still read prior versions per §2.2.
4. Run the eval harness (`HARNESS_METRICS.md` §5); all evals MUST pass.
5. Run the changelog chain verifier; chain MUST remain intact.
6. Run the harness integrity log verifier (`HARNESS_SECURITY.md` §8); continuity MUST hold.
7. Append a `spec-bump` entry to `harness.log` with: `<file>`, `from_version`, `to_version`, `effective_date`, `human` (who performed the bump), `commit` (git SHA of the change).
8. Publish the bump in the changelog of the *repository* (human-curated `CHANGELOG.md`, distinct from `CHANGELOG.agent.md`).

### 3.2 Artifact schema bump procedure

A schema bump is a heavier event. It follows a dual-period:

**Period 1 — dual write (default 30 days).**
- Harness writers produce both the old and new canonical form on each write (changelog entries are appended in both formats). Hash chain entries from this period are computed the same way as before (hash of v1 canonical text); a parallel ledger `CHANGELOG.agent.v2.jsonl` records v2-with-hash entries; the new ledger's first entry's `prev_hash` references the v1 ledger's latest `this_hash`.
- Readers may read either ledger; writers must keep both consistent.

**Period 2 — cutover (one day, human-supervised).**
- Harness writers stop producing v1 artifacts. The v1 ledger is frozen and marked `finalized` via a `ledger-finalized` entry in both ledgers (the v1 form is allowed this once because the entry itself marks the closure).
- A `schema-cutover` entry is appended to `harness.log`.
- After cutover, v1 writers are disabled. v1 readers continue per §2.2.

A schema bump may be aborted during Period 1 by removing the v2 writer; aborting during Period 2 is forbidden — it would orphan the v2 ledger's chain.

### 3.3 Migration scripts

For schemas that cannot be forward-written (snapshots, approvals, interventions, incidents, config files), a migration script is required. Migration scripts are:

- Stored under `.harness/migrations/<from>-<to>-<artifact>.ts` (or `.py` per phase language).
- Idempotent: running twice produces the same result.
- Audited: each migration appends a row to `.harness/migrations/applied.jsonl` recording `script_id, artifact_ids_migrated, hash_before, hash_after, at`.
- Reversible where stated (e.g., config.toml `schema_version 1 → 2` is reversible; snapshots are not).
- Signed: scripts run only after signature verification against the publisher key list.

Forbidden: ad-hoc one-offs. A migration that touched the repo without being recorded in `applied.jsonl` is an F2 Audit-Forgery incident.

### 3.4 Deprecation

A spec section can be deprecated (not deleted) by appending to it:

```
> spec_version: 1
> status: deprecated
> deprecated_in: <file>#<section>
> removal_target: <ISO date or null>
```

Deprecated sections MUST still be honored by readers until `removal_target` passes AND a major spec_version bump removes them. Deprecation never breaks compatibility.

Anti-pattern (forbidden): deprecating a section because the implementation lags. Deprecation models intent for future readers, not implementation gaps.

---

## 4. Release gates for the harness itself

A new harness version is releasable only when ALL of the following hold. The checklist is run out-of-run by the human and the eval harness; it is NOT an agent task.

1. Every spec file's header is internally consistent (no file references a section that does not exist).
2. Every artifact schema has its reader and writer implementations passing round-trip tests on representative samples from the past 90 days.
3. The eval suite (`HARNESS_METRICS.md` §5) passes at the rubric's required thresholds.
4. The hash chain (`CHANGELOG.agent.md`) verifies end-to-end.
5. The harness integrity log chain verifies end-to-end.
6. The tool-call ledger cross-checks against the episodes of the trailing 30 days passes (mismatch rate < 0.1%).
7. No open `cadence-miss` older than the cadence it violated.
8. The new version's compatibility matrix in §2.1 is updated and every "migration required" entry has a script in `.harness/migrations/`.
9. A `release-notes.md` entry, signed, is appended under `.harness/releases/<version>.md` summarizing: new features, deprecations, breaking changes, migration requirements, and the human-approved release date.

Skipped gates are recorded as `release-gate-skip` in `harness.log`. Three or more skips in a row block the release regardless of severity.

---

## 5. Compatibility rules for old episodes / runs against new harness

A run begun under spec version N continues under spec version N within its own process. Mid-run harness upgrades are forbidden: the running process holds the read spec at startup and does not pick up edits. An upgrade becomes visible only to runs started after the upgrade.

For reading old episodes by a new harness (for memory inheritance, post-mortems, audits):

- Episodes carry `spec_version` of the harness that produced them. A reader reading v1 episodes MUST interpret them as v1. Superimposing v2 semantics onto a v1 episode is forbidden — that is a silent-action violation.
- Memory entries written under v1 retain v1 provenance. If a v2 memory schema adds a field (e.g., `confidence`), v1 entries are surfaced with `confidence: null` and the retriever downweights them slightly unless the human reviewer overrides. Visual dashboards mark v1-sourced entries with a glyph.
- Re-running an old eval fixture under a new harness MUST produce the same gate outcomes on the golden set; otherwise the harness has regressed and is not releasable.

---

## 6. Bootstrap protocol (pre-Entry-1 validation)

`HARNESS_ENGINEERING.md` §4 says the harness's own creation is not logged in `CHANGELOG.agent.md`. This section defines what happens *instead*: the harness validates itself, in order, before any agent ever runs.

### 6.1 Self-test sequence

Run out-of-run, by the human, immediately after these five spec files are first written to disk:

1. **Spec sanity.** Every spec file parses; every header carries an integer `spec_version`, a `status`, a `supersedes`, an `effective`. Every cross-file reference in any of the five files resolves to an existing section. Result: written to `.harness/artifacts/bootstrap/spec-sanity.json`.

2. **Directory skeleton.** The artifact store layout (`HARNESS_PRIMITIVES.md` §3.1) is created empty, with the lock directory present and writable. Perms: `0700` on `.harness/artifacts/snapshots`, `0600` on others. Result: `bootstrap/dir-layout.json`.

3. **Hash chain genesis.** The hash chain is seeded by appending the genesis marker to `CHANGELOG.agent.md`:
   ```
   ## GENESIS — <ISO-8601>
   - prev_hash: null
   - this_hash: SHA-256 of "GENESIS<timestamp>"
   - bootstrap_human: <name>
   - bootstrap_commit: <sha of the repo at bootstrap>
   - spec_versions: { engineering:1, primitives:1, security:1, metrics:1, versioning:1 }
   ```
   The genesis entry's `this_hash` becomes the `prev_hash` of the first real Entry 1 when the first harness-operated change lands. Genesis is **not** counted as an entry number; Entry 1 is the first agent change.

4. **Harness integrity log seed.** A first line is appended to `.harness/artifacts/harness.log`:
   ```
   {"schema_version":1,"event":"harness-bootstrap","at":"…","human":"…","commit":"…","spec_versions":{...}}
   ```
   This is the only legitimate unversioned-carried artifact at bootstrap, because it is the seed that defines versioning; thereafter every line carries `schema_version`. Each subsequent line in `harness.log` chains via SHA-256 of the prior line's canonical text.

5. **Tool registry primed.** Built-in tool declarations are materialized to `.harness/tools/<id>.toml` with `schema_version = 1`. Their JSON schemas are written under `.harness/tools/schemas/`. The registry is loaded; `tools.list()` is required to return a non-empty set.

6. **Permission defaults loaded.** `.harness/permissions.toml` is seeded with the harness-protective rules:
   ```toml
   schema_version = 1

   [[rules]]
   id = "R-DENY-HARNESS"
   effect = "deny"
   tool = "*"
   paths = [".harness/**"]
   priority = 9999
   reason = "harness config immutability"

   [[rules]]
   id = "R-DENY-SECRETS"
   effect = "deny"
   tool = "*"
   paths = [".env", ".env.*", "**/secrets/**", "**/*.pem", "**/*.key"]
   priority = 9999
   reason = "secret redaction"

   [[rules]]
   id = "R-ASK-DESTRUCTIVE"
   effect = "ask"
   tool = "*"
   # matches any tool with capability.destructive = true (enforced via capability matrix)
   priority = 500
   reason = "destructive ops need approval"
   ```

7. **Models and routing stub.** `.harness/models.toml` and `.harness/routing.toml` seeded with one local model id and one allow-listed cloud model id marked `tier = "local"` and `tier = "flagship"` respectively. Cloud routes default to `mode = "manual"` until the user explicitly enrolls.

8. **Config seed.** `.harness/config.toml` written at `schema_version = 2` with the documented sections and documented defaults.

9. **Bootstrap eval run.** The eval harness is invoked against a trivial fixture (e.g., `evals/bootstrap-readme/`) whose task is "list the files in the bootstrap fixture". The run MUST:
   - produce an episode file with all 10 checklist items answered,
   - produce a changelog Entry 1 (the bootstrap chunk is *inside* the harness's self-test, but the eval run is a real agent run; its first change to the eval fixture's `notes.md` is genuine Entry 1 for that repo),
   - pass all four gates,
   - incur cost within `rubric.max_cpmc = $0.10` (or zero if local model),
   - produce no incidents with impact, and
   - be reverted by the eval runner to leave the eval fixture pristine.

   The bootstrap eval run is written to `bootstrap/eval.json`. If it fails, the harness is NOT considered bootstrapped. Fix the harness, not the eval expectation.

10. **Bootstrap artifact.** `.harness/artifacts/bootstrap/bootstrap.json` is signed and written:
    ```
    {
      "schema_version": 1,
      "bootstrapped_at": "…",
      "bootstrap_human": "…",
      "bootstrap_commit": "…",
      "spec_versions": {...},
      "genesis_hash": "…",
      "eval": { "passed": true, "run_id": "...", "cpmc_usd": 0.0 },
      "integrity_log_seed_hash": "…"
    }
    ```

Until `bootstrap.json` exists with `passed = true`, NO agent run may begin. A run attempted before bootstrap MUST be refused by the runtime with a `not-bootstrapped` error and recorded in `harness.log`.

### 6.2 Re-bootstrap (recovery)

If `bootstrap.json` is missing or its hash chain to `harness.log` is broken:

- The harness is considered unbootstrapped; runs are refused.
- Re-bootstrap is allowed only by an explicit human action recorded as `re-bootstrap` in `harness.log`. Re-bootstrap reuses the *current* `CHANGELOG.agent.md` tail (it does not rewrite history); it appends a new `bootstrap.json` recording the recovery and the prior bootstrap hash (if discoverable).

Re-bootstrap does NOT invalidate prior entries; it re-establishes the integrity chain.

---

## 7. Section redirects

| From | To | Reason |
|---|---|---|
| _(empty on first version)_ | | |

When a section is moved or renamed, add a row here. Empty until needed.

---

## 8. Quick reference

```
SPECS          header blockquote: spec_version, status, supersedes, effective
ARTIFACTS      schema_version per persisted record; readers refuse unknown majors
DUAL PERIOD    30-day dual-write for changelog schema bumps; cutover supervised
MIGRATIONS      idempotent · audited · signed · reversible-or-marked
RELEASE GATES   9 items incl eval pass + chain verify + cross-check + no cadence-miss
COMPAT          new harness reads old episodes as their own spec_version, no coercion
MID-RUN         no harness upgrade; running process holds startup spec
BOOTSTRAP       10-step, ends with signed bootstrap.json with passed=true
GENESIS         marks changelog prev_hash chain start; not an entry number
RE-BOOTSTRAP    human-only; no history rewrite; integrity chain re-established
NO-EXCEPTION    spec_version everywhere, no silent upgrades, no dangling refs
```

---

*End of versioning. With `HARNESS_ENGINEERING.md`, `HARNESS_PRIMITIVES.md`, `HARNESS_SECURITY.md`, `HARNESS_METRICS.md`, and this file, the harness is binding (rules), enforced (primitives), protected (security), measurable (metrics), and upgradeable (versioning).*