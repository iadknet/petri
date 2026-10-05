# E0 baseline identity: the deterministic projection of an input-opportunity
# summary. Strips the fields the run 1 plan's baseline-identity rule excludes
# (timing, thread count, source revision, the raw file's path); keeps the raw
# record's sha256 and byte count, every replicate row, verdict, family block
# and the discovery baseline. Used unchanged for the launch-revision pilot and
# for the post-E1 reproduction.
del(.wall_secs, .threads, .source_revision, .raw.path)
