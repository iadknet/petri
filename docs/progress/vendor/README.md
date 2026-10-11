# Vendored files for the progress page

`../index.html` loads these files from this folder, so the page has no build step and no network dependency. Each file is pinned to an exact version, and each `<script>` tag carries the SHA-384 below as its `integrity` attribute.

## Upgrading a file

1. Run `npm pack <package>@<version>` in an empty folder.
2. Copy the file across, renamed with its version.
3. Update the table below and the `integrity` attribute in `index.html`.
4. The repository's seven-day minimum package age applies (see `SECURITY.md`).

## Files

| File | Source | Version | Licence | SHA-256 |
| --- | --- | --- | --- | --- |
| `plot-0.6.17.umd.min.js` | npm `@observablehq/plot`, `dist/plot.umd.min.js` | 0.6.17 | ISC (`plot.LICENSE`) | `4358086467740777dd788d6b27a95cebdbaefdd50c730a3060117073bd7134cb` |
| `d3-7.9.0.min.js` | npm `d3`, `dist/d3.min.js` | 7.9.0 | ISC (`d3.LICENSE`) | `f2094bbf6141b359722c4fe454eb6c4b0f0e42cc10cc7af921fc158fceb86539` |
| `geist-sans-latin-400-normal.woff2` | npm `@fontsource/geist-sans` | 5.2.5 | OFL-1.1 (`geist-sans.LICENSE`) | `e6ef6abb39c762ed2259cc814e6f16888c540cd84835962227b56ab2354f27f1` |
| `geist-sans-latin-500-normal.woff2` | npm `@fontsource/geist-sans` | 5.2.5 | OFL-1.1 | `40073e90816315c92e4f4381bd50b6fdc950b22b0dd010a4179046cf588d4f12` |
| `geist-sans-latin-600-normal.woff2` | npm `@fontsource/geist-sans` | 5.2.5 | OFL-1.1 | `9d99fbd791968493fa507ac846de561cee47b00f8100c23cad333b3cb78392d6` |
| `jetbrains-mono-latin-wght-normal.woff2` | npm `@fontsource-variable/jetbrains-mono` | 5.2.8 | OFL-1.1 (`jetbrains-mono.LICENSE`) | `18be452724bfdc236c074ca94a249a7f41a86752c7d04ab258ce9ed5651f6a7e` |

## Registry integrity of the npm tarballs

- `@observablehq/plot@0.6.17`: `sha512-/qaXP/7mc4MUS0s4cPPFASDRjtsWp85/TbfsciqDgU1HwYixbSbbytNuInD8AcTYC3xaxACgVX06agdfQy9W+g==`
- `d3@7.9.0`: `sha512-e1U46jVP+w7Iut8Jt8ri1YsPOvFpg46k+K8TpCb0P+zjCkjkPnV7WzfDJzMHy1LnA+wj5pLT1wjO901gLXeEhA==`

## Known risk

Plot 0.6.17 is from Feb 2025 and has had no release since. Its API is stable, and this copy keeps working whatever happens upstream.
