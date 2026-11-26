# Homoiconic records audit

Revision audited: `mycelium-linux/records` `1fa03a239b6191f61ecf9d5f9c5284ba9045a1cd`
(the remote named in this checkout; the founder’s `FuturePresentLabs/records`
label refers to the same code). Worktree: `/tmp/records-homoiconic`.

## What the store actually holds

- `src/lib.rs`: `Op.body` and `Record.fields` are `serde_json::Value`.
- SQLite: `ops.body TEXT`, `records.fields TEXT`; `revision` is a
  `blake3:<hex>` content hash, while `updated_hlc` is the sync clock.
- `RecordEntry.fields` is JSON. `put_record`/`append` accept JSON only.
- Sync: per-device op log, HLC, `(book_id, device_id, seq)` uniqueness,
  `op_id` BLAKE3 over the envelope plus `serde_json::to_string(body)`.
- Changes: materialized `scan_changes` with tombstones, LWW by
  `(updated_hlc, origin)`; no per-field merge.
- Query: `search_book` is SQLite `fields LIKE '%text%'`; `query_records` is a
  cross-namespace substring search with an opaque revision cursor. There is no
  field-level index or structural query in this repo.
- Schema: schema id + version are stored per record; no migration table or
  schema registry. `soil:records` WIT is `0.1.0` with opaque `body: list<u8>`.

## Native Soil values and the JSON boundary

`fungOS/soil/crates/soil-lang/src/sexp.rs` defines exactly:

`Nil | Bool | Int | Float | Symbol | Keyword | String | List`.

There is no Vector or Map primitive in `Value`.

`soil-lang/src/eval.rs` JSON conversion is lossy:

- `Symbol`, `Keyword`, and `String` all become JSON strings.
- `Nil` becomes JSON null; an empty list becomes `[]`.
- every `List` becomes a JSON array unless *all* elements are two-element
  lists starting with a keyword; then it becomes a JSON object.
- a quoted form is just a list beginning with `quote`; it round-trips as data,
  but JSON has no quote marker.
- `Int`/`Float` survive as JSON numbers; no bignum; non-finite floats become
  null in the encoder.
- maps are not a primitive; the object heuristic loses order and cannot
  distinguish a map from a list of keyword pairs.

`soil-lang/src/records_host.rs` encodes `Record.fields` with
`serde_json::to_vec` and decodes bodies by `serde_json::from_slice`, so the same
loss applies at the host boundary. `soil-sdk-records` uses `JsonCodec` on the
same bytes.

## Loam consumers / version skew

`loam/crates/loam/src/records_host` assumes JSON: `validate_body` parses
`serde_json::Value`, queries use `json_extract(fields, ?)`, and projections
build JSON objects. It exposes a newer `soil:records` shape
(`query`/`aggregate`/`scan-packed`/`apply-result`) that differs from this
repository’s `wit/soil-records.wit` (cross-namespace `query`/`watermark`) while
both still say `@0.1.0`. That is an **incompatible same-version interface** and
must be resolved before any WIT migration.

## What native values already round-trip

- JSON scalars, arrays, objects, nil-as-null: yes, through the existing store.
- Nested lists/forms as data: yes, as JSON arrays.
- Symbols vs keywords vs strings: **no** (all strings).
- `nil` vs empty list: **no** (null vs `[]` by shape, but a Soil `Nil` and an
  empty `List` are distinct values and JSON has no tag).
- list vs map: **no** (object heuristic).
- quote vs ordinary list: only by inspecting the first symbol; no explicit tag.
- stable content identity: `op_id`/`revision` use non-canonical JSON
  serialization choices; map order is deterministic in serde_json’s default
  BTreeMap, but cross-language canonicalization is not specified.

## Smallest compatible improvement staged

A new module `src/form.rs` adds an **optional** schema
`fungos.soil/form` and a tagged canonical JSON envelope:

```json
{"$soil":"form","v":1,"value":["list",[["symbol","defn"],["symbol","f"]]]}
```

It preserves exactly the eight native `Value` variants, distinguishes
symbol/keyword/string and nil/empty list, keeps quotes as inert lists, computes
a stable BLAKE3 digest, and fails closed on unknown tags/versions/non-finite
floats. It is additive: existing records, WIT, SQLite schema, JSON bodies, and
all host query paths are unchanged. A consumer opts in by using the schema and
codec. It does not evaluate forms or grant capabilities.

Tests in `src/form.rs` cover every native variant, symbol/keyword/string and
nil/empty-list non-collisions, stable digests, malformed tags, and a real
`Store::put_record`/readback/search round-trip. No production migration is
included.

## Decision points for Marbles / Singularity

1. Keep the existing JSON body for existing schemas; adopt `fungos.soil/form`
   only for new homoiconic records.
2. Resolve the `@0.1.0` WIT mismatch before adding a `body-format` field.
3. If structural queries over tagged forms are needed, add JSON-path indexes
   or a projection table; do not make the form codec an evaluator.
4. No vectors/maps until `soil-lang::Value` has them; do not invent a body shape
   no native reader can produce.

## Test receipt

Remote Linux build/test (Bob, Agora One), revision `69d0099`:

```text
cargo test -p fungos-records form::tests
running 4 tests ... 4 passed; 0 failed
cargo test -p fungos-records
23 passed; 0 failed
```
