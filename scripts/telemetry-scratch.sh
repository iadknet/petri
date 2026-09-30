# Sourced (POSIX sh) by scripts/telemetry-verify, telemetry-overhead and
# telemetry-dashboards-check. The production volume petri-telemetry is
# external (telemetry/compose.yaml), so no `docker compose down -v` removes
# it; each scratch stack creates and removes its own volume here instead.
# The caller sets compose_file and defines compose() on the selected project.

# scratch_stack NAME: selects the scratch Compose project and volume NAME
# (petri-telemetry-<suffix>, never petri-telemetry) for every compose call and
# for scripts/telemetry-cleanup; refuses while petri-telemetry is running,
# since it holds the same host ports.
scratch_stack() {
  case "$1" in
    petri-telemetry-?*) ;;
    *) printf '%s: refusing scratch stack name %s\n' "${0##*/}" "$1" >&2; exit 2 ;;
  esac
  export PETRI_TELEMETRY_PROJECT="$1" PETRI_TELEMETRY_VOLUME="$1"
  # shellcheck disable=SC2154 # compose_file is the caller's.
  if [ -n "$(docker compose -p petri-telemetry -f "$compose_file" ps -q --status running lgtm 2>/dev/null)" ]; then
    printf '%s: the petri-telemetry stack is up and holds the same ports; stop it with make telemetry-down\n' "${0##*/}" >&2
    exit 1
  fi
}

# scratch_volume_create: removes any stale scratch stack, then creates an
# empty scratch volume.
scratch_volume_create() {
  scratch_stack_remove
  docker volume create "$PETRI_TELEMETRY_VOLUME" >/dev/null
}

# scratch_stack_ready: waits up to 240 s for Prometheus, Loki and Grafana to
# be ready and the collector to accept OTLP/HTTP on 127.0.0.1:4318.
scratch_stack_ready() {
  waited=0
  until compose exec -T lgtm curl -sf http://127.0.0.1:9090/-/ready >/dev/null 2>&1 &&
    compose exec -T lgtm curl -sf http://127.0.0.1:3100/ready >/dev/null 2>&1 &&
    compose exec -T lgtm curl -sf http://127.0.0.1:3000/api/health >/dev/null 2>&1 &&
    curl -sf -o /dev/null -X POST -H 'Content-Type: application/json' -d '{}' \
      http://127.0.0.1:4318/v1/metrics 2>/dev/null; do
    waited=$((waited + 2))
    [ "$waited" -lt 240 ] || { printf '%s: stack not ready after 240 s\n' "${0##*/}" >&2; exit 1; }
    sleep 2
  done
}

# scratch_stack_remove: the scratch stack's `down -v`, then its volume.
scratch_stack_remove() {
  compose down -v --remove-orphans >/dev/null 2>&1 || true
  docker volume rm -f "$PETRI_TELEMETRY_VOLUME" >/dev/null 2>&1 || true
}
