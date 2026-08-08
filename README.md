# rlean-search

Type-aware search over **Lean 4** `theorem`, `lemma`, and `axiom` declarations.

Parses `.lean` sources (not `.olean`), understands **Lake** package layout, writes
XML indexes under the schema namespace

`http://github.com/createyourpersonalaccount/rlean-search`

and answers patterns such as:

| Pattern | Meaning |
|--------|---------|
| `_ + _ = 0` | holes match any subexpression |
| `?a - ?a = 0` | named holes must unify to the same term |
| `\|- tsum _ = _ * tsum _` | match only the main conclusion |

## Build

```bash
cargo build --release
```

## Index

```bash
rlean-search index path/to/Mathlib path/to/lean4/src \
  -o .rlean-search/index.xml
```

Lake-aware discovery reads `lakefile.toml` / `lakefile.lean`, `srcDir`, and
`lean_lib` roots, then walks `.lean` files (skipping `.lake`, `build`, etc.).

## One-shot search (uses cache when present)

```bash
rlean-search search '_ + _ = 0' -p path/to/pkg
rlean-search search '?a - ?a = 0' -p path/to/pkg --format jsonl
rlean-search search '|- _ * 1 = _' -p path/to/pkg --format xml
```

## Daemon mode (Tokio, multi-client)

Keeps the full type index in memory for low-latency queries:

```bash
rlean-search daemon --bind 127.0.0.1:7878 path/to/pkg
```

Clients speak **JSONL** or **XML**; the response format matches the request:

```bash
# JSONL
echo '{"cmd":"search","pattern":"_ + 0 = _","limit":20}' | nc 127.0.0.1 7878

# XML
echo '<rlean:search xmlns:rlean="http://github.com/createyourpersonalaccount/rlean-search" pattern="?a - ?a = 0" limit="20"/>' \
  | nc 127.0.0.1 7878

rlean-search query --pattern '_ + 0 = _'
rlean-search query --xml --pattern '|- _ = 0'
```

Commands: `search`, `stats`, `reload`, `ping`.

## XML schema

See [`schema/rlean-search.xsd`](schema/rlean-search.xsd). Index documents look like:

```xml
<rlean:index xmlns:rlean="http://github.com/createyourpersonalaccount/rlean-search" ...>
  <rlean:package name="mathlib" root="...">...</rlean:package>
  <rlean:declaration kind="theorem" name="add_comm" full_name="Nat.add_comm" ...>
    <rlean:typeSurface>∀ (n m : Nat), n + m = m + n</rlean:typeSurface>
    <rlean:type>...</rlean:type>
    <rlean:conclusion head="op:=">...</rlean:conclusion>
  </rlean:declaration>
</rlean:index>
```

## Tests

Fixtures under `tests/fixtures/` are derived from Lean 4 / Mathlib 4 type shapes
(from the accompanying `lean4-v4.32.2` / `mathlib4-v4.32.2` sources):

```bash
cargo test
```

## Coverage notes

The type parser targets a **useful fragment** of Lean surface types: binders,
arrows/Pi, applications, common infix operators (`+`, `*`, `=`, `≤`, `∧`, `↔`, …),
quantifiers, sorts, and search holes. Extremely advanced syntax may be stored as
`raw` nodes so declarations remain searchable by surface text and structure
where parsed.
