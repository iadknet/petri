# syntax=docker/dockerfile:1
# The base index digest is the official Docker Library Python 3.12 slim-bookworm
# image resolved on 2026-08-29. It supports both linux/amd64 and linux/arm64.
FROM python:3.12-slim-bookworm@sha256:0f5b26b9518d002b6173fd61daad821fa340635ebfec5bba471013f9ca114579

ARG SKILLSPECTOR_VERSION=2.11.0
ARG SKILLSPECTOR_COMMIT=b7241089d7ec15d8b30df980dacbb428214732b9
ARG SKILLSPECTOR_ARCHIVE_SHA256=1370abe7a76dbe90d6bddf7d1e0353cc58ae9442eece3b03aa7ba75830210dfc
ARG UV_VERSION=0.12.3

LABEL org.opencontainers.image.title="petri-skillspector" \
      org.opencontainers.image.version="${SKILLSPECTOR_VERSION}" \
      org.opencontainers.image.revision="${SKILLSPECTOR_COMMIT}" \
      org.opencontainers.image.source-archive-sha256="${SKILLSPECTOR_ARCHIVE_SHA256}"

ENV PYTHONDONTWRITEBYTECODE=1 \
    PYTHONUNBUFFERED=1 \
    UV_LINK_MODE=copy \
    PATH=/opt/skillspector/.venv/bin:${PATH}

WORKDIR /opt/skillspector
COPY . /opt/skillspector

# Upstream's committed uv.lock is the dependency authority. The wrapper builds
# this image only after verifying the source archive hash and source contract.
RUN python -m pip install --no-cache-dir "uv==${UV_VERSION}" \
    && uv sync --locked --no-dev \
    && test "$(.venv/bin/skillspector --version)" = "SkillSpector v${SKILLSPECTOR_VERSION}" \
    && groupadd --gid 65532 skillspector \
    && useradd --uid 65532 --gid 65532 --create-home --home-dir /home/skillspector --shell /usr/sbin/nologin skillspector \
    && chmod -R a+rX /opt/skillspector

USER 65532:65532
WORKDIR /tmp
