# horus-sdk (Python, beta)

Python client for [HORUS](https://github.com/usehorus/horus). Tracks the
reference Rust SDK; beta until parity lands.

```bash
pip install horus-sdk
```

```python
from horus_sdk import Facts, commit, estimate_budget

facts = Facts(n_rows=12_000, schema_hash=b"\x00" * 32, freshness=1_730_000_000, field_presence=0)
c = commit(facts, salt=b"\x07" * 32)
budget = estimate_budget(per_query=1_000, n=50)  # 50_000 lamports
```

The commitment is BLAKE3 over little-endian fields and must match the Rust
implementation byte-for-byte — see RFC-0009 in the main repo.
