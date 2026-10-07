# Sourced (POSIX sh) by scripts/telemetry-overhead and
# telemetry-dashboards-check. Every check runs on the production stack, the
# Compose project petri-telemetry on its external volume, the one every run
# exports to (user decision, 2026-10-07): a check isolates its data by the run
# IDs it creates, never by an empty store. Nothing here, and no caller, stops,
# pauses, restarts or takes down the stack or removes its volume.
# The caller sets compose_file (telemetry/compose.yaml).

compose() {
  # shellcheck disable=SC2154 # compose_file is the caller's.
  docker compose -f "$compose_file" "$@"
}

# stack_up: starts the stack as `make telemetry-up` does (a running stack is
# left as it is), then waits up to 240 s for Prometheus, Loki and Grafana to
# be ready and the collector to accept OTLP/HTTP on 127.0.0.1:4318.
stack_up() {
  volume=${PETRI_TELEMETRY_VOLUME:-petri-telemetry}
  docker volume inspect "$volume" >/dev/null 2>&1 || docker volume create "$volume" >/dev/null
  compose up -d --quiet-pull >/dev/null 2>&1
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
