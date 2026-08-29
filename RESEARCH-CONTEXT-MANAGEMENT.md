# Context & Token Management for AI Agents — Research Synthesis

Compiled 2026-08-29. External sources only — no local repository content, no session history.
Every claim below carries its source URL and, where available, date. Claims are labeled
**[retrieved]** (read directly from the source during this research) or **[claimed]** (the
vendor's own number, not independently verified). Gaps are stated, not filled.

Method: `ddgs` (search), `trafilatura` / raw `raw.githubusercontent.com` fetches (pages),
`ctx7` (library docs), `paper` / `paper-search` (arXiv), `scrapling stealthy-fetch` (one 403
block). The skill that prescribes these tools: `web-research`.

---

## 1. The tool landscape

### 1.1 The four named tools

**Headroom** — github.com/headroomlabs-ai/headroom (Trendshift-tracked; reported 29.5k+ stars
as of June 2026, explainx.ai). A *compression layer* deployable as library, local proxy,
agent wrapper (`headroom wrap claude|codex|cursor|…`), or MCP server. [retrieved from README]

- **ContentRouter** detects content type and picks a compressor: SmartCrusher (JSON),
  CodeCompressor (AST-aware), or Kompress-v2-base — a small learned text-compression model
  served from Hugging Face. [retrieved]
- **CacheAligner** detects volatile content that would bust provider KV-cache prefixes and
  warns; it deliberately *never rewrites* prompts. [retrieved]
- **CCR (reversible compression)**: originals are cached locally; the LLM gets a
  `headroom_retrieve` tool to fetch them on demand. Nothing is deleted, nothing invented.
  [retrieved]
- Cross-agent memory store with auto-dedup; `headroom learn` mines failed sessions and writes
  corrections into `CLAUDE.local.md`. [retrieved]
- Output-side trimming: drops ceremony/restated code from what the model *writes back*.
  [retrieved]
- Claimed: 60–95% fewer tokens for JSON, 15–20% for coding agents; workload table 47–92%;
  GSM8K accuracy delta 0.000 at n=100. [claimed]
- Independently measured: none found. The JetBrains benchmark series has not tested it.
  [retrieved absence]

**RTK (Rust Token Killer)** — github.com/rtk-ai/rtk. A single Rust binary CLI proxy, 100+
commands, <10ms overhead, wired in via agent command-interception hooks. [retrieved]

- Mechanism is *rewriting command output*, not the conversation: `ls`/`tree` → tree + counts;
  `cat` → signatures and structure over full bodies; `grep` → grouped by file, truncated
  lines; `git diff` → headers stripped; test runners → failures only, passes collapsed to a
  count. [retrieved]
- Notably honest self-assessment in its README: "Bash output is **one contributor to input
  tokens**… The reduction dilutes at every step," token counts are `bytes / 4` with no
  tokenizer, and "the percentages are reliable but the absolute token numbers are
  approximate." [retrieved]
- Claimed: up to 90% of bash output. [claimed]
- Measured (JetBrains, 80 paired tasks): **+7.6% cost** — i.e. slightly *worse* than
  baseline. blog.jetbrains.com/ai/2026/07/ponytail-skill-claude-tested/ [retrieved]

**Caveman** — github.com/JuliusBrussee/caveman. Two products in one repo. [retrieved]

- The *skill* (output-side): the agent answers in compressed "caveman-speak" — short
  fragments, no preamble — while code, commands, and errors stay byte-exact. Ships as a
  skill/plugin for 30+ agents. [retrieved]
- The *proxy* (input-side): shrinks what the agent reads before every provider call, with
  byte-exact recovery. [retrieved]
- `cavemem` persistent memory: hybrid BM25 + local vectors, exposed as `memory_search` /
  `memory_save` tools with relevant recall auto-injected each turn; subagent output is
  compressed ~60% before injection back into main context (andrew.ooo review).
  [retrieved via andrew.ooo/posts/caveman-claude-code-skill-token-savings-review/]
- Claimed: 33.2% fewer provider-reported input tokens in a pinned Claude Code benchmark
  (`docs/WRAP-BENCHMARK.md`). [claimed]
- Measured (JetBrains): −8.5% code vs **−65% advertised** — the weakest advertised-vs-measured
  gap in the series. [retrieved]
- Counter-evidence from a competing tool's own benchmark: as a terse-prose control, caveman
  measured **+7% tokens / +3% cost** vs no-skill baseline (ponytail's 12-task agentic bench).
  github.com/DietrichGebert/ponytail [retrieved]

**Ponytail** — github.com/DietrichGebert/ponytail + ponytail.dev. A *pure prompt skill* — no
pipeline, no proxy. It injects a ruleset ("the lazy senior dev") plus six commands, via a
SessionStart hook, every session. [retrieved]

- The mechanism is a decision ladder the model runs before writing: does this need to exist →
  is it already in the codebase → stdlib → native platform feature → installed dependency →
  can it be one line → write the minimum. Validation, error handling, security, and
  accessibility are explicitly off the chopping block. [retrieved — JetBrains part 3 + README]
- Claimed (own benchmark, 12 tasks, Haiku 4.5, scored on `git diff`): −54% LOC (up to 94%
  where the agent over-builds; near zero where code is already minimal), −22% tokens, −20%
  cost, −27% time. The README documents its own methodology fixes: a contamination bug where
  a SessionStart hook fired on the baseline arm, and an earlier "chatty baseline" critique
  (issue #126) they rebuilt the benchmark to answer. [claimed, unusually well-documented]
- Measured (JetBrains, 80 paired SkillsBench tasks, Claude Sonnet 5, medium effort): −15%
  code, **−10.3% cost (p = 0.004)**, −11% time, no detectable quality difference. The first
  tool in the series with a statistically solid cost saving — but roughly a quarter to a half
  of the advertised effect, and the effect appears only "where there was room to over-build."
  [retrieved]

### 1.2 Other tools in the category

| Tool | Layer | Mechanism (one sentence) | Source |
| --- | --- | --- | --- |
| Compresr **Context-Gateway** | API proxy | Sits between agent and LLM API; compresses conversation history in the background when it gets long, so compaction never blocks the turn | github.com/Compresr-ai/Context-Gateway [retrieved, repo description] |
| **Serena** | MCP toolkit | LSP/symbol-level code retrieval: `find_symbol` with name paths, depth, `include_body` — returns signatures and line ranges instead of whole files | ctx7 /oraios/serena, src/serena/tools/symbol_tools.py [retrieved] |
| **Context7** (Upstash) | MCP server | Just-in-time version-specific library docs pulled from source and placed into the prompt, replacing stale training-data recall | github.com/upstash/context7 [retrieved] |
| **mem0** | Memory layer | `add()` pipeline: one LLM call extracts new facts from messages → hash dedup → embed → vector + entity store; relevant memories retrieved per turn | ctx7 /mem0ai/mem0 [retrieved] |
| **Letta** (ex-MemGPT) | Agent framework | Memory blocks: named text blocks held in context that the agent itself edits via API (`update_block`), plus archival storage outside the window | ctx7 /letta-ai/letta [retrieved] |
| **LLMLingua** (Microsoft) | Library | Model-based prompt compression using a small LM's perplexity: budget controller + token-level iterative compression + instruction-tuned alignment | github.com/microsoft/LLMLingua + arXiv 2310.05736 [retrieved] |
| **claude-mem** | Claude Code plugin | Captures tool usage across sessions and generates semantic summaries so context survives session boundaries | ctx7 /thedotmack/claude-mem [retrieved] |
| **OMNI** | Proxy/handle layer | "Your agent pays twice for output it has already seen. OMNI returns a handle instead" — content-addressed dedup with byte-exact recovery | github.com/topics/context-compression [retrieved, repo description] |
| **Platform built-ins** (Anthropic) | API feature | Context editing (auto-clears stale tool calls/results near the token limit) + file-based memory tool (client-side CRUD, persists across conversations) | claude.com/blog/context-management, 2025-09-29 [retrieved] |
| **Claude Code compaction** | Agent built-in | Nearing the limit, history is summarized (keeping decisions, bugs, implementation state, 5 most-recent files); tool-result clearing is the "lightest touch" version, shipped as platform context editing | anthropic.com/engineering/effective-context-engineering-for-ai-agents [retrieved] |

A GitHub topics sweep (github.com/topics/context-compression) shows the category is crowded —
dozens of proxies, handles-based stores, and "reversible compression" layers — with the same
handful of primitives recombined. [retrieved]

---

## 2. The academic evidence

### 2.1 Why long context fails

**Lost in the Middle** — Liu et al., arXiv 2307.03172 (TACL). Performance on multi-document QA
and key-value retrieval "degrades significantly when changing the position of relevant
information"; models do not robustly use long contexts, with a U-shaped positional curve
(beginnings and ends favored). [retrieved abstract]

**Context Rot** — Chroma research, research.trychroma.com/context-rot (July 2025). Controlled
study across **18 LLMs**: even with task complexity held constant and only input length
varying, "model performance degrades as input length increases, often in surprising and
non-uniform ways." NIAH (needle-in-a-haystack) is lexical retrieval and flatters long
contexts; semantic (non-lexical) matching tasks and distractor-rich haystacks degrade much
faster. Cites NoLiMa and MRCR as convergent evidence. [retrieved]

**How Long Contexts Fail** — Drew Breunig, dbreunig.com, 2025-06-22. Taxonomy of four failure
modes: **poisoning** (a hallucination enters context and gets referenced repeatedly —
documented in the Gemini 2.5 technical report's Pokémon runs), **distraction** (past ~100k
tokens the Gemini agent "favor[ed] repeating actions from its vast history rather than
synthesizing novel plans"; a Databricks study found correctness falling near 32k for Llama
3.1 405b and earlier for smaller models), **confusion** (superfluous content yields
low-quality responses), **clash** (parts of the context directly disagree — e.g. MCP tool
descriptions vs your prompt). [retrieved]

### 2.2 Compression strategies

**LLMLingua** — Jiang et al., arXiv 2310.05736 (EMNLP 2023). Coarse-to-fine prompt
compression: a budget controller preserves semantic integrity under high ratios; a
token-level iterative compression algorithm models interdependence; distribution alignment
via instruction tuning. Evaluated on GSM8K and BBH. Uses a *small* LM's perplexity as the
compressor signal. [retrieved abstract]

**Selective Context** — Li et al., arXiv 2304.12102. Filter context by **self-information**:
lexical units (tokens/phrases/sentences) with low information content are dropped, extending
effective context for summarization and QA. [retrieved abstract]

**RECOMP** — Xu et al., arXiv 2310.04408 (ICLR 2024). *Trained* compressors for RAG:
extractive (select sentences) and abstractive (summarize) variants, trained against end-task
likelihood (does the LM still produce the right answer with the compressed doc prepended?),
plus selective augmentation (retrieve only when needed). [retrieved]

**ICAE (In-context Autoencoder)** — arXiv 2307.06945 (ICLR 2024). Compresses context into a
few *soft* memory slots via a LoRA encoder pretrained on autoencoding + text continuation —
compression into the model's latent space rather than its token space. [retrieved outline]

### 2.3 Memory & paging

**MemGPT** — Packer et al., arXiv 2310.08560. The OS analogy formalized: an LLM operating
system with **main context** (prompt tokens), a **queue manager** that pages information
between main context and external storage, self-editing memory via function calls, and
control flow via function chaining. Evaluated on deep-memory-retrieval conversation
consistency and multi-document QA / nested KV retrieval. [retrieved outline] This is the
academic ancestor of every memory tool above (Letta is its productization).

### 2.4 KV-cache level (model internals, not curation)

These operate *below* the token level — they change what the inference engine keeps and
attends to, not what the agent puts in the window. Category error risk: they reduce compute
and memory, not the number of tokens in your prompt.

- **StreamingLLM** — Xiao et al., arXiv 2309.17453 (ICLR 2024). Window attention fails
  without early tokens ("attention sinks"); a rolling KV cache + sink tokens gives stable
  streaming generation far beyond the training window. [retrieved outline]
- **H2O** — Zhang et al., arXiv 2306.14048. Heavy-hitter oracle: a small set of
  attention-critical KV entries plus recent tokens retains quality, enabling large KV-cache
  reduction. [retrieved outline]
- **SnapKV** — arXiv 2404.14469. The prompt's own "observation window" (its attention
  pattern) predicts which KV entries matter; cluster and pool them *before* generation.
  [retrieved outline]

### 2.5 Retrieval vs long context

**Long-Context LLMs Meet RAG** — Google, arXiv 2410.05983. Long-context models still benefit
from RAG: the report's whole structure ("the effect of retrieved context size on RAG
performance," "the interplay of retrieval quality and LLM capabilities," "the importance of
hard negatives," training for robustness) is an argument that retrieval *remains necessary*
even when the window technically fits everything. [retrieved outline + intro]

**Structured Context Engineering for File-Native Agentic Systems** — McMillan, 2026 (via
simonwillison.net/tags/context-engineering). 9,649 experiments, 11 models, 4 serialization
formats (YAML/Markdown/JSON/TOON), schemas from 10 to 10,000 tables. Findings: model
capability dominates outcomes; filesystem-based retrieval helps frontier models but not
weaker open-weights models; and the "**grep tax**" — TOON, a token-minimal format, *cost
more* tokens end-to-end because models are unfamiliar with it and burn iterations
mis-parsing it. Token-density of a format matters less than the model's familiarity with it.
[retrieved via Willison's commentary]

### 2.6 Coverage gaps (honest)

`paper-search` returned empty for **LLoCo** ("Learning Long Contexts Offline"), **SARA**
(ACL 2026 selective/adaptive RAG with compression), and **Compressed Context Memory for
Online Language Model Interaction** (ICLR'24) after multiple query shapes — search failure,
not absence. Not covered; do not cite numbers for them from memory.

---

## 3. Practitioner positions

**Anthropic — Effective context engineering for AI agents** (anthropic.com/engineering/
effective-context-engineering-for-ai-agents, 2025-09-29). [retrieved]

- Context is a **finite resource with diminishing marginal returns**; models have an
  "**attention budget**" (n² pairwise attention, training-data distribution skewed to short
  sequences). "Context rot" applies across all models.
- The goal: "**the smallest set of high-signal tokens** that maximize the likelihood of some
  desired outcome."
- System prompts at the "**right altitude**"; minimal viable tool sets; curated canonical
  examples rather than laundry lists of edge cases.
- **Just-in-time retrieval** over pre-loading: keep lightweight identifiers (paths, queries,
  links), load data at runtime; "progressive disclosure" through exploration. Claude Code's
  hybrid: CLAUDE.md dropped in up front, glob/grep for everything else.
- Long-horizon techniques: **compaction** (summarize near the limit, keep decisions/bugs/
  implementation state + 5 recent files; tune for recall first, then precision; tool-result
  clearing as the safe first lever), **structured note-taking** (NOTES.md / to-do lists /
  memory tool), **sub-agent architectures** (sub-agents burn tens of thousands of tokens and
  return condensed 1–2k summaries; isolation keeps search noise out of the lead context).
- "Waiting for larger context windows… is likely that for the foreseeable future, context
  windows of all sizes will be subject to context pollution."

**Anthropic — Managing context on the Claude Developer Platform** (claude.com/blog/
context-management, 2025-09-29). [retrieved] Context editing + memory tool as products.
Self-reported numbers: memory tool + context editing **+39%** over baseline on an internal
agentic-search eval (editing alone **+29%**); in a 100-turn web-search eval, context editing
let agents finish workflows that otherwise died of context exhaustion while **reducing token
consumption 84%**. [claimed — vendor's own evals, no third-party replication found]

**Manus — Context Engineering for AI Agents: Lessons from Building Manus** (manus.im blog,
2025-07-18, Yichao "Peak" Ji). [retrieved]

- "If I had to choose just one metric… the **KV-cache hit rate** is the single most important
  metric for a production-stage AI agent." Cached vs uncached Claude Sonnet input: $0.30 vs
  $3.00/MTok — 10x. Keep prompt prefixes stable (no timestamps), keep context **append-only**
  with deterministic serialization, mark cache breakpoints explicitly.
- **Mask, don't remove**: dynamically loading/unloading tools mid-run invalidates the cache
  (tool defs live at the front) and confuses the model; instead mask token logits /
  state-machine tool availability. Consistent action-name prefixes (`browser_`, `shell_`)
  enable group-level masking.
- **Use the file system as context**: "unlimited in size, persistent by nature, directly
  operable by the agent itself." Any compression must be **restorable** — drop a page but
  keep its URL, omit a document but keep its path. "Any irreversible compression carries
  risk… an agent must predict the next action based on all prior state — and you can't
  reliably predict which observation might become critical ten steps later."
- **Recitation**: rewriting `todo.md` step-by-step re-places the plan at the *end* of the
  context, exploiting recency to fight lost-in-the-middle across ~50 tool-call loops.
- **Keep the wrong stuff in**: failed actions + stack traces stay; erasing failure removes
  evidence the model needs to adapt.
- **Don't get few-shotted**: uniform contexts induce mimicry and drift; introduce structured
  variation (serialization templates, phrasing, ordering noise).

**Cognition — Don't Build Multi-Agents** (cognition.ai/blog/dont-build-multi-agents, 2025).
[retrieved through principle 2; later sections did not survive extraction]

- "**Share context, and share full agent traces**, not just individual messages." Parallel
  subagents make conflicting implicit decisions and the combiner inherits the mess
  (the Flappy Bird example).
- "**Actions carry implicit decisions**, and conflicting decisions carry bad results."
- Positions context engineering as "effectively the #1 job of engineers building AI agents."
- Directly opposed to OpenAI Swarm / Microsoft Autogen-style multi-agent architectures.

**LangChain — Context Engineering** (blog.langchain.dev/context-engineering/, Dec 2024).
[retrieved] The **write / select / compress / isolate** taxonomy, which became the field's
default vocabulary. Quotes Karpathy's LLM-as-OS framing ("the LLM is like the CPU and its
context window is like RAM") and his definition of context engineering as "the delicate art
and science of filling the context window with just the right information for the next step."
Cites Breunig's failure taxonomy and Cognition's principles. [Karpathy's original tweet was
not directly retrievable; the definition here is via LangChain's quote of it]

**Simon Willison** (simonwillison.net/tags/context-engineering). [retrieved] Carries the
field's running commentary: the McMillan structured-context study (above), Liz Fong-Jones'
operational metaphor — the model is "a junior developer… prone to forgetting anything but the
most recent hour"; notes are sticky notes that must be *periodically cleared or they pile up*
— and Matt Webb's "**context plumbing**": the engineering job is *moving* context from where
it arises to where the model needs it, at the right time.

**JetBrains — "token saver" benchmark series** (blog.jetbrains.com/ai/, parts 1–3, June–July
2026). [retrieved via part 3] The only independent paired A/B measurements of the community
tools found anywhere: Harbor harness, Docker sandboxes, SkillsBench 80 paired tasks,
verifier-scored, contamination audits on every trial. Results: caveman −8.5% (advertised
−65%), rtk **+7.6%** (advertised −60–90%), ponytail −10.3% cost with p=0.004 (advertised
−54%). Their methodology explicitly adopted ponytail's own contamination-audit practice —
the tools' authors and the benchmarkers are converging on stricter measurement than the
rest of the space exhibits.

---

## 4. The underlying mechanisms (the gimmicks stripped out)

Everything above reduces to eight primitives. Most tools are a bundle of two to four.

| # | Mechanism | What it actually does | Exemplars |
| --- | --- | --- | --- |
| 1 | **Filter/drop** | Remove content before it enters the window: stale tool results, low self-information spans, superfluous logs | Anthropic context editing, Selective Context, RTK (failure-only test output) |
| 2 | **Reformat** | Same content, cheaper/deterministic shape: signatures over bodies, grouped matches, stable JSON key order | RTK, Headroom SmartCrusher, Manus's serialization rules |
| 3 | **Retrieve, don't hold** | Keep identifiers; load content at use-time (just-in-time), or symbol-level views instead of whole files | Claude Code glob/grep, Context7, Serena, RAG |
| 4 | **Page out + address** | Persist state outside the window with addresses (paths, handles); reload on demand; originals stay recoverable | MemGPT/Letta, memory tool, Headroom CCR, OMNI, Manus's file system |
| 5 | **Summarize** | Lossy distillation of history or documents; can be tuned (recall-first) or *trained* against end-task likelihood | Compaction, mem0 extraction, RECOMP, LLMLingua, ICAE (soft slots) |
| 6 | **Cache-align** | Protect provider prefix caches: stable prefixes, append-only, explicit breakpoints, no mid-history edits | Manus's #1 rule, Headroom CacheAligner, MemGPT's append design |
| 7 | **Constrain output** | Reduce what the model *writes back*: style rules (prompt-side) or logit masking (decode-side) | Caveman skill, Ponytail, Manus's "mask, don't remove" |
| 8 | **Isolate** | Give sub-tasks their own windows; only condensed results flow back | Anthropic sub-agents, caveman subagent compression, Manus's per-subagent budgets |

Two clarifications that cut through most vendor noise:

- **Layer separation.** Curation (what enters the window) ≠ attention (what the model uses
  effectively) ≠ inference economics (what the KV cache stores). Tools 1–5 and 8 are
  curation; the academic KV work (StreamingLLM/H2O/SnapKV) is attention/infra and does *not*
  shrink your token bill; 6 is pure economics.
- **Reversibility is the dividing line.** Manus's argument is structural: an agent must act
  on all prior state, and you cannot predict what becomes critical later, so *any
  irreversible compression is a bet against your own agent*. The best-regarded designs
  (Headroom CCR, caveman proxy, OMNI, Manus) therefore keep originals addressable and
  compress only the *view*. Anthropic's compaction is the deliberate exception — lossy, but
  tuned recall-first and applied as a last lever.

---

## 5. Common threads

1. **Context is a finite, degrading resource — treat every token as spent attention.** The
   one claim with the strongest and most convergent evidence: Lost in the Middle (2023),
   Context Rot's 18-model controlled study (2025), Breunig's distraction ceiling (~32k for
   Llama 3.1 405b), Anthropic's "attention budget." Bigger windows do not fix it; Anthropic
   explicitly says windows of all sizes will stay subject to pollution. Every serious tool
   and every serious post starts here.

2. **Don't hold what you can re-derive.** The dominant positive strategy across all three
   streams — Anthropic's just-in-time retrieval, Manus's file system + restorable
   compression, MemGPT's paging, Headroom's CCR handles, Serena's symbol views, Context7's
   docs-on-demand. This is virtual memory, re-invented at the prompt layer: keep addresses,
   not payloads. It is the only mechanism that scales context without paying the
   degradation curve, because the degraded content never enters the window.

3. **Selection beats compression; lossless-with-retrieval beats lossy summarization.**
   Google's RAG paper shows retrieval still pays even with million-token windows; the
   Chroma/McMillan results show *how* you present and select matters (format familiarity can
   outweigh token density — the "grep tax"); trained compressors (RECOMP) that optimize
   against end-task likelihood beat heuristic truncation. Where summarization is used,
   the field converged on recall-first tuning (Anthropic) and restorability (Manus).

4. **The KV/prompt cache is load-bearing infrastructure, and most tools that rewrite history
   are quietly taxing it.** Manus's 10x cached/uncached differential explains why Headroom
   ships a component whose only job is to *not* touch prompts, why append-only and
   deterministic serialization are repeated rules (Manus, RTK's own docs), and why
   compaction/context-editing must run at explicit breakpoints. Savings mechanics and cache
   mechanics fight each other; mature designs reconcile them explicitly.

5. **Advertised savings are inflated roughly 5–10× versus independent measurement — and the
   honest tools say so themselves.** The JetBrains series is the only independent paired
   A/B evidence: caveman −8.5% vs −65% claimed; rtk +7.6% cost (a *loss*) vs −60–90%
   claimed; ponytail −10.3% cost, real but ~4x smaller than advertised. RTK's own README
   pre-explains the dilution (bash output is one contributor; percentages ≠ bill). The
   pattern: input-pipeline compression attacks a *slice* of the bill; dilution is
   arithmetic, not implementation failure. Anyone quoting a headline percentage without a
   baseline definition is selling.

6. **Prompt-side (output/style) mechanisms are the weakest lever; code-side and pipeline-side
   mechanisms are the strong ones.** Caveman's measured effect (−8.5%, and +7% tokens in
   ponytail's control arm) versus ponytail's −10.3% *with p=0.004* isolates the difference:
   telling a model to *talk* shorter buys little; changing what it *builds* (the minimal-code
   ladder) and what it *reads* (RTK-style rewrites, symbol-level tools) buys measurable,
   quality-neutral savings. The mechanism that matters in ponytail is the decision ladder,
   not the terseness.

7. **Memory and compaction are the same primitive at different timescales.** Session-scoped
   compaction (Anthropic), cross-session memory (memory tool, mem0, Letta, claude-mem,
   cavemem), and note-taking (NOTES.md, todo recitation) are all "page state out of the
   window with an address, bring back a condensed view." The field's disagreements are about
   *where the paging boundary sits* and *who does the condensing* (model itself, a dedicated
   summarizer, a small compression LM, or a proxy), not about whether to page.

8. **Attention can be managed without touching tokens at all.** Manus's recitation (rewriting
   the todo list to keep the plan at the context's end), structured note-taking, and
   few-shot diversity injection are zero-pipeline, zero-infrastructure techniques with
   mechanistic grounding (lost-in-the-middle) behind them. They are the cheapest wins in the
   entire space and the most underused.

9. **Two live disagreements, unresolved:**
   - **Sub-agents.** Cognition: share full traces, avoid parallel subagents (conflicting
     implicit decisions). Anthropic: sub-agent isolation is a primary context technique
     (condensed 1–2k returns). Both cite reliability; the difference is whether condensation
     is trusted to preserve decision context. Unresolved — both are self-reported.
   - **How lossy is acceptable.** Maximal-compression camps (LLMLingua-style ratios; 90%+
     tool claims) versus the restorability school (Manus, Headroom, OMNI). The measured
     evidence (Context Rot, Lost in the Middle) supports *aggressive selection with
     addresses*, not aggressive lossy summarization — but compaction's real-world success in
     Claude Code shows lossy summarization works when tuned recall-first. Likely task-
     dependent rather than one winner.

10. **Model capability still dominates.** McMillan's 9,649-experiment study: the model itself
    was the biggest variable; filesystem-retrieval strategies only paid on frontier models.
    Every context technique is a multiplier on model quality, not a substitute for it.

---

## 6. Evidence quality summary

| Claim class | Grade | Basis |
| --- | --- | --- |
| Long-context degradation (position, length, distractors) | **Strong** | Controlled multi-model studies: Lost in the Middle, Context Rot (18 LLMs), NoLiMa/MRCR convergence |
| Retrieval remains necessary at long context | **Strong** | Google 2410.05983 + the RAG literature it reviews |
| KV-cache economics (prefix stability → 10x cost) | **Strong (arithmetic)** | Published pricing differentials; vendor-independent |
| KV-cache compression (StreamingLLM/H2O/SnapKV) | **Strong, but different layer** | Peer-reviewed; reduces compute/memory, not your token bill |
| Trained/heuristic prompt compression works | **Moderate** | Peer-reviewed (LLMLingua, RECOMP, ICAE) but on static tasks, not live agent loops |
| Platform context editing / memory tool | **Moderate (self-reported)** | Anthropic's internal evals: +39%, +29%, 84% token reduction; no third-party replication found |
| Community tool savings (rtk, caveman, ponytail) | **Measured, small, mixed** | JetBrains paired A/B (80 tasks): −8.5%, +7.6%, −10.3% (p=0.004); advert claims 5–10x higher |
| Style/prompt-only savings (terse modes) | **Weak** | Two independent measurements near zero or negative on tokens |
| Multi-agent context sharing (Cognition vs Anthropic) | **Contested** | Principled arguments both sides; no controlled study found |

---

## 7. Round 2 — the 21 reviewed sources (addendum, 2026-08-29)

A second, user-supplied source set, reviewed with the same method and labels. Six arXiv
papers, one ACM paper, one preprint, one NIST post, twelve practitioner/vendor pieces.
Verdicts below; what they change in the round-1 synthesis at the end.

### Papers

| Source | Verdict |
| --- | --- |
| **A Survey of Context Engineering for LLMs** — Mei et al., arXiv 2507.13334 (2025) | The canonical academic survey. Taxonomy: *context retrieval and generation / context processing / context management*, motivated by current limitations, performance enhancement, and resource optimization. A finer-grained map that validates LangChain's write/select/compress/isolate and this doc's eight primitives as compressions of the same structure. **[retrieved — outline + section]** |
| **Externalization in LLM Agents: Memory, Skills, Protocols and Harness Engineering** — Zhou et al., arXiv 2604.08224 | Formalizes the weights/context/harness layer stack and names the unifying mechanism: **externalization** — capability moved out of weights and the window into addressable external structure. This is round-1 thread 2 given a literature backbone. **[retrieved — outline]** |
| **AI Agents Need Memory Control Over More Context** — Bousetouane, arXiv 2601.11653 | Position paper: transcript replay grows the prompt linearly while models under-attend early tokens (cites the long-context literature); the need is *control over what is retained/retrieved/compressed*, not a larger window. Reinforces threads 1, 2, 7. **[retrieved — section]** |
| **Building Effective AI Coding Agents for the Terminal** — Bui, arXiv 2603.05344 | Systems-experience paper from a real harness. Section titles are the findings: "Context pressure as the central design constraint," "Lazy loading and bounded growth," "Adaptive context compaction," LSP-based semantic code analysis, token-efficient MCP tool discovery, "Designing for approximate outputs." Practitioner-systems evidence matching the tool mechanisms in §1. **[retrieved — outline]** |
| **Context Engineering** — Vishnyakova, arXiv 2603.09619 | Overview/position: prompt engineering "necessary but insufficient" as stateless chatbots become multi-step agents. Conceptual, no new evidence. **[retrieved — abstract + intro]** |
| **Contextual Memory Intelligence** — Wedel, arXiv 2506.05370 | Frames agent memory inside organizational-memory theory (Walsh & Ungson 1991). Conceptual/enterprise framing; no agent-loop measurements. **[retrieved — intro]** |
| **Context Engineering for AI Agents in Open-Source Software** — ACM 10.1145/3793302.3793350 (2026-07-31) | The round-2 find: **empirical** study of AI context files across 466 OSS projects (5% adoption). AGENTS.md+CLAUDE.md is the most common co-occurring pair; Copilot instructions average 310 lines, CLAUDE.md 287, GEMINI.md 106. Real-world numbers on the always-on rules-file primitive — and an implicit tension with the "keep context minimal" advice: a 287-line CLAUDE.md is a recurring per-session token cost by default. **[retrieved — RQ1/RQ2 sections]** |
| **Agentic AI Context Engineering: Patterns, Offloading Strategies…** — Authorea preprint 10.22541/au.175743558 (2025-09-09) | Survey of offloading/window-management patterns. Cloudflare-blocked; only the abstract was retrievable (via search listing). Non-peer-reviewed preprint — treat as a map, not evidence. **[retrieved — abstract only]** |
| NIST — Agentic AI identity foundations (nist.gov blog) | **Tangential.** About agent identity/authorization (SPIFFE, OAuth, DPoP, RAR, draft NISTIR 8587), not context curation. Relevant at the boundary — scoped agent access to tools/data is what makes untrusted context survivable — but not a context-management mechanism. Not folded into the synthesis. **[retrieved]** |

### Practice

| Source | Verdict |
| --- | --- |
| **Context Engineering for Coding Agents** — Martin Fowler / Thoughtworks (2026-02-05) | Best practitioner taxonomy of the *configuration* layer: instructions vs guidance; "context interfaces" (tools/MCP/skills) with three loaders — LLM-decides, human-decides, software-decides (hooks); "keep the context as small as possible," build rules files gradually, demand transparency about what is occupying the window. No measurements; normative. **[retrieved]** |
| **Why AI coding agents need context graphs** — Postman blog | Typed entity/relation graphs (service catalogs, API registries, ownership maps) built *outside* the prompt and queried by the agent — the enterprise-scale version of "retrieve, don't hold" + structure. Cites Chroma and Lost in the Middle correctly. Argument, not measurement. **[retrieved]** |
| **The semantic layer** — MIT Sloan / CISR research briefing (Lefebvre, Legner, Beath) | Same primitive for enterprise data: machine-interpretable business context (dictionaries, taxonomies, ontologies). Has real survey evidence: 349 executives, 21% rate data curation well developed; mature practices → >3× more likely to report effective data/AI initiatives. Adjacent to agent token management; converges with Postman and the externalization survey. **[retrieved]** |
| **Context budget** — mdazlaanzubair blog | Reviews the ContextBudget paper — **verified 2026-08-29: arXiv 2604.01664**, "Budget-Aware Context Management for Long-Horizon Search Agents": most compression is **budget-free** — policies that don't condition on remaining capacity; budget-conditioned compression (decide what to preserve/compress *given how much room is left*) is a genuine mechanism refinement beyond round 1. **[retrieved — paper now verified directly]** |
| **Context engineering for production AI agents** — ai-crescent guide | Accurate synthesis of the round-1 canon (compaction, tool-result clearing, memory, isolation; Lost in the Middle + context rot), with useful *scoping*: RAG is an input to context engineering, not the discipline; fine-tuning is out. Claims a "2026 follow-up study on long-horizon search" — unverified. **[retrieved]** |
| **Context engineering: delivery discipline** — rickpollick.com | Organizational framing: context components are owned, versioned, regression-prone systems — treat the window as a product surface with owners, evals, observability. "The prompt is a rounding error next to forty steps of accumulated history." No measurements; the ops complement to the technical threads. **[retrieved]** |
| **In-context learning is the only AI feature that matters** — Medium (Matricardi, 2026-08) | Hyperbolic enthusiast opinion piece: ICL as "software execution," weights as "storage." Directionally aligned with context-centricity; no evidence grade; do not cite beyond illustrating the framing. **[retrieved — thesis + opening]** |
| **Context window comparison** — elvex (2026) | Vendor content but the most quantified round-2 piece: 13 models ≥1M advertised; **effective capacity ≈ 60–70% of advertised on every model benchmarked**, with sharp threshold drop-offs (RULER; Chroma recap); full-window fill cost spans 71× ($0.14 DeepSeek V4 Flash → $10.00 Claude Fable 5). Directly quantifies thread 1's "advertised window ≠ usable window." **[retrieved]** |
| **Long-context verification tests** — aimultiple | Benchmark-methodology write-up across 10 models at 850k–1M windows; notable detail: verify-YES vs verify-NO asymmetry (90/94% for claude-fable-5) — confirming a value requires finding it, rejecting one only requires spotting a mismatch. Methodology content, not a strategy source. **[retrieved — excerpt]** |
| **Enterprises with context layers report agent failures at >2× the rate** — VentureBeat (2026-08-17) | VB Pulse survey, n=101 enterprises: 68% traced confident-but-wrong agent answers to missing/inconsistent business context (up from 57% in June); enterprises **with** governed context layers report failures at more than twice the rate of those without. The honest reading is a **detection effect** — governance raises measurement and reporting, not necessarily incidence — but it is a published counterweight to "context layer solves it" vendor claims. **[retrieved]** |
| **Context rot in agents** — techahead | Secondary aggregation: degradation after 20–30 turns; UC Berkeley MAST finding that **79% of multi-agent failures are specification/coordination problems**, not model capability; JIT retrieval keeping context under 8k tokens; a PwC 10%→70% accuracy jump from judge-agent verification. Useful pointers (MAST especially), unverifiable vendor framing. **[retrieved — key takeaways]** |
| **Context engineering** — Slite | Vendor piece selling a "company brain" MCP: four context layers by durability; own benchmark (90%/39s vs 68%/102s for Claude wired to eight MCPs) — unverified, self-interested. The durability-layer idea is sound and matches the memory thread. **[retrieved]** |

### What round 2 adds to the synthesis

- **The externalization literature now has a formal backbone.** Zhou et al. (2604.08224)
  and the Mei et al. survey (2507.13334) independently re-derive round-1 threads 2, 3, and 7
  from the academic side; "externalize + address + query" is no longer just Anthropic/Manus
  practitioner wisdom.
- **Structure at org scale.** Postman's context graphs and MIT CISR's semantic layer are
  the same primitive as Serena's symbol graph and MemGPT's paging, one altitude up: typed,
  queryable structure outside the window. Three independent sources converge on it.
- **The always-on rules file is empirically heavy.** The ACM study's 287-line median
  CLAUDE.md quantifies the fixed token cost every session pays — the round-1 tools (caveman,
  ponytail) monetize exactly this line item, which explains their market better than any
  claimed percentage.
- **Budget-conditioned compression** (preserve/compress decisions conditioned on remaining
  capacity) is a mechanism refinement beyond the round-1 primitives.
- **Advertised windows are ~1.4–1.7× effective capacity** on every benchmarked model, and
  failures are *detected more*, not caused more, where context governance exists
  (VentureBeat). Both sharpen thread 5's skepticism toward headline numbers.
- **Round 2 changes no verdicts**; it strengthens threads 1, 2, 5, 7, 10 and adds the
  budget-conditioning and detection-effect nuances above.

---

## 9. Addendum — the working-memory parallel (2026-08-29)

Is the context window a model of human short-term memory? The parallel is close enough that
agent-memory researchers use human memory as their design template, and specific effects
have been tested on both sides. [retrieved this session: the LLM-side papers; the human-side
canon below is background knowledge, labeled as such]

| Context-window phenomenon | Human-memory effect | Bridging work |
| --- | --- | --- |
| Lost-in-the-middle U-curve (arXiv 2307.03172) | Serial position curve — primacy and recency (Murdock 1962; background canon) | **Serial Position Effects of LLMs** (arXiv 2406.15981) explicitly re-derives lost-in-the-middle as primacy + recency, notes both are "well-documented cognitive biases in human psychology," and argues they are adaptive, not flaws **[retrieved]** |
| Attention budget / degradation with length (Anthropic; Context Rot) | Working-memory capacity limits (Miller's 7±2 1956; Cowan's 4±1 chunks 2001; Baddeley & Hitch 1974 — background canon) | CoALA — Cognitive Architectures for Language Agents (arXiv 2309.02427) imports the cognitive-architecture tradition directly: working, episodic, semantic, procedural memory as the design taxonomy for agents **[retrieved — skim]** |
| Distractor degradation (Context Rot's distractor haystacks; arXiv 2310.01558 — RALMs degrade on irrelevant context) | Retroactive interference / fan effect (Anderson 1974; background canon) | The parallel is structural; no single bridging paper found this session — flagged as an open link |
| File-system / external memory tools (memory tool, MemGPT, Letta) | Long-term memory vs working memory; Atkinson–Shiffrin dual store (1968; background canon) | MemGPT's virtual-memory framing; HippoRAG (arXiv 2405.14831, NeurIPS'24) builds retrieval on **hippocampal indexing theory** — pattern separation + completion over an episodic memory graph **[retrieved]** |
| Sleep-time compute (arXiv 2504.13171, Letta 2025) — offline "thinking" over stored context before queries arrive | Memory consolidation during rest/sleep | Direct design import; the paper's own framing **[retrieved]** |
| Recitation (Manus todo rewrite), structured note-taking | Rehearsal in working memory; external notes as cognitive artifacts | Manus blog (round 1); Clark & Chalmers' extended mind is the philosophical frame (background, not retrieved) |
| Cognitive Overload Attack (arXiv 2410.11272) — long-context prompt-injection risk analyzed via **cognitive load theory** (Sweller) | Cognitive load theory applied to instruction design | Explicit import of the human-cognition framework into LLM security **[retrieved]** |
| Effective capacity ≈ 60–70% of advertised window (elvex) | Humans can *hold* more than they can *use* — storage vs attentional capacity dissociation | Structural parallel; no bridging paper found |

**Where the analogy breaks** (and why "context window = short-term memory" is a metaphor,
not an equivalence):

- A transformer attends over its *entire* history by construction — there is no hard
  capacity limit like Cowan's ~4 chunks. Degradation is a learned positional/attention
  artifact, not a structural bottleneck; it varies by model and by task (Context Rot's
  non-uniform results).
- Human working memory decays within seconds without rehearsal; context is lossless by
  default and degrades only through curation choices or KV eviction. The failure modes are
  opposite: humans *lose* content, LLMs *drown* in it.
- Interference in humans is retrieval competition among similar traces; in long contexts it
  shows up as distraction and clash (Breunig) — superficially similar, mechanistically
  different (attention weights vs cue-overload).

Net: the parallel is strongest at the *systems-design* level (paging, rehearsal,
consolidation, interference) — which is exactly where CoALA, MemGPT, HippoRAG, and
sleep-time compute operate — and weakest at the *mechanistic* level.

---

## 10. Addendum — papers that put memory degradation into the mix (2026-08-29)

A follow-up to §9: work that treats forgetting as a *design element* — decay, consolidation,
and eviction — rather than a failure mode. All retrieved this session unless labeled.

**Explicit decay functions (Ebbinghaus-inspired):**

- **MemoryBank** (arXiv 2305.10250) — "incorporates a dynamic [memory updating mechanism]
  by the Ebbinghaus Forgetting Curve theory, a well-established psychological principle
  that describes how the strength of memory decreases over time." The canonical example of
  degradation imported from psychology into agent memory. **[retrieved]**
- **Ebbinghaus Forgetting Curve and LLM Memory** (ACM 3803291.3803294, 2026-06) — "a dynamic
  memory retention rate model… integrat[ing] memory intensity and time decay."
  **[retrieved — abstract via search listing; full text not accessible]**
- **memory-decay-engine** (github.com/Emmimal/memory-decay-engine) — practitioner library:
  retention scored by the forgetting curve, "every recall reinforces an item's stability and
  pushes its eviction horizon out non-linearly" — spacing-effect-style reinforcement.
  **[retrieved — repo description]**

**Decay as a retrieval score:**

- **Generative Agents** (arXiv 2304.03442) — memory retrieval "combines relevance, recency,
  and importance," with recency scored by time-decay. The oldest mainstream deployment of
  graded forgetting inside an agent loop. **[retrieved]**

**Learned consolidation (degradation by discard/summarize):**

- **MEM1** (arXiv 2506.15841) — RL-trained agents that, each turn, "discar[d] the contents
  from previous steps to achieve constant memory usage" — forgetting as a learned policy,
  not a heuristic. **[retrieved]**
- **Sleep-time compute** (arXiv 2504.13171) — consolidation during inactivity (§9).

**Architecture-level decay:**

- **Titans** (arXiv 2501.00663) — "a decaying mechanism" inside the test-time associative
  memory module, alongside surprise-gated writes — decay in the update rule itself.
  **[retrieved]**
- The KV-eviction line (StreamingLLM, H2O, SnapKV — §2.4) is forced forgetting at the cache
  level. **[retrieved, round 1]**

**Where degradation lives — the structural finding:** decay enters at three layers:
(1) *retrieval scoring* (Generative Agents), (2) *storage policy* (MemoryBank, MEM1,
memory-decay-engine), (3) *the update rule itself* (Titans). Notably absent: a widely cited
mechanism applying *graded decay to the context window's own contents* — the window layer
knows only binary operations (keep, evict, compact). The round-1 tension resolves here:
context rot is degradation-as-bug at the window layer; this literature is degradation-as-
feature at the memory layer — controlled forgetting in the store instead of uncontrolled
interference in the window.

---

## 11. Addendum — the case against externalization (2026-08-29)

No paper argues against externalization by name. The counter-case exists as four empirical
clusters plus one standing philosophical objection — and together they don't refute it;
they make it *conditional*.

**1. Long context beats external retrieval on quality.** Google DeepMind, "Retrieval
Augmented Generation or Long-Context LLMs?" (arXiv 2407.16833): long-context LLMs
"outperform RAG in almost all settings (when resourced sufficiently)." RAG's advantage is
cost, not quality; the paper's Self-Route routes *conditionally*. The direct counter to
"externalize everything."

**2. Extra context often doesn't help — and can hurt.** "Sufficient Context: A New Lens on
Retrieval Augmented Generation Systems" (arXiv 2411.06037, ICLR 2025, Google): a formal
notion of whether the context (retrieved or not) is sufficient for the query; models
hallucinate even with sufficient context, and insufficient retrieved context degrades them
further. External context is not automatically signal.

**3. Retrieved experience distorts behavior.** "How Memory Management Impacts LLM Agents"
(arXiv 2505.16067): agents show an **experience-following property** — high similarity
between the current task and a retrieved memory record produces highly similar outputs —
with a dedicated **error propagation** analysis: wrong records in memory steer future
behavior. Independent confirmation of Manus's "don't get few-shotted," now quantified.
The strongest agent-level argument against naive memory layers: the store is not neutral.

**4. External context overrides internal knowledge, even when wrong.** Longpre et al.,
"Entity-Based Knowledge Conflicts in Question Answering" (arXiv 2109.05052): models
systematically prefer contextual over parametric knowledge in conflicts. Whatever is
externalized becomes authority — stale or poisoned records don't just fail to help, they
override what the model knew. (Adjacent: arXiv 2310.01558 — irrelevant context degrades
RALMs.)

**5. The philosophical objection.** Sutton's bitter-lesson argument — hand-designed
structure loses to scale and learning — cuts against elaborate hand-coded memory
scaffolding. Round-2 evidence points the same way: McMillan's study found model capability
dominated and filesystem strategies didn't help weaker models; MEM1's response is to
*learn* the forgetting policy rather than hand-code it.

**Human-side caution (analogy only, background canon — not retrieved this session):**
Sparrow et al. 2011 (the "Google effect" — offloading degrades internal memory) and the
2025 MIT Media Lab essay on ChatGPT and cognitive offloading — if §9's parallel holds,
agents may have an analog: over-reliance on retrieval weakening in-window reasoning.
No agent-level paper verifying this was found.

**Synthesis:** the literature supports *conditional* externalization — externalize only
when in-window/in-weights supply is insufficient (sufficient-context gating, Self-Route),
treat memory writes as epistemically dangerous (error propagation, conflict override),
keep conflict handling explicit, and prefer learned policies over hand-coded scaffolds.
The round-3 architecture verdict stands, with one amendment: the context layer's write path
needs the governance, and its read path needs a sufficiency test — not everything the store
knows should come back.

---

## 12. Addendum — round-3 source set (2026-08-29)

Sixteen more sources: ten arXiv papers, six practitioner pieces. The theme that emerged
without being asked: the "context as a separate layer" architecture (round-3 discussion)
now has position papers, a theory of acquisition, an automation story, a typed-runtime
implementation, and a governance story.

### Papers

| Source | Verdict |
| --- | --- |
| **Agentic Context Management: Solving Agent Memory and Cost by Treating Them as Lifecycle and Architecture Problems** — Dadhich, arXiv 2607.21503 | Position paper defining Agentic Context Management as "the discipline of deciding what an agent should hold in context, when, for how long, and at what cost, across the full lifecycle." The separate-layer architecture argued in paper form. **[retrieved]** |
| **Active Inference as Context Acquisition for AI Agents** — Dutta, Ramachandran, Sra, arXiv 2608.19202 | Casts question-asking, retrieval, and tool calls as **active inference** (Friston): acquire the context that best reduces expected uncertainty. The theoretical grounding the assembly policy lacked — selection as principled inference, not heuristic. **[retrieved]** |
| **Trace: TRajectory Attribution for Automated Context Engineering** — Zhao, Misra, Pandey, arXiv 2608.09153 | Attributes production agent failures to specific context sources (system prompts, knowledge bases, tool descriptions, skills) and automates fixes via "textual gradients." Context engineering made self-improving — the layer's policy with its own feedback loop. **[retrieved]** |
| **Inadvertent Context Leakage in Language Models** — Fairoze et al. (Google), arXiv 2608.19857 | Agents processing sensitive data leak context irrelevant to the prompt; formal adaptive-adversary framework. Context is a **leak surface** — extends §11's governance requirement from truth (write path) to scope (read path). **[retrieved]** |
| **Token Optimization and Context Window Management in Multi-Agent AI Workflows** — arXiv 2608.17188 | Multi-agent workflows "limited not only by model quality but by token cost, latency, and context-window quality" — window economics as first-class system constraints. **[retrieved — abstract]** |
| **Robustness Analysis of Agentic AI to Inconsistent and Incomplete Tool Responses** — Xu et al., arXiv 2608.22676 | Robustness to degraded tool outputs — context *quality at the source*, upstream of any layer. Title-level retrieval only (parser limits); noted, not synthesized. **[retrieved — title]** |
| **The Evaluation Context Protocol (ECP)** — arXiv 2608.19263 | A portable contract for agent evaluation. Context-aware evaluation infrastructure; title-level retrieval only. **[retrieved — title]** |
| **AI Evaluation Should Measure Verification Cost, Not Correctness Alone** — arXiv 2608.08709 | Current evaluation is "verification-blind" — correctness measured without the cost of establishing it. Connects to rickpollick's delivery discipline and aimultiple's verify-YES/NO asymmetry. **[retrieved]** |
| **LearnAI: Just-in-Time AI Co-Creation at a University** — arXiv 2608.19164 | Education deployment; **tangential** — not folded into the synthesis. **[retrieved — abstract]** |

### Practice

| Source | Verdict |
| --- | --- |
| **Building Intelligent Memory for AI Agents** — flur.ee | The best practitioner piece in the set: context window as working memory — "amnesia with good note-taking" across tasks; failures arrive in order — **continuity, then cost (quadratic transcript replay), then governance**; four memory kinds — episodic, semantic, procedural, and **policy**. The policy-as-memory move is the novel contribution: an agent that reads constraints from the same governed store as its facts is "governed by construction." Vendor (memory infrastructure), argument not measurement — but consistent with CoALA's taxonomy and adds the governance leg. **[retrieved]** |
| **Running a Software Factory Efficiently at Uber Scale** — uber.com blog | Production telemetry, the hardest practitioner numbers in this document: **70% of PRs attributed to agents, 3,600 skills, 30K skill executions/day**, weekly agentic requests up 9.4× while **total AI spend stabilized since April** through optimization; cost decomposed per layer (portfolio / unit economics / model economics / driver decomposition). Context and cost engineering measured at fleet scale. **[retrieved]** |
| **How to reduce context cost with smart context construction** — mem0 blog | Vendor cost arithmetic, checkable: input tokens dominate (prompts run 10–50× output size); naive full-history replay is O(n) per call, retrieval-bounded construction is O(k); worked example at ~$0.045/call by turn 50. Consistent with JetBrains' dilution finding — the lever is what gets sent. **[retrieved]** |
| **Why AI agents fail: enterprise context** — Alation | Enterprise framing via a Mark Nelson (ex-Tableau) interview: models have coverage, not *your* world model; **institutional context is capturable, versionable, queryable — taste is not**; non-deterministic systems "fail silently" — well-formed, confident, wrong — so oversight must reconstruct grounding after the fact; cites Gartner's 2030 projection that half of agent deployment failures will trace to insufficient runtime enforcement. Vendor (data catalog), podcast-derived, but the institutional-context-vs-taste distinction is a keeper. **[retrieved]** |
| **Beyond prompt engineering: better enterprise AI needs better context** — hackernoon | Listicle: context pipelines, five properties (relevant, fresh, complete, trustworthy, **authorized**). The authorization property aligns with the leakage paper and NIST. Low depth. **[retrieved]** |
| **AI agents don't need more context — they need typed context** — Towards Data Science (E. Alexander, 2026-08-24) | A working runtime layer separating instructions, evidence, memory, and tool output as **typed context objects** — validation, provenance ledger, Design-by-Contract (Meyer) applied to context, tested before serialization; the model still sees plain tokens, everything upstream is the contribution. A practitioner implementation of the separate-layer architecture. Same author as the memory-decay-engine library (§10). Benchmark is self-run. **[retrieved]** |
| **6 types of contexts for AI agents** — Daily Dose of DS | Newsletter taxonomy: context as a "multi-dimensional design layer"; thesis that a weaker model with right context beats a SOTA model with incomplete context. Useful framing, no evidence. **[retrieved — framing]** |
| **I built a context engineering prompt from scratch** — dev.to | Personal anecdote: one prompt, five transformations, "10× more useful" self-assessed. Honest about being an experiment; no benchmark. **[retrieved]** |

### What round 3 changes

- **The separate-layer architecture has converged from idea to subfield.** Position paper
  (2607.21503) + acquisition theory (active inference) + automation (Trace) + typed runtime
  (TDS) + governance (flur.ee policy-as-memory). The open question is no longer *whether*
  the window should be a view over a governed layer — it is what the layer's *contract* is:
  types, validation, sufficiency, decay, and authorization.
- **Context governance is now three-sided, not one:** truth (error propagation, conflict
  override — §11), scope (leakage, authorization), and audit (lineage, verification cost,
  ECP). NIST's round-2 identity piece slots into scope.
- **Production economics landed.** Uber's stabilized-spend telemetry, mem0's O(n)→O(k)
  arithmetic, and flur.ee's quadratic-replay point all say the same thing JetBrains
  measured: the input side is where the money and the lever are.
- **Evaluation is becoming context-aware** — attribution (Trace), verification cost
  (2608.08709), portable eval contracts (ECP). The field noticed that context changes are
  regressions and started building the test harness.

---

## 13. Recommendations — what a good context management layer looks like (2026-08-29)

Synthesized from everything above. Each element carries its evidence grade:
**[S]** strong, **[M]** moderate/self-reported, **[J]** judgment from the evidence.

### 13.1 Load-bearing findings (design inputs)

1. **The window is a view, not the memory.** Externalization is the field's converged
   direction: MemGPT → Letta → Manus's file system → platform memory tools → the
   round-3 subfield (position paper, active-inference acquisition, Trace automation,
   typed runtime). **[S for direction; M for specific implementations]**
2. **The window is a degrading resource.** Context rot across 18 models; lost-in-the-middle;
   effective capacity ~60–70% of advertised. Everything in the window must earn its place,
   every turn. **[S]**
3. **Cache economics constrain everything.** Cached vs uncached input is ~10×; per-turn
   free rebuild is the most expensive mistake available. Append-only + stable prefix +
   breakpoints is the only viable assembly discipline. **[S — arithmetic]**
4. **Writes are the dangerous side.** Error propagates through memory records
   (2505.16067); externalized records override internal knowledge even when wrong
   (2109.05052); leakage via read scope (2608.19857). **[S]**
5. **Extra context is not free signal.** Insufficient context hurts; even sufficient
   context doesn't guarantee correctness (2411.06037). Sufficiency-gate every injection. **[M→S]**
6. **Savings claims inflate 5–10×.** The only independent A/Bs found single-digit effects
   (JetBrains). Measure dilution honestly; the reliable wins are input-side and
   code/output-side, not prompt-style tricks. **[S for the measured set]**

### 13.2 The layer, component by component

**A. The store — system of record.**

- Typed context objects, typed by *role*: instructions, evidence, memory (episodic /
  semantic / procedural / **policy**), tool output. Type determines which operations are
  valid and where each may appear in the window. [TDS typed context; flur.ee; LangChain. **M** —
  format is a measured performance variable: identical contexts re-serialized into plain
  text, Markdown, JSON, and YAML "significantly affect model performance" across reasoning,
  code, and translation (arXiv 2411.10541); format familiarity can outweigh token density
  (McMillan "grep tax", §2.6)]
- Every object carries: source, originator (human / agent / tool), timestamp, version,
  freshness signal, and supersession links. Provenance is not metadata garnish — it is
  what makes "why did you decide that in March" answerable. [Alation, flur.ee. **M**]
- Policy is memory in the governance sense but lives on a **separate read-only plane**:
  constraints read at runtime from a canonical, human/CI-owned source the agent cannot
  write; **binding** enforcement sits outside the model at the tool-call boundary
  (Progent, arXiv 2504.11703 — symbolic rules over tool names/arguments), while
  in-context policy text is explanatory only. Rationale: models obey context over their
  own knowledge (2109.05052), so any agent-writable policy is an escalation path
  [flur.ee's governance goal retained via enforcement + read-only plane, not
  co-location; resolved §16. **M**]
- Byte-deterministic serialization everywhere — it is simultaneously a correctness
  property and the cache-preservation property. [Manus. **S**]

**B. Write path — governed, because writes are the attack surface.**

- Explicit write authority (who/what may persist what), and a scope envelope per agent so
  leakage through read paths is bounded. [NIST-adjacent; 2608.19857. **M**]
- Experience-derived records are quarantined until corroborated or reinforced:
  auto-persisted lessons are how errors propagate (experience-following). Reinforce on
  successful recall, not on write. [2505.16067; memory-decay-engine. **S**]
- Conflicts supersede with lineage — never silent overwrite; stale records are marked,
  because a stale record still *overrides* the model's own knowledge. [2109.05052. **S**]

**C. Read path — sufficiency-gated, placement-aware.**

- Just-in-time retrieval with handles; pre-load only policy and stable scaffolding.
  [Anthropic hybrid; Claude Code glob/grep. **S**]
- Sufficiency gate: inject because the query needs it *and* the object is fresh,
  authorized, and scoped to the task — not because it exists. [2411.06037. **M**]
- Placement is part of the content's meaning: critical constraints at head and tail
  (U-curve), plan recited at the tail, load-bearing facts never buried mid-window.
  [2307.03172; Manus recitation; LIFBENCH (ACL 2025) — instruction-following stability
  measured across 20 LLMs and six length intervals degrades in long contexts. **S**]
- Diversity guard: don't retrieve N near-identical precedents; mimicry drift is measured.
  [Manus; 2505.16067. **S**]

**D. Assembly policy — the budget conditioner (the differentiator).**

- Every assembly decision conditioned on remaining capacity, in a fixed priority order:
  policy/constraints → unresolved errors and their traces → current task state →
  selected evidence → condensed history. [ContextBudget via round-2; Manus keep-errors;
  Anthropic recall-first compaction. **M/J**]
- The window is rebuilt *only* as: stable prefix + append-only tail; destructive edits
  (compaction, tool-result clearing) run at explicit breakpoints. [Manus; Anthropic
  context editing. **S**]
- Per-source token accounting with visibility — the developer should always be able to
  answer "what is occupying the window and why." [Fowler; Uber decomposition. **M**]

**E. Degradation engine — the graded middle ground nobody ships.**

- Strength score per memory object: recency base + reinforcement on recall/use +
  outcome feedback on tasks where it was used. Decay controls *retrieval priority and
  eviction* — window operations stay binary; decay lives in the store where the whole
  decay literature already works. [MemoryBank; Generative Agents; §10 finding. **M**]
- Consolidation jobs move validated episodes into semantic summaries on a schedule
  (sleep-time pattern), with originals retained. [2504.13171. **M**]

**F. Attribution and feedback — the layer must be observable and testable.**

- When a task fails, attribute to context sources (which prompt/knowledge/tool object was
  on stage) and emit fixes as candidate policy changes. [Trace. **M**]
- Every policy change ships as a paired A/B with contamination audits — context changes
  are regressions and deserve a test harness. [JetBrains methodology. **S for method**]
- Track verification cost, not just correctness. [2608.08709. **M**]

### 13.3 Invariants (non-negotiable)

1. **No irreversible drop without an address.** Everything evicted, compacted, or
   truncated remains retrievable by handle — restorability is what makes lossy views safe.
   [Manus; Headroom CCR. **S**]
2. **No per-turn free rebuild.** The serializer never mutates the prefix mid-run.
   **[S — arithmetic]**
3. **The store is the source of truth for durable objects; working state is window-local
   by design.** Decisions, corrections, facts, commitments, and unresolved errors must be
   promoted to the store to exist as knowledge — a **promotion gate at breakpoints**
   (before compaction or any destructive window op) formalizes Anthropic's compaction
   discipline. Scratchpads and hypotheses live in the window and die there deliberately:
   no write tax on reasoning, and data minimization is a leakage defense. Cross-session
   continuity is measurable — LongMemEval (arXiv 2410.10813, ICLR 2025) benchmarks it and
   finds existing assistants far from saturated. [flur.ee; LongMemEval; resolved §16.
   **M**]
4. **No savings claim without a baseline definition and paired measurement.** [JetBrains;
   VentureBeat detection effect. **S**]
5. **Isolation is a tool, not the default.** Sub-agents get clean windows and return
   condensed results with their decision context; the main thread keeps full trace
   continuity. Both sides now have evidence: multi-agent gains on benchmarks are "often
   minimal" with specification and coordination failures dominating the taxonomy (MAST,
   arXiv 2503.13657, NeurIPS 2025), yet parallel sampling-and-voting measurably scales
   with task difficulty when coordination is trivial (More Agents Is All You Need,
   arXiv 2402.05120). The synthesis: isolation pays where the sub-task is independent
   and the return contract is condensed; it costs where implicit decisions must stay
   shared. [**M** — still no head-to-head study; direction changed from J]

### 13.4 What to build vs adopt

Adopt the store (Letta/mem0/platform memory are commoditized, each with real adoption).
Build the policy bundle — assembly, sufficiency, decay, attribution — because that is
where every source says the leverage and the open questions live. Build the eval harness
before the policies; the first policy will be wrong and the harness is how you find out
cheaply.

### 13.5 The metrics that matter, in order

1. **KV/prefix cache hit rate** — the single most cost-sensitive number. [Manus. **S**]
2. **Task quality delta, paired A/B** — the only savings measurement anyone should quote.
   [JetBrains. **S**]
3. **Cost per completed task, decomposed by source** — input dominance means the assembly
   policy *is* the cost policy. [mem0 arithmetic; Uber. **M**]
4. **Context recall/precision at act time** — did the agent have what it needed when it
   acted; failure attribution rate. [Trace. **M**]
5. **Continuity** — repeat-task consistency across sessions. [flur.ee. **M**]
6. **Verification cost** — what it costs to establish correctness of outputs. [**M**]

### 13.6 Open problems worth claiming

- **Graded decay at the window boundary** — decay-as-feature in the store is published;
  decay-as-policy mediating between store and window is not. The strength score already
  computed for eviction could drive *re-injection priority* and *placement*.
- **Learned assembly policies** — MEM1 learned forgetting; nobody has learned the full
  assembly policy against a verification-cost objective.
- **Active-inference acquisition** is theory; a working implementation that measurably
  reduces redundant tool calls would be a first.

---

## 14. Addendum — evidence backfill for the judgment pieces (2026-08-29)

Five claims in §13 were marked **[J]**. Targeted evidence hunt per claim:

1. **Typed/structured context [J→M].** "Does Prompt Formatting Have Any Impact on LLM
   Performance?" (arXiv 2411.10541): identical contexts re-serialized into plain text,
   Markdown, JSON, and YAML across reasoning, code generation, and translation —
   "different formats can significantly affect model performance." Combined with
   McMillan's format-familiarity result (§2.6) and LIFBENCH, serialization is a measured
   performance variable, not a stylistic choice. The typed-by-role taxonomy itself
   remains a design synthesis.
2. **Policy-as-memory [J→M for the mechanism].** Progent (arXiv 2504.11703): privilege
   control for LLM agents as symbolic policy rules over tool names and arguments,
   enforced during execution — runtime policy-as-context exists and is peer-reviewed
   territory. What remains [J] is only flur.ee's specific unification: reading those
   policies from the same store as facts.
3. **Assembly priority order [still J for the exact ordering].** Placement sensitivity is
   solid (lost-in-the-middle; Manus recitation; LIFBENCH — ACL 2025, 20 LLMs, six length
   intervals, instruction-following stability degrades with length). No paper validates
   the specific priority chain (policy → errors → state → evidence → history); each
   ingredient is individually evidence-backed, the sequence is engineering judgment.
4. **Store-as-source-of-truth / continuity [J→M].** LongMemEval (arXiv 2410.10813,
   ICLR 2025): a 500-question benchmark of long-term interactive memory across sessions
   (information extraction, multi-session reasoning, temporal reasoning, knowledge
   updates, abstention) — continuity is benchmarked and unsaturated, so the invariant is
   now testable rather than asserted.
5. **Isolation as tool, not default [J→M].** MAST, "Why Do Multi-Agent LLM Systems
   Fail?" (arXiv 2503.13657, NeurIPS 2025): multi-agent gains on popular benchmarks are
   often minimal; the failure taxonomy is dominated by specification and coordination
   issues. Counterweight: "More Agents Is All You Need" (arXiv 2402.05120): trivially
   coordinated sampling-and-voting scales with task difficulty. Both sides measured →
   the recommendation is evidence-graded, with the boundary condition stated: isolation
   pays for independent sub-tasks with condensed return contracts.

Remaining **[J]** marks in §13: the exact assembly priority chain, the
policy-in-store unification, the store-as-truth invariant as stated (testable via
LongMemEval-style evals, but not itself a tested design). Those three are now
*specified* judgments — each has a named benchmark or study that could confirm or kill
it.

---

## 15. Addendum — pros/cons on the remaining [J]s (2026-08-29)

**J1 — the fixed assembly priority chain (policy → errors → state → evidence → history).**
For: matches what recall-first compaction actually preserves (Anthropic); errors/state are
the anti-drift payload (Manus recitation; error-propagation cuts the other way only for
*persisted* memory); policy loss is the least-detectable failure. Against: priority and
placement are different axes — the tail dominates attention, so "policy first" must mean
head-stable *plus* recited copy, which a linear chain can't express; mid-task, active
evidence can outrank global policy, so a rigid chain is a fallback order, not an assembly
order; tail volatility (errors, state) fights append-only caching. Settler: forced-budget
ablations over order permutations (ContextBudget-style) measuring task quality and
violation rates per order.

**J2 — policy-in-store unification.** For: one governance surface — versioning, lineage,
freshness, and diffability apply to policy changes like any other change; Progent shows
runtime enforcement is tractable; cross-agent consistency for free. Against: the
knowledge-conflict result means a stale or poisoned policy object is obeyed *confidently*
— unification raises the blast radius of a policy bug to every session; policies beside
retrievable content expand the injection surface, so the store needs internal trust tiers
(anyway), which is the complexity the unification claimed to remove; store outage must
fail closed on constraints; some compliance regimes require human-only policy custody.
Settler: eval of prompt-inlined vs store-read policy under update frequency and injection
pressure (Progent-style harness plus leakage frameworks).

**J3 — store-as-only-source-of-truth.** For: kills the context-clash bug class at the
root (Breunig) instead of patching instances; makes compaction a view change rather than
a data event, which is what makes aggressive compaction safe; auditability and
replayability come free. Against: in-window reasoning state (scratchpads, hypotheses) is
real work — forcing it through store writes costs output tokens (3–5× input price), adds
tool-call latency, and depends on the model choosing to save (Fowler's non-determinism,
inverted); KV-cached views can't reflect mid-run store updates without breaking the
prefix, so the invariant only truly holds at breakpoints; not everything *should*
persist — data minimization is a leakage defense (2608.19857), and store-everything ends
in retrieval drowning (the noise complaint in §10's YourMemory). Settler: A/B strict
store-first vs window-first-with-checkpoints on LongMemEval-style continuity, cost per
task, and auditability.

---

## 16. Addendum — resolution of J2 and J3 (2026-08-29)

**J2 resolved by splitting the plane, not unifying it.** The risk in policy-in-store was
co-location: with models obeying context over internal knowledge (2109.05052), an
agent-writable policy object is a self-enforcing escalation path. Design: (a) **binding**
policy enforced structurally at the tool boundary (Progent-style symbolic rules) —
injection-resistant by construction; (b) **explanatory** policy served read-only from a
human/CI-owned plane the agent can never write; (c) the agent-writable memory store never
contains policy. Residual: prose policy is advisory — the enforcement layer is
load-bearing; the policy plane deploys like configuration, never runtime-edited via agent
paths.

**J3 resolved as two-tier truth with a promotion gate.** The strict invariant contradicted
the working/long-term memory distinction the field is built on (§9): scratchpads are
ephemeral by design, and persisting everything costs 3–5×-priced output tokens, adds
tool-call latency, and violates data minimization (2608.19857). Design: durable objects
(decisions, corrections, facts, commitments, unresolved errors) must be promoted to the
store to exist as knowledge; working state dies at breakpoints; a **promotion gate before
every destructive window operation** promotes anything matching the durable categories
(Anthropic's compaction discipline, formalized). Residual: the gate is a classifier —
err toward promoting the durable categories, keep a visible promotion log, and treat
repeatedly-recited content as durable by definition.

Remaining **[J]** after these resolutions: only the exact assembly priority chain (J1,
§15). **Resolved 2026-08-29 without experiment** — see §16.1.

### 16.1 J1 resolved by restricting to validated behavior (no experiment)

Adoption constraint set 2026-08-29: no experiments; use only what the research already
validates. The five-level priority chain was the speculative part and is dropped. What
survives is the ordering the evidence already supports, which locus's existing
implementations largely embody:

- **Evict first: stale/superseded tool results and over-threshold outputs.** Validated by
  Anthropic's context editing — cleared first, "safest lightest-touch form of compaction"
  (self-reported +29%/+39% evals, §3) — and already locus's tool-compaction + artifacts
  design.
- **Then: decayed memories by measured strength** (similarity × strength ranking, prune
  below threshold) — the decay literature's validated mechanism (§2, §10), already in
  locus.
- **Never evict from the head:** constitution/policy — stable prefix, cache arithmetic
  (§5, thread 4), split-plane (§16 J2).
- **Never evict within-run:** unresolved errors and current task state (Manus; Anthropic
  recall-first compaction preserves decisions/bugs/state). They die at session end, as
  locus's working layer already does.
- **Incoming content:** sufficiency-gated (2411.06037), strength-ranked, placed head/tail
  by the U-curve evidence (2307.03172).

The `sticky`/`standard`/`disposable` eviction-class attribute (fit-doc R1) is retained
only as a label for this validated pattern, not as a novel mechanism requiring
validation. The forced-budget ablation is retired. Under the same constraint, the §13.6
"open problems worth claiming" (learned assembly, active-inference acquisition,
decay-at-boundary measurement) are research-only and out of adoption scope.

---

## 8. Source list

Tools: github.com/headroomlabs-ai/headroom · headroomlabs.ai · github.com/rtk-ai/rtk ·
rtk-ai.app · github.com/JuliusBrussee/caveman · andrew.ooo caveman review ·
github.com/DietrichGebert/ponytail · ponytail.dev · github.com/Compresr-ai/Context-Gateway ·
github.com/oraios/serena (via ctx7) · github.com/upstash/context7 · github.com/mem0ai/mem0
(via ctx7) · github.com/letta-ai/letta (via ctx7) · github.com/microsoft/LLMLingua ·
github.com/thedotmack/claude-mem (via ctx7) · github.com/topics/context-compression

Academic: arXiv 2310.05736 (LLMLingua) · 2310.04408 (RECOMP) · 2304.12102 (Selective
Context) · 2310.08560 (MemGPT) · 2307.03172 (Lost in the Middle) · 2309.17453 (StreamingLLM)
· 2306.14048 (H2O) · 2404.14469 (SnapKV) · 2307.06945 (ICAE) · 2410.05983 (Long-Context
LLMs Meet RAG) · research.trychroma.com/context-rot · McMillan 2026 structured-context study
(via simonwillison.net) · arXiv 2507.13334 (Context Engineering survey) · arXiv 2604.08224
(Externalization review) · arXiv 2601.11653 (Memory control) · arXiv 2603.05344 (Terminal
coding agents) · arXiv 2603.09619 (Context Engineering overview) · arXiv 2506.05370
(Contextual Memory Intelligence) · dl.acm.org/10.1145/3793302.3793350 (context files in
466 OSS projects) · authorea.com/10.22541/au.175743558 (offloading patterns preprint)
· nist.gov agentic-identity blog · memory-parallel papers: arXiv 2406.15981 (serial position
effects), 2309.02427 (CoALA), 2405.14831 (HippoRAG), 2504.13171 (sleep-time compute),
2410.11272 (cognitive overload), 2310.01558 (robustness to irrelevant context) ·
degradation papers: arXiv 2305.10250 (MemoryBank / Ebbinghaus), 2304.03442 (Generative
Agents recency decay), 2506.15841 (MEM1), 2501.00663 (Titans decaying memory),
dl.acm.org/10.1145/3803291.3803294 (retention-rate model), github.com/Emmimal/memory-decay-engine ·
counter-evidence: arXiv 2407.16833 (long-context vs RAG / Self-Route), 2411.06037
(sufficient context, ICLR'25), 2505.16067 (experience-following, error propagation),
2109.05052 (knowledge conflicts) · round-3 papers: arXiv 2607.21503 (agentic context
management), 2608.19202 (active inference acquisition), 2608.09153 (Trace attribution),
2608.19857 (context leakage), 2608.17188 (multi-agent token optimization), 2608.08709
(verification-cost evaluation), 2608.22676 (tool-response robustness), 2608.19263 (eval
context protocol) · round-3 practice: flur.ee intelligent memory · uber.com software
factory · mem0.ai context cost · alation.com enterprise context · towardsdatascience.com
typed context · hackernoon, dailydoseofds, dev.to pieces · judgment-backfill papers: arXiv 2411.10541
(prompt format sensitivity), 2504.11703 (Progent privilege control), 2410.10813
(LongMemEval), 2503.13657 (MAST multi-agent failures, NeurIPS'25), 2402.05120 (More
Agents), LIFBENCH (ACL 2025)

Practice: anthropic.com/engineering/effective-context-engineering-for-ai-agents (2025-09-29)
· claude.com/blog/context-management (2025-09-29) · manus.im blog (2025-07-18) ·
cognition.ai/blog/dont-build-multi-agents (2025) · blog.langchain.dev/context-engineering
(2024-12) · dbreunig.com how-contexts-fail (2025-06-22) · simonwillison.net/tags/
context-engineering · blog.jetbrains.com/ai/ ponytail-skill-claude-tested (2026-07-28) · martinfowler.com
context-engineering-coding-agents (2026-02-05) · blog.postman.com context-graphs ·
mitsloan.mit.edu semantic-layer (CISR) · blog.mdazlaanzubair.com ai-agents-context-budget ·
ai-crescent.com production guide · rickpollick.com delivery discipline · elvex.com context
comparison · aimultiple.com long-context verification · venturebeat.com context-layer survey
(2026-08-17) · techaheadcorp.com context-rot · slite.com context-engineering ·
medium.com/@fabio.matricardi ICL piece (2026-08)
