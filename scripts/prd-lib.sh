#!/bin/sh

prd_error_count=0

prd_error() {
  printf 'prd-check: %s\n' "$*" >&2
  prd_error_count=$((prd_error_count + 1))
}

prd_status() {
  sed -n 's/^- Status: //p' "$1" | sed -n '1p'
}

prd_line_count() {
  awk 'END { print NR + 0 }' "$1"
}

prd_check_document() {
  prd_doc=$1
  prd_kind=$2
  prd_lines=$(prd_line_count "$prd_doc")
  if [ "$prd_lines" -gt 750 ]; then
    prd_error "$prd_doc has $prd_lines physical lines; maximum is 750"
  fi

  for prd_heading in \
    '## Goal' \
    '## Scope' \
    '## Non-Goals' \
    '## Inputs and Existing-Code Interactions' \
    '## Boundaries and Abstraction Layers' \
    '## Separation of Concerns and Decomposition' \
    '## Tech Debt and Spaghetti-Code Implications' \
    '## Documentation Impact and Synchronization' \
    '## Implementation or Decision Tasks' \
    '## Verification and Observable Success Criteria' \
    '## Current Status'
  do
    grep -Fqx "$prd_heading" "$prd_doc" || prd_error "$prd_doc is missing heading: $prd_heading"
  done

  grep -Eq '^- \[[ x]\] Affected durable documentation is created, updated, or synchronized, or a no-change rationale is recorded\.$' "$prd_doc" || \
    prd_error "$prd_doc is missing the documentation synchronization gate"

  if [ "$prd_kind" = master ]; then
    grep -Fqx '## Stage Order and Links' "$prd_doc" || prd_error "$prd_doc is missing heading: ## Stage Order and Links"
    grep -Fqx '## Cross-Stage Decisions' "$prd_doc" || prd_error "$prd_doc is missing heading: ## Cross-Stage Decisions"
  else
    grep -Fq '[Master PRD](master-prd.md)' "$prd_doc" || prd_error "$prd_doc does not link to master-prd.md"
  fi

  prd_doc_status=$(prd_status "$prd_doc")
  case $prd_doc_status in
    Draft|Ready|'In Progress'|Complete) ;;
    *) prd_error "$prd_doc has invalid or missing status: ${prd_doc_status:-<empty>}" ;;
  esac

  if [ "$prd_doc_status" = Complete ] && grep -Eq '^- \[ \]' "$prd_doc"; then
    prd_error "$prd_doc is Complete but contains unchecked tasks"
  fi
}

prd_check_set() {
  prd_set=$1
  prd_location=$2
  prd_slug=$(basename "$prd_set")
  case $prd_slug in
    ''|*[!a-z0-9-]*|-*|*-|*--*) prd_error "$prd_set does not use a valid kebab-case slug" ;;
  esac

  prd_master=$prd_set/master-prd.md
  [ -f "$prd_master" ] || { prd_error "$prd_set is missing master-prd.md"; return; }
  prd_check_document "$prd_master" master

  prd_review_count=$(sed -n 's/^- Review Count: //p' "$prd_master" | sed -n '1p')
  case $prd_review_count in
    ''|*[!0-9]*)
      prd_error "$prd_master has an invalid or missing Review Count"
      prd_review_count=0
      ;;
  esac
  if [ "$prd_review_count" -gt 3 ]; then
    prd_error "$prd_master Review Count exceeds the maximum of 3"
  fi

  prd_review_status=$(sed -n 's/^- Review Status: //p' "$prd_master" | sed -n '1p')
  case $prd_review_status in
    DRAFT|APPROVED) ;;
    *) prd_error "$prd_master has an invalid or missing Review Status: ${prd_review_status:-<empty>}" ;;
  esac

  prd_stage_count=0
  prd_expected_stage=1
  prd_all_draft=1
  prd_all_ready=1
  prd_all_exactly_ready=1
  prd_all_complete=1
  for prd_stage in "$prd_set"/stage-*.md; do
    [ -e "$prd_stage" ] || continue
    prd_stage_count=$((prd_stage_count + 1))
    prd_stage_number=$(printf '%02d' "$prd_expected_stage")
    prd_stage_base=$(basename "$prd_stage")
    case $prd_stage_base in
      "stage-$prd_stage_number-"*.md) ;;
      *) prd_error "$prd_stage is out of order or has an invalid stage number; expected stage-$prd_stage_number-<slug>.md" ;;
    esac
    prd_stage_slug=${prd_stage_base#stage-[0-9][0-9]-}
    prd_stage_slug=${prd_stage_slug%.md}
    case $prd_stage_slug in ''|*[!a-z0-9-]*|-*|*-|*--*) prd_error "$prd_stage has an invalid kebab-case stage slug" ;; esac

    prd_check_document "$prd_stage" stage
    prd_stage_status=$(prd_status "$prd_stage")
    [ "$prd_stage_status" = Draft ] || prd_all_draft=0
    case $prd_stage_status in Ready|'In Progress'|Complete) ;; *) prd_all_ready=0 ;; esac
    [ "$prd_stage_status" = Ready ] || prd_all_exactly_ready=0
    [ "$prd_stage_status" = Complete ] || prd_all_complete=0

    grep -Fq "($prd_stage_base)" "$prd_master" || prd_error "$prd_master does not link to $prd_stage_base"
    grep -Fq "($prd_stage_base) — \`$prd_stage_status\`" "$prd_master" || prd_error "$prd_master does not summarize $prd_stage_base with status $prd_stage_status"

    prd_stage_dependencies=$(sed -n 's/^- Depends on: //p' "$prd_stage" | sed -n '1p')
    if [ -z "$prd_stage_dependencies" ]; then
      prd_error "$prd_stage is missing Depends on metadata"
    elif [ "$prd_stage_dependencies" != None ]; then
      prd_old_ifs=$IFS
      IFS=', '
      # shellcheck disable=SC2086 # Intentional split of comma/space dependency metadata.
      set -- $prd_stage_dependencies
      IFS=$prd_old_ifs
      for prd_dependency in "$@"; do
        [ -n "$prd_dependency" ] || continue
        case $prd_dependency in
          stage-[0-9][0-9]-*.md) ;;
          *) prd_error "$prd_stage has invalid dependency '$prd_dependency'"; continue ;;
        esac
        [ -f "$prd_set/$prd_dependency" ] || prd_error "$prd_stage depends on missing $prd_dependency"
        prd_dependency_number=${prd_dependency#stage-}
        prd_dependency_number=${prd_dependency_number%%-*}
        prd_dependency_number=$(printf '%s' "$prd_dependency_number" | sed 's/^0*//')
        [ -n "$prd_dependency_number" ] || prd_dependency_number=0
        if [ "$prd_dependency_number" -ge "$prd_expected_stage" ]; then
          prd_error "$prd_stage dependency $prd_dependency is not an earlier stage"
        fi
        if [ -f "$prd_set/$prd_dependency" ]; then
          case $prd_stage_status in
            'In Progress'|Complete)
              prd_dependency_status=$(prd_status "$prd_set/$prd_dependency")
              [ "$prd_dependency_status" = Complete ] || prd_error "$prd_stage cannot be $prd_stage_status until dependency $prd_dependency is Complete"
              ;;
          esac
        fi
      done
    fi
    prd_expected_stage=$((prd_expected_stage + 1))
  done
  [ "$prd_stage_count" -gt 0 ] || prd_error "$prd_set must contain at least one stage"

  prd_master_stage_count=0
  # shellcheck disable=SC2016 # Backticks are literal Markdown.
  prd_master_stages=$(sed -n 's/^.*(\(stage-[0-9][0-9]-[a-z0-9-]*\.md\)) — `[^`]*`$/\1/p' "$prd_master")
  for prd_master_stage in $prd_master_stages; do
    prd_master_stage_count=$((prd_master_stage_count + 1))
    [ -f "$prd_set/$prd_master_stage" ] || prd_error "$prd_master summarizes missing $prd_master_stage"
  done
  [ "$prd_master_stage_count" -eq "$prd_stage_count" ] || prd_error "$prd_master stage summary count does not match the stage files"

  prd_master_status=$(prd_status "$prd_master")
  case $prd_master_status in
    Draft)
      [ "$prd_review_status" = DRAFT ] || prd_error "Draft PRD $prd_set must have Review Status DRAFT"
      [ "$prd_all_draft" -eq 1 ] || prd_error "Draft PRD $prd_set requires every stage to be Draft"
      ;;
    Ready)
      [ "$prd_review_status" = APPROVED ] || prd_error "Ready PRD $prd_set requires Review Status APPROVED"
      [ "$prd_review_count" -gt 0 ] || prd_error "Approved PRD $prd_set requires at least one review"
      [ "$prd_all_exactly_ready" -eq 1 ] || prd_error "Ready PRD $prd_set requires every stage to be Ready"
      ;;
    'In Progress'|Complete)
      [ "$prd_review_status" = APPROVED ] || prd_error "$prd_master_status PRD $prd_set requires Review Status APPROVED"
      [ "$prd_review_count" -gt 0 ] || prd_error "Approved PRD $prd_set requires at least one review"
      [ "$prd_all_ready" -eq 1 ] || prd_error "$prd_master_status PRD $prd_set has a stage that is not Ready, In Progress, or Complete"
      ;;
  esac
  if [ "$prd_master_status" = Complete ]; then
    [ "$prd_all_complete" -eq 1 ] || prd_error "Complete PRD $prd_set has an incomplete stage"
  fi
  if [ "$prd_location" = archive ] && [ "$prd_master_status" != Complete ]; then
    prd_error "archived PRD $prd_set is not Complete"
  fi
}

prd_check_index() {
  prd_index_root=$1
  prd_index_location=$2
  prd_index=$prd_index_root/$prd_index_location/README.md
  [ -f "$prd_index" ] || { prd_error "$prd_index is missing"; return; }
  for prd_index_set in "$prd_index_root/$prd_index_location"/*; do
    [ -d "$prd_index_set" ] || continue
    prd_index_slug=$(basename "$prd_index_set")
    grep -Fq "[$prd_index_slug]($prd_index_slug/master-prd.md)" "$prd_index" || prd_error "$prd_index does not link to $prd_index_slug"
    prd_index_status=$(prd_status "$prd_index_set/master-prd.md")
    grep -Fq "[$prd_index_slug]($prd_index_slug/master-prd.md) — $prd_index_status" "$prd_index" || prd_error "$prd_index has a stale status for $prd_index_slug"
  done

  prd_index_directory_count=0
  for prd_index_set in "$prd_index_root/$prd_index_location"/*; do
    [ -d "$prd_index_set" ] || continue
    prd_index_directory_count=$((prd_index_directory_count + 1))
  done
  prd_index_link_count=$(grep -Ec '^- \[[^]]+\]\([^)]*/master-prd\.md\) — ' "$prd_index" || true)
  [ "$prd_index_link_count" -eq "$prd_index_directory_count" ] || prd_error "$prd_index contains stale or duplicate PRD links"
}
