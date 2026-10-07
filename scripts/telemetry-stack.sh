# Sourced (POSIX sh) by scripts/telemetry-overhead and
# telemetry-dashboards-check. Every check runs on the production stack, the
# Compose project petri-telemetry on its external volume petri-telemetry, the
# one every run exports to (user decision, 2026-10-07): a check isolates its
# data by the run IDs it creates, never by an empty store. Project and volume
# are pinned whatever COMPOSE_PROJECT_NAME, PETRI_TELEMETRY_PROJECT or
# PETRI_TELEMETRY_VOLUME the caller inherits (those select
# scripts/telemetry-verify's scratch stack). Nothing here, and no caller,
# stops, pauses, restarts, recreates or takes down the stack or removes its
# volume.
# The caller sets compose_file (telemetry/compose.yaml).

compose() {
  # shellcheck disable=SC2154 # compose_file is the caller's.
  PETRI_TELEMETRY_VOLUME=petri-telemetry docker compose -p petri-telemetry -f "$compose_file" "$@"
}

# stack_up: makes no `up` call when every service runs, so a configuration
# that differs from the checkout that started the stack recreates nothing;
# otherwise starts the missing or stopped services without recreating any
# container. Then waits up to 240 s for Prometheus, Loki and Grafana to be
# ready and the collector to accept OTLP/HTTP on 127.0.0.1:4318.
stack_up() {
  docker volume inspect petri-telemetry >/dev/null 2>&1 || docker volume create petri-telemetry >/dev/null
  [ "$(compose ps --status running --services 2>/dev/null | sort)" = "$(compose config --services | sort)" ] ||
    compose up -d --no-recreate --quiet-pull >/dev/null 2>&1
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
