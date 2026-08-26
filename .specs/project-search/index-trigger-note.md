# `codanna` index trigger decision

Decision: **on demand**.

Project search does not start `codanna` while reading a checkout. A linked checkout may be changing,
and an implicit timer or git hook would make the editor launch an unexpected process and still leave a
race between the index and the tree being searched. The search engine therefore consumes an existing
index when one is available and falls back to content search when it is not.

`SearchEngine::index_command` is the explicit refresh boundary. It emits `codanna index <checkout>`
and refuses a run clone. The policy is represented by `CODANNA_INDEX_TRIGGER` and
`IndexTrigger::OnDemand` in `crates/locus-core/src/search.rs`.

The policy can be revisited if indexing latency or repository size makes on-demand refresh too costly;
there is intentionally no schedule or automatic git-change watcher in this feature.
