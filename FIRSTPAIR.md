# FirstPair book delivery

Marciana's manuscript and book assets are owned by this repository. FirstPair
provides the shared build, artifact verification, reader routes, and catalog
delivery. The canonical shared workflow is
`~/src/firstpair/publishing/PUBLISH.md` and
`~/src/firstpair/publishing/UNIFIED_BOOK_BUILD_GOAL.md`.

## Source contract

- Book root: `docs/book/`
- Manuscript: `docs/book/manuscript.md`
- Metadata: `docs/book/metadata.yaml`
- Cover source: `docs/book/cover.md`
- Cover asset: `docs/book/cover/marciana-cover.png`
- Headboard asset: `docs/book/cover/querygraph-blog-headboard.png` (copied
  from the tracked QueryGraph blog headboard at the exact source revision)
- Stable stem: `marciana`
- Build configuration: `book.build.json`
- Publish-complete outputs: `docs/book/dist/`
- Preview outputs, when introduced: `docs/book/dist-preview/`
- Full outputs, when introduced: `docs/book/dist-full/`

The existing FirstPair headboard/catalog identity remains the publisher-side
reference. The source repository owns the Marciana cover and manuscript; no
public catalog or Blob metadata is edited from this repository during a local
build.

## Safe workflow

```sh
pgrep -x Obsidian || true
git status --short --branch
~/src/firstpair/publishing/scripts/build-library-book.sh \
  --repo-root "$PWD" --edition full
```

Before a non-dry-run publication, this repository and FirstPair must both be
clean, pushed, and pass the canonical Git preflight. Full-edition publication
requires explicit user confirmation; this repository's book goal builds and
verifies artifacts but does not replace a public preview or publish outward.
