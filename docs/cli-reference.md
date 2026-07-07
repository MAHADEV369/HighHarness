# HighHarness CLI Reference

**spec_version:** 0.1.0
**Binary:** `HighHarness`
**Repository:** https://github.com/MAHADEV369/HighHarness

---

## `bootstrap`

**Handler:** `src/cli/bootstrap.rs`

Initialize or verify the harness skeleton and hash chain.

| Subcommand | Description |
|------------|-------------|
| `init` | Initialise a new harness workspace |
| `verify` | Verify an existing bootstrap is intact |

### `bootstrap init`

```
HighHarness bootstrap init --human <NAME>
```

| Flag | Description |
|------|-------------|
| `--human <NAME>` | Name of the human performing the bootstrap |

### `bootstrap verify`

```
HighHarness bootstrap verify
```

---

## `changelog`

**Handler:** `src/cli/changelog.rs`

Append, get, list, or verify the hash-chained changelog.

| Subcommand | Description |
|------------|-------------|
| `append` | Append an entry from a JSON file |
| `latest` | Print the latest entry |
| `verify-chain` | Verify the chain; exit 0 if healthy |
| `get` | Get entry N |

### `changelog append`

```
HighHarness changelog append --entry <PATH>
```

| Flag | Description |
|------|-------------|
| `--entry <PATH>` | Path to a JSON file conforming to `schema::changelog::Entry` |

### `changelog latest`

```
HighHarness changelog latest
```

### `changelog verify-chain`

```
HighHarness changelog verify-chain [--tail <N>]
```

| Flag | Description |
|------|-------------|
| `--tail <N>` | Optional number of trailing entries to check |

### `changelog get`

```
HighHarness changelog get <N>
```

| Argument | Description |
|----------|-------------|
| `<N>` | 1-based entry number to fetch |

---

## `episode`

**Handler:** `src/cli/episode.rs`

Open, append, or close episode traces.

| Subcommand | Description |
|------------|-------------|
| `open` | Open a new episode for a run |
| `append` | Append a section to an open episode |
| `append-tool-call` | Append a tool call record to an episode |
| `close` | Close an episode with verification |
| `hash` | Compute the episode hash |

### `episode open`

```
HighHarness episode open --run-id <ID> --agent-id <ID> --task-spec-file <PATH> --tier <TIER> --phase <PHASE>
```

| Flag | Description |
|------|-------------|
| `--run-id <ID>` | Run identifier |
| `--agent-id <ID>` | Agent identifier |
| `--task-spec-file <PATH>` | Path to the task spec file |
| `--tier <TIER>` | Budget tier for the run |
| `--phase <PHASE>` | Build phase of the run |

### `episode append`

```
HighHarness episode append --run-id <ID> --section <NAME> --body-file <PATH>
```

| Flag | Description |
|------|-------------|
| `--run-id <ID>` | Run identifier |
| `--section <NAME>` | Section name to append |
| `--body-file <PATH>` | Path to the body content file |

### `episode append-tool-call`

```
HighHarness episode append-tool-call --run-id <ID> --tool-call-json <PATH>
```

| Flag | Description |
|------|-------------|
| `--run-id <ID>` | Run identifier |
| `--tool-call-json <PATH>` | Path to the tool call JSON file |

### `episode close`

```
HighHarness episode close --run-id <ID> --verification-json <PATH> --files-touched <PATH>...
```

| Flag | Description |
|------|-------------|
| `--run-id <ID>` | Run identifier |
| `--verification-json <PATH>` | Path to the verification JSON file |
| `--files-touched <PATH>...` | Files touched during the run (repeatable) |

### `episode hash`

```
HighHarness episode hash --run-id <ID>
```

| Flag | Description |
|------|-------------|
| `--run-id <ID>` | Run identifier |

---

## `snapshot`

**Handler:** `src/cli/snapshot.rs`

Take, diff, or revert git snapshots.

| Subcommand | Description |
|------------|-------------|
| `take` | Take a new file snapshot |
| `diff` | Diff two snapshots |
| `revert` | Revert to a previous snapshot |

### `snapshot take`

```
HighHarness snapshot take --run-id <ID> --label <LABEL>
```

| Flag | Description |
|------|-------------|
| `--run-id <ID>` | Run identifier for the snapshot |
| `--label <LABEL>` | Label for the snapshot |

### `snapshot diff`

```
HighHarness snapshot diff --before <SNAPSHOT_ID> --after <SNAPSHOT_ID>
```

| Flag | Description |
|------|-------------|
| `--before <SNAPSHOT_ID>` | Snapshot ID of the before state |
| `--after <SNAPSHOT_ID>` | Snapshot ID of the after state |

### `snapshot revert`

```
HighHarness snapshot revert --snapshot-id <SNAPSHOT_ID>
```

| Flag | Description |
|------|-------------|
| `--snapshot-id <SNAPSHOT_ID>` | Snapshot ID to revert to |

---

## `gates`

**Handler:** `src/cli/gates.rs`

Run verification gates (syntactic/functional/semantic/regression).

| Subcommand | Description |
|------------|-------------|
| `run` | Run a single gate; exit 0 on pass |

### `gates run`

```
HighHarness gates run --phase <PHASE> --gate <GATE> --run-id <ID> --changes <JSON> [--verification <PATH>]
```

| Flag | Description |
|------|-------------|
| `--phase <PHASE>` | Build phase identifier |
| `--gate <GATE>` | Gate name to evaluate (`syntactic`, `functional`, `semantic`, `regression`) |
| `--run-id <ID>` | Run identifier for the gate check |
| `--changes <JSON>` | Inline JSON or path to a JSON file describing changes |
| `--verification <PATH>` | Path to judgment JSON (required for `--gate semantic`) |

---

## `tools`

**Handler:** `src/cli/tools.rs`

Invoke built-in tools or list tool descriptors.

| Subcommand | Description |
|------------|-------------|
| `list` | List all registered tools |
| `invoke` | Invoke a registered tool |

### `tools list`

```
HighHarness tools list
```

### `tools invoke`

```
HighHarness tools invoke --tool <ID> --args <JSON> [--scope-narrow <JSON>] [--run-id <ID>] [--agent-id <ID>]
```

| Flag | Description |
|------|-------------|
| `--tool <ID>` | Tool identifier to invoke |
| `--args <JSON>` | Inline JSON object, or path to a JSON file |
| `--scope-narrow <JSON>` | Inline JSON object, or path to a JSON file (optional) |
| `--run-id <ID>` | Optional run identifier for the tool call |
| `--agent-id <ID>` | Optional agent identifier for the tool call |

---

## `permissions`

**Handler:** `src/cli/permissions.rs`

List or test permission rules.

| Subcommand | Description |
|------------|-------------|
| `list` | List all permission rules |
| `check` | Check a tool invocation against permission rules |

### `permissions list`

```
HighHarness permissions list
```

### `permissions check`

```
HighHarness permissions check --tool <ID> --args <PATH>
```

| Flag | Description |
|------|-------------|
| `--tool <ID>` | Tool identifier to check |
| `--args <PATH>` | Path to a JSON file with the tool arguments |

---

## `spend`

**Handler:** `src/cli/spend.rs`

Track API spend and query cost rollups.

| Subcommand | Description |
|------------|-------------|
| `append` | Append a spend line from a JSON file |
| `summary` | Print a spend summary for a given month |

### `spend append`

```
HighHarness spend append <PATH>
```

| Argument | Description |
|----------|-------------|
| `<PATH>` | Path to the JSON file describing the spend line |

### `spend summary`

```
HighHarness spend summary <YYYY-MM>
```

| Argument | Description |
|----------|-------------|
| `<YYYY-MM>` | Month in `YYYY-MM` format |

---

## `hook`

**Handler:** `src/cli/hooks.rs`

Manage session hooks for pre/post run lifecycle.

| Subcommand | Description |
|------------|-------------|
| `pre-tool` | Run the pre-tool hook (permission check) |
| `post-tool` | Run the post-tool hook (log the tool call) |
| `session-start` | Run the session-start hook (bootstrap and chain verification) |

### `hook pre-tool`

```
HighHarness hook pre-tool [<PATH>]
```

| Argument | Description |
|----------|-------------|
| `[<PATH>]` | Path to the JSON payload (reads stdin if absent) |

### `hook post-tool`

```
HighHarness hook post-tool [<PATH>]
```

| Argument | Description |
|----------|-------------|
| `[<PATH>]` | Path to the JSON payload (reads stdin if absent) |

### `hook session-start`

```
HighHarness hook session-start
```

---

## `integrity`

**Handler:** `src/cli/integrity.rs`

Verify or append integrity log entries.

| Subcommand | Description |
|------------|-------------|
| `verify` | Verify the integrity log chain |
| `append` | Append an event to the integrity log |

### `integrity verify`

```
HighHarness integrity verify
```

### `integrity append`

```
HighHarness integrity append <EVENT>
```

| Argument | Description |
|----------|-------------|
| `<EVENT>` | Event name to record in the integrity log |

---

## `clarification`

**Handler:** `src/cli/clarification.rs`

Request or respond to agent clarification questions.

| Subcommand | Description |
|------------|-------------|
| `list` | List all clarification requests |
| `request` | Request a clarification |
| `resolve` | Resolve a clarification request with an answer |

### `clarification list`

```
HighHarness clarification list
```

### `clarification request`

```
HighHarness clarification request --question <TEXT>
```

| Flag | Description |
|------|-------------|
| `--question <TEXT>` | The question to ask |

### `clarification resolve`

```
HighHarness clarification resolve --id <ID> --answer <TEXT>
```

| Flag | Description |
|------|-------------|
| `--id <ID>` | ID of the clarification request |
| `--answer <TEXT>` | The answer to the clarification |

---

## `eval`

**Handler:** `src/cli/eval.rs`

Run evaluations against episode data.

| Subcommand | Description |
|------------|-------------|
| `list` | List all available evals |
| `run` | Run a single eval by id |

### `eval list`

```
HighHarness eval list
```

### `eval run`

```
HighHarness eval run <ID> [--run-id <ID>]
```

| Argument | Description |
|----------|-------------|
| `<ID>` | Eval id (directory name under `.harness/evals/`) |

| Flag | Description |
|------|-------------|
| `--run-id <ID>` | Optional run id (auto-generated if omitted) |

---

## `id-run`

**Handler:** `src/cli/id_cmd.rs`

Generate or pin run identifiers.

```
HighHarness id-run [--slug <SLUG>] [--agent <AGENT>] [--pin]
```

| Flag | Description |
|------|-------------|
| `--slug <SLUG>` | Short slug for the run (default: `run`) |
| `--agent <AGENT>` | Short identifier of the calling agent (default: `agent`) |
| `--pin` | Pin to GENESIS bootstrap timestamp for reproducible demos (requires `deterministic` feature) |

---

## `id-agent`

**Handler:** `src/cli/id_cmd.rs`

Generate or pin agent identifiers.

```
HighHarness id-agent [--state-dir <PATH>] [--pin]
```

| Flag | Description |
|------|-------------|
| `--state-dir <PATH>` | Path to state dir for sticky agent ids (default: `.`) |
| `--pin` | Pin to GENESIS bootstrap timestamp for reproducible demos (requires `deterministic` feature) |

---

## `metrics`

**Handler:** `src/cli/metrics.rs`

Compute KPI rollups and alerts.

| Subcommand | Description |
|------------|-------------|
| `rollup` | Compute a metrics rollup for the given window |
| `alert` | Evaluate alerts over the given window |
| `health` | Print a simple health summary |

### `metrics rollup`

```
HighHarness metrics rollup --window <WINDOW>
```

| Flag | Description |
|------|-------------|
| `--window <WINDOW>` | Window size: `7d`, `30d`, or `90d` |

### `metrics alert`

```
HighHarness metrics alert --window <WINDOW>
```

| Flag | Description |
|------|-------------|
| `--window <WINDOW>` | Window size: `7d`, `30d`, or `90d` |

### `metrics health`

```
HighHarness metrics health
```

---

## `cadence`

**Handler:** `src/cli/cadence.rs`

Schedule periodic metrics rollups.

| Subcommand | Description |
|------------|-------------|
| `run` | Run a cadence rollup for the specified window |

### `cadence run`

```
HighHarness cadence run [--daily] [--weekly] [--monthly]
```

| Flag | Description |
|------|-------------|
| `--daily` | Run daily cadence |
| `--weekly` | Run weekly cadence |
| `--monthly` | Run monthly cadence |

---

## `redaction`

**Handler:** `src/cli/redaction.rs`

Manage secret-redaction patterns.

| Subcommand | Description |
|------------|-------------|
| `scan` | Scan content for redactable secrets |
| `list` | List registered redaction patterns |
| `add` | Add a new pattern |

### `redaction scan`

```
HighHarness redaction scan [--file <PATH>]
```

| Flag | Description |
|------|-------------|
| `--file <PATH>` | Path to file (reads stdin if omitted) |

### `redaction list`

```
HighHarness redaction list
```

### `redaction add`

```
HighHarness redaction add --id <ID> --regex <REGEX> --severity <LEVEL>
```

| Flag | Description |
|------|-------------|
| `--id <ID>` | Pattern id |
| `--regex <REGEX>` | Regex pattern |
| `--severity <LEVEL>` | Severity level |

---

## `incident`

**Handler:** `src/cli/incident.rs`

Declare and manage security incidents.

| Subcommand | Description |
|------------|-------------|
| `declare` | Declare a new incident |
| `list` | List incidents |
| `ack` | Acknowledge an incident |
| `close` | Close an incident |

### `incident declare`

```
HighHarness incident declare --run-id <ID> --detection-rule <RULE> --vector <VECTOR> --severity <LEVEL> --had-impact <BOOL> [--evidence <PATH>...]
```

| Flag | Description |
|------|-------------|
| `--run-id <ID>` | Run identifier associated with the incident |
| `--detection-rule <RULE>` | Detection rule that triggered the incident |
| `--vector <VECTOR>` | Attack or failure vector |
| `--severity <LEVEL>` | Severity level |
| `--had-impact <BOOL>` | Whether the incident had impact |
| `--evidence <PATH>...` | Evidence paths or URLs (repeatable) |

### `incident list`

```
HighHarness incident list [--open-only]
```

| Flag | Description |
|------|-------------|
| `--open-only` | Only list incidents that are not yet closed |

### `incident ack`

```
HighHarness incident ack <ID> --by <NAME>
```

| Argument | Description |
|----------|-------------|
| `<ID>` | Incident identifier |

| Flag | Description |
|------|-------------|
| `--by <NAME>` | Person or agent acknowledging the incident |

### `incident close`

```
HighHarness incident close <ID> [--postmortem <PATH>]
```

| Argument | Description |
|----------|-------------|
| `<ID>` | Incident identifier |

| Flag | Description |
|------|-------------|
| `--postmortem <PATH>` | Optional postmortem document path |

---

## `models`

**Handler:** `src/cli/models.rs`

Configure model routing and inference.

| Subcommand | Description |
|------------|-------------|
| `list` | List configured models from `.harness/models.toml` |
| `complete` | Complete a model request |

### `models list`

```
HighHarness models list
```

### `models complete`

```
HighHarness models complete --model <ID> [--messages-file <PATH>] [--messages <JSON>]
```

| Flag | Description |
|------|-------------|
| `--model <ID>` | Model id |
| `--messages-file <PATH>` | File with messages JSON |
| `--messages <JSON>` | Inline messages JSON |

---

## `mcp`

**Handler:** `src/cli/mcp.rs`

Start MCP server (stdio or HTTP transport) and manage MCP server processes.

| Subcommand | Description |
|------------|-------------|
| `register` | Register an MCP server config |
| `start` | Start a registered MCP server |
| `stop` | Stop a running MCP server |
| `list` | List registered servers |
| `serve` | Start the harness as an MCP server over stdio |
| `serve-http` | Start the harness as an MCP server over HTTP |

### `mcp register`

```
HighHarness mcp register <ID> --command <CMD>... [--paths-allowed <PATH>...] [--network-allowed <ADDR>...] [--env-allowed <VAR>...] [--cpu-seconds <N>] [--memory-mb <N>] [--timeout-seconds <N>]
```

| Argument | Description |
|----------|-------------|
| `<ID>` | Server id |

| Flag | Description |
|------|-------------|
| `--command <CMD>...` | Command and arguments for the server binary |
| `--paths-allowed <PATH>...` | Filesystem paths the server may access |
| `--network-allowed <ADDR>...` | Network addresses the server may connect to |
| `--env-allowed <VAR>...` | Environment variables to forward |
| `--cpu-seconds <N>` | CPU time limit in seconds |
| `--memory-mb <N>` | Memory limit in megabytes |
| `--timeout-seconds <N>` | Wall-clock timeout in seconds |

### `mcp start`

```
HighHarness mcp start <ID>
```

| Argument | Description |
|----------|-------------|
| `<ID>` | Server id |

### `mcp stop`

```
HighHarness mcp stop <ID>
```

| Argument | Description |
|----------|-------------|
| `<ID>` | Server id |

### `mcp list`

```
HighHarness mcp list
```

### `mcp serve`

```
HighHarness mcp serve
```

Reads JSON-RPC 2.0 from stdin, writes responses to stdout.

### `mcp serve-http`

```
HighHarness mcp serve-http [--port <PORT>] [--host <HOST>]
```

| Flag | Description |
|------|-------------|
| `--port <PORT>` | Port to listen on (default: `8931`) |
| `--host <HOST>` | Host to bind to (default: `127.0.0.1`) |

---

## Global flags

These flags are available on every command.

| Flag | Description |
|------|-------------|
| `--root <PATH>` | Working directory (defaults to current directory) |
| `--help` | Print help information |
| `--version` | Print version information |

### Usage

```
HighHarness [--root <PATH>] <COMMAND> [<ARGS>...]
HighHarness --help
HighHarness --version
```

### Examples

```
HighHarness bootstrap init --human "Your Name"
HighHarness changelog get 7
HighHarness changelog verify-chain
HighHarness episode open --run-id my-run --agent-id my-agent --task-spec-file task.md --tier trivial --phase highharness
HighHarness snapshot take --run-id my-run --label baseline
HighHarness gates run --phase highharness --gate syntactic --run-id my-run --changes changes.json
HighHarness tools invoke --tool fs.read --args '{"path":"Cargo.toml"}' --run-id my-run --agent-id my-agent
HighHarness mcp serve-http --port 8931
HighHarness id-run --slug add-version-flag --pin
HighHarness cadence run --daily
```
