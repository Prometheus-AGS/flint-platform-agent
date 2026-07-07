# Flint Platform Agent — convenience targets (thin wrappers over smoke/ scripts).
#
# These add no new behavior — they make the smoke surface discoverable + one-command.
# See smoke/README.md for the full story (stub vs real, --no-build, forge-full profile).

.PHONY: help smoke smoke-real smoke-real-nobuild smoke-real-forge

help: ## List the available targets
	@echo "Flint Platform Agent — make targets:"
	@echo "  make smoke              Stub smoke (self-contained: agent + postgres + wiremock)"
	@echo "  make smoke-real         Real-sibling smoke (agent + real gate + real fabric)"
	@echo "  make smoke-real-nobuild Real smoke on PRE-BUILT images (the reliable fast path)"
	@echo "  make smoke-real-forge   Real smoke incl. the forge gateway (needs flint-forge#7)"

smoke: ## Self-contained stub smoke (no siblings, no secrets)
	./smoke/run.sh

smoke-real: ## Real-sibling smoke — builds images (heavy; see --no-build)
	./smoke/run-real.sh

smoke-real-nobuild: ## Real smoke on pre-built images — the reliable path
	# The 12 GiB VM runs the 8-service stack fine (~60s); only concurrent BUILDS OOM it.
	# Build images once per-service first, then boot: `docker compose -f smoke/compose.real.yml build <svc>`.
	./smoke/run-real.sh --no-build

smoke-real-forge: ## Real smoke incl. forge gateway — BLOCKED on flint-forge#7
	./smoke/run-real.sh --forge-full
