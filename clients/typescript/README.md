# Marciana TypeScript client

This is the TypeScript counterpart to `clients/python`: strict request
validation and a transport/authentication seam for the four memory verbs.
Authorization, signing, storage, and recovery remain server responsibilities.
Wire fields intentionally use the shared snake_case contract (`space_id`,
`memory_id`, `memory_ids`) pinned by `crates/marciana-memory/src/api.rs` and
mirrored by the Python client; the server denies unknown fields, so the
request types carry no client-only extras. Validation recurses into improve
replacements and each forget memory id, and the shared wire fixture in
`compat/fixtures/api_remember_v1.json` is round-tripped in the tests.

```sh
npm install
npm test
```
