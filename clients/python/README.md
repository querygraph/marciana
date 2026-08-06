# Marciana Python client

This package is a deliberately thin, Pydantic v2 wire boundary. The caller
supplies an authenticated `MemoryTransport`; signing, authorization, storage,
and recovery remain server/adapter responsibilities.

Run its tests from the repository root:

```sh
PYTHONPATH=clients/python uv run --python 3.13 --with 'pydantic>=2.7,<3' \
  -- python -m unittest discover -s clients/python/tests -p 'test_*.py'
```
