# Documentation site

*Read when: documentation work (`md_docs/`, `mkdocs.yml`) or publishing checks.
Do docs work in a docs/verification phase — not while implementing features.*

## Layout

- Sources: `md_docs/` + `mkdocs.yml` at the repo root.
- Generated output `md_docs/site/` is **ignored — never edit**, it is a build product.
- Domain docs live in `md_docs/domain_docs/`: `NN-<env>.md` per environment
  (`04-canvas-2d.md`, …), plus `domain_docs/guides/` for how-to guides
  (e.g. `01-creating-environment`, `09-domain-engine-integration`).
- App-engine oriented pages: `md_docs/app_engine_docs/`, top-level `index.md`,
  `02-architecture.md`, `13-domain-engine-integration.md`, `15-operations-and-undo.md`,
  `16-serialization.md`, `17-testing.md`, `18-public-api-reference.md`,
  `19-error-reference.md`, `20-quick-start.md` (verify the current page list when working).

## Rules

1. Adding or renaming a page requires the matching `mkdocs.yml` nav update — a page
   without a nav entry is invisible in the site.
2. Per-environment docs are a late-phase deliverable (the docking plan defers them to
   its verification phase); write code docs (module doc headers, PRD references)
   during implementation instead.
3. Public-API claims in docs must match code (verify signatures before publishing).

## Published artifacts & links

- Docs site: <https://unique-digital-resources.github.io/AppShell-Documentation/>
- Repo: <https://github.com/Unique-Digital-Resources/AppShell>
- Key guides: [Creating a New Environment](https://unique-digital-resources.github.io/AppShell-Documentation/domain_docs/guides/01-creating-environment/)
  · [DomainEngine integration](https://unique-digital-resources.github.io/AppShell-Documentation/domain_docs/guides/09-domain-engine-integration/)

(The original link list was `agents/reference/important_links.md`; its contents are
folded here. When the site is regenerated, re-check guide links.)
