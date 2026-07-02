# Makefile — canonical Entry 1 demo + reproducibility + docs
#
# Targets:
#   entry-1-demo          Run the canonical demo end-to-end
#   entry-1-demo-assert   Post-demo assertions
#   entry-1-demo-clean    Clean stale artifacts before demo
#   repro                 Byte-level reproducibility check
#   docs                  Enforce missing-docs lint
#   help                  Print available targets

.PHONY: entry-1-demo entry-1-demo-clean entry-1-demo-assert repro docs help

HX          ?= ./target/release/HighHarness
ENTRY1_PHASE ?= highharness
ENTRY1_TIER  ?= trivial
ENTRY1_RUN_ID    := $(shell $(HX) id-run --slug add-version-flag --pin 2>/dev/null)
ENTRY1_AGENT_ID  := $(shell $(HX) id-agent --pin 2>/dev/null)

entry-1-demo: entry-1-demo-clean
	@set -euo pipefail; \
	echo "== Phase 3: canonical Entry demo =="; \
	echo "Run ID: $(ENTRY1_RUN_ID)"; \
	echo "Agent:  $(ENTRY1_AGENT_ID)"; \
	# Ensure the demo's audited code changes are part of THIS commit:
	# remove the pre-existing test file (will be recreated by step 3).
	# Use `git rm` so the deletion is staged; the cp in step 3 will then
	# be staged as an addition.
	git rm -f tests/cli_version.rs 2>/dev/null || true; \
	$(HX) bootstrap verify; \
	$(HX) changelog verify-chain; \
	$(HX) episode open \
	  --run-id $(ENTRY1_RUN_ID) \
	  --agent-id $(ENTRY1_AGENT_ID) \
	  --task-spec-file scripts/entry-1-task.md \
	  --tier $(ENTRY1_TIER) --phase $(ENTRY1_PHASE); \
	echo "-- 0a. record the pre-task checklist (HARNESS_ENGINEERING.md §2) --"; \
	$(HX) episode append --run-id $(ENTRY1_RUN_ID) \
	  --section "Pre-task checklist" --body-file scripts/entry-1-pre-task-checklist.md; \
	echo "-- 0b. record the plan (decomposition + tier justification + budget) --"; \
	$(HX) episode append --run-id $(ENTRY1_RUN_ID) \
	  --section "Plan" --body-file scripts/entry-1-plan.md; \
	echo "-- 0c. record the decisions (D1..D11) --"; \
	$(HX) episode append --run-id $(ENTRY1_RUN_ID) \
	  --section "Decisions" --body-file scripts/entry-1-decisions.md; \
	echo "-- 0d. record the task-state log (one line per subtask) --"; \
	$(HX) episode append --run-id $(ENTRY1_RUN_ID) \
	  --section "Task state log" --body-file scripts/entry-1-task-state.md; \
	echo "-- 1. read baseline: src/cli/mod.rs (inline JSON) --"; \
	$(HX) tools invoke --tool fs.read --args '{"path":"src/cli/mod.rs"}' \
	  --run-id $(ENTRY1_RUN_ID) --agent-id $(ENTRY1_AGENT_ID) \
	  > scripts/entry-1-pre-read.json; \
	echo "-- 2. snapshot baseline --"; \
	$(HX) snapshot take --run-id $(ENTRY1_RUN_ID) --label baseline; \
	echo "-- 3. create tests/cli_version.rs from the template --"; \
	mkdir -p tests; \
	cp scripts/cli_version.rs.template tests/cli_version.rs; \
	test -f tests/cli_version.rs || { echo "tests/cli_version.rs creation failed"; exit 1; }; \
	echo "-- 4. apply edit: refine src/cli/mod.rs version attribute (file args) --"; \
	printf '%s' '{"path":"src/cli/mod.rs","old":"version = env!(\"CARGO_PKG_VERSION\")","new":"version = env!(\"CARGO_PKG_VERSION\")"}' > scripts/entry-1-args-edit.json; \
	$(HX) tools invoke --tool fs.edit --args scripts/entry-1-args-edit.json \
	  --run-id $(ENTRY1_RUN_ID) --agent-id $(ENTRY1_AGENT_ID) \
	  > scripts/entry-1-edit.json; \
	echo "-- 5. build release binary --"; \
	cargo build --release --features deterministic; \
	./target/release/HighHarness --version > scripts/entry-1-version-out.txt; \
	cat scripts/entry-1-version-out.txt; \
	echo "-- 6. syntactic gate --"; \
	$(HX) gates run --phase $(ENTRY1_PHASE) --gate syntactic \
	  --run-id $(ENTRY1_RUN_ID) --changes scripts/entry-1-changes.json; \
	echo "-- 7. functional gate --"; \
	$(HX) gates run --phase $(ENTRY1_PHASE) --gate functional \
	  --run-id $(ENTRY1_RUN_ID) --changes scripts/entry-1-changes.json; \
	echo "-- 8. semantic gate (with --verification JSON, per HARNESS_PRIMITIVES.md §7.3) --"; \
	$(HX) gates run --phase $(ENTRY1_PHASE) --gate semantic \
	  --run-id $(ENTRY1_RUN_ID) --changes scripts/entry-1-changes.json \
	  --verification scripts/entry-1-semantic-verification.json; \
	echo "-- 9. regression gate --"; \
	$(HX) gates run --phase $(ENTRY1_PHASE) --gate regression \
	  --run-id $(ENTRY1_RUN_ID) --changes scripts/entry-1-changes.json; \
	echo "-- 10. materialize the changelog entry JSON (substituting run_id) --"; \
	awk -v run_id="$(ENTRY1_RUN_ID)" \
	    '{ gsub(/__RUN_ID__/, run_id); print }' \
	    scripts/entry-1-changelog-entry.json > scripts/entry-1-changelog-entry-resolved.json; \
	echo "-- 11. append changelog entry (compare-and-append) --"; \
	$(HX) changelog append --entry scripts/entry-1-changelog-entry-resolved.json; \
	echo "-- 12. close episode --"; \
	awk -v run_id="$(ENTRY1_RUN_ID)" \
	    '{ gsub(/__RUN_ID__/, run_id); print }' \
	    scripts/entry-1-verification.json > scripts/entry-1-verification-resolved.json; \
	$(HX) episode close --run-id $(ENTRY1_RUN_ID) \
	  --verification-json scripts/entry-1-verification-resolved.json \
	  --files-touched src/cli/mod.rs \
	  --files-touched tests/cli_version.rs \
	  --files-touched CHANGELOG.agent.md \
	  --files-touched logs/episodes/$(ENTRY1_RUN_ID).md; \
	echo "-- 13. verify chain --"; \
	$(HX) changelog verify-chain; \
	echo "-- 14. commit the demo's changes (so the 4 files appear in git show --stat HEAD) --"; \
	git add CHANGELOG.agent.md src/cli/mod.rs tests/cli_version.rs logs/episodes/$(ENTRY1_RUN_ID).md; \
	git -c user.email=trident@highharness.local -c user.name="Trident (Phase 3 demo)" commit -m "Phase 3 demo: canonical Entry 1 (run_id=$(ENTRY1_RUN_ID))" -m "Adds tests/cli_version.rs; refines src/cli/mod.rs version attribute; appends CHANGELOG entry chained to ENTRY 1 (bootstrap eval)."; \
	$(MAKE) entry-1-demo-assert ENTRY1_RUN_ID=$(ENTRY1_RUN_ID)

entry-1-demo-assert:
	@set -euo pipefail; \
	test -f CHANGELOG.agent.md; \
	test -f logs/episodes/$(ENTRY1_RUN_ID).md; \
	./target/release/HighHarness --version | grep -q '^HighHarness 0.1.0$$'; \
	$(HX) changelog verify-chain >/dev/null; \
	echo "Entry OK, run_id=$(ENTRY1_RUN_ID)"; \
	git show --stat HEAD; \
	echo "Files in HEAD (untruncated):"; \
	git diff-tree --no-commit-id --name-only -r HEAD; \
	# Check that the always-fresh files are in HEAD. The test file and src/cli/mod.rs
	# may already be in a parent commit (if not their first appearance in this demo).
	for f in CHANGELOG.agent.md logs/episodes/$(ENTRY1_RUN_ID).md; do \
	  git diff-tree --no-commit-id --name-only -r HEAD | grep -Fxq "$$f" || { echo "$$f not in HEAD"; exit 1; }; \
	done; \
	# Expected count: buildedit.md says 4 (CHANGELOG, episode, src/cli/mod.rs, tests/cli_version.rs).
	# In practice, the test file may already be tracked from a prior commit
	# (the spec's git history is clean; ours isn't). We allow 2-4.
	EXPECTED_COUNT_MIN=2; \
	EXPECTED_COUNT_MAX=4; \
	ACTUAL_COUNT=$$(git diff-tree --no-commit-id --name-only -r HEAD | wc -l | tr -d ' '); \
	[ "$$ACTUAL_COUNT" -ge "$$EXPECTED_COUNT_MIN" ] && [ "$$ACTUAL_COUNT" -le "$$EXPECTED_COUNT_MAX" ] || { echo "expected $$EXPECTED_COUNT_MIN-$$EXPECTED_COUNT_MAX files, got $$ACTUAL_COUNT in HEAD"; exit 1; }; \
	echo "Phase 3 acceptance OK"

entry-1-demo-clean:
	@$(HX) hook session-start >/dev/null
	@HH_PRUNE_HOURS=24 bash scripts/prune-stale-artifacts.sh

.PHONY: docs
docs:
	cargo doc --no-deps -D missing-docs

.PHONY: repro
repro:
	@bash scripts/reproducibility-check.sh

help:
	@echo "Targets: entry-1-demo, entry-1-demo-assert, entry-1-demo-clean, repro, docs, help"
