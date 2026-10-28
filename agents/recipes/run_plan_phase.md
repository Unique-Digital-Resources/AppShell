# Playbook — run one planned phase

*Follow when executing the current phase of the feature plan. Reads:
`current_state/CHECKPOINT.md` first, then `current_state/plan.md` for the phase.*
*The plan itself (what the feature is, phase list) lives in
`current_state/plan.md`; requirements in `current_state/PRD.md`.*

## Protocol

1. **Read the checkpoint first** — always, in any session touching the work. It says:
   what is established (code reality), settled do-not-relitigate decisions, current
   phase, and the minimal reading list for the next phase.
2. **Take the phase's tasks from `current_state/plan.md`** — objective, tasks,
   related files, completion criteria. Read **only** the files in the checkpoint's
   reading list (plus knowledge files from `agents/` per the routing triggers).
   Do not skim the project.
3. **Read the previous phase record**
   (`current_state/records/phaseN_<name>.md`) before starting — it settles
   conventions and lists next-step details.
4. **Implement the phase.** Verify continuously:
   ```bash
   cargo test --test domain_foundation_tests -- <area>   # new area's tests
   cargo test                                            # full suite, 0 regressions
   cargo check --features serde                          # serde path compiles
   ```
5. **Write the phase record** — `current_state/records/phase<N>_<name>.md`:
   - header: phase number/name, status, verification summary (suites + counts);
   - what was built (file-by-file table);
   - decisions made during implementation (numbered — these become the
     do-not-relitigate entries);
   - completion-criteria checklist from the plan;
   - files touched;
   - "Next: Phase N+1" input section (ready-to-use pointers for the next agent).
6. **Update `current_state/CHECKPOINT.md`**:
   - status snapshot table (+ build/verify state with fresh test counts);
   - "what is established" file table;
   - settled decisions section (append the new phase's do-not-relitigate items);
   - "what is next" — next phase's tasks + preview;
   - reading list for the next phase (§4.2) — keep it truly minimal;
   - bump "Last updated" + "Current phase" at the top.
7. **Stop when the phase's completion criteria are met.** The next phase starts
   in a fresh session from the checkpoint — no giant multi-phase runs.

## Rules of engagement

- Transient mid-operation states are model-valid only inside an operation;
  committed state must satisfy the environment's validation.
- A phase record is where decisions are *settled*: contradicting them requires
  writing down why in a new record, not silently relitigating.
- Keep `plan.md`'s phase list untouched while phases execute; if reality diverges
  (a phase splits/merges), record the change in the phase record and update the
  plan's affected section in the same commit.
- Do not move stateless knowledge into records or vice versa (see
  `agents/README.md` maintenance rules).
