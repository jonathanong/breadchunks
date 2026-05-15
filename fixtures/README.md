# Test Fixtures

Markdown documents used by the `breadchunks` test suite. Each fixture is chosen to exercise a distinct set of chunker behaviours.

| File | Purpose |
|---|---|
| `tech-guide.md` | H1–H4 hierarchy, preface paragraph, fenced code blocks containing `#` comment lines (decoys), inline backtick code |
| `recipe.md` | No preface, H2-heavy structure, short paragraphs that trigger phase-2 merging |
| `deeply-nested.md` | Full H1–H6 chain, exercises phase-3 bottom-up parent absorption across all six header levels |
| `code-heavy.md` | Many fenced code blocks with `# Not a header` inside — verifies code-block protection in phase 1 |
| `gettysburg.md` | Public-domain prose structured as a real document; exercises long paragraphs and preface + nested headers. Source: Abraham Lincoln, *Gettysburg Address* (1863, Bliss copy). Public domain — U.S. Government work, pre-1928. |
