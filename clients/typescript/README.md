# Marciana TypeScript client

This is the TypeScript counterpart to `clients/python`: strict request
validation and a transport/authentication seam for the four memory verbs.
Authorization, signing, storage, and recovery remain server responsibilities.
Wire fields intentionally use the shared snake_case contract (`memory_id`,
`memory_ids`) used by the Rust and Python clients.

```sh
npm install
npm run build
node --test test/client.test.mjs
```
