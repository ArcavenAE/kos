# Remaining SDD systems to survey for seed scanner

*Idea — pre-hypothesis brainstorming. Generative, uncommitted.*

## Surveyed (session-014)

- **speckit** — github/spec-kit (77k stars, linear pipeline)
- **bmad** — bmad-orc/BMAD-METHOD + modules + ThreeDoors consumer
- **chainlink** — dollspace-gay/chainlink (task tracker)
- **beads** — gastownhall/beads (issue tracker)

## Still to survey

### Local repos available
- **pennyfarthing** — penny-orc/ (aclaude predecessor, TypeScript)
- **spectacle** — aae-orc/spectacle/ (our own IEEE templates)
- **ThreeDoors** — aae-orc/ThreeDoors/ (BOARD, experiments, L0-L4)
- **kos** — itself (dogfood: what does a kos-using project look like?)
- **-orc orchestrators** as a pattern class (aae-orc, bmad-orc, alpha-orc,
  clip-orc, conductor-orc, penny-orc — justfile, repos.yaml, CLAUDE.md,
  SOUL.md, sprint/, charter.md conventions)

### External repos to fetch
- **multiclaude** — dlorenc/multiclaude + ArcavenAE/multiclaude-enhancements
  (ROADMAP convention, actively changing)
- **OpenClaudia** — dollspace-gay/OpenClaudia (orchestration)
- **gastown** — gastownhall/gastown

### Need pointers / access
- **VSDD** — no repo identified yet

## Universal conventions (system-independent)

These appear across many projects regardless of SDD system. The scanner
should detect these built-in, no bridge profile needed:

- ADR variants: MADR, nygard, etc. Directory names: decisions/, docs/adr/,
  docs/decisions/, adr/
- README / CONTRIBUTING / CODE_OF_CONDUCT / LICENSE / SECURITY
- CHANGELOG / HISTORY / release notes
- docs/ tree patterns
- .claude/ / CLAUDE.md / .cursor/ / .github/ / .gitlab/
- Makefile / justfile / taskfile / package.json scripts
- CI/CD: .github/workflows/, .gitlab-ci.yml, Jenkinsfile
- Test conventions: tests/, __tests__/, *_test.go, *_test.rs, test/
- Config: .toml, .yaml, .json at root
- _kos/ (kos itself)
- SOUL.md, charter.md (aae-orc conventions, may become universal)
