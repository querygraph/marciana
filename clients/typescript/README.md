# Marciana TypeScript client

This is the TypeScript counterpart to `clients/python`: strict request
validation and a transport/authentication seam for the four memory verbs.
Authorization, signing, storage, and recovery remain server responsibilities.

```sh
npm install
npm run build
node --test test/client.test.mjs
```
