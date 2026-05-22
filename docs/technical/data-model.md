# Data Model

## Source Manifest

- Conversation: XMind CLI product and technical design discussion
- Scope: Rust domain model design for workbook, sheet, topics, selectors, and paths
- Last updated: 2026-05-22

## Principles

The domain model should encode product semantics directly. Avoid passing raw `String` values through the core when a stronger type can prevent mistakes.

Examples:

- `TopicPath`, not `String`.
- `TopicId`, not `String`.
- `SheetRef`, not loosely coupled title/id/index arguments.
- `Selector`, not raw selector text after parsing.
- `PatchOp`, not untyped YAML maps.

## Core Types

```rust
pub struct Workbook {
    pub sheets: Vec<Sheet>,
    pub resources: ResourceIndex,
    pub preservation: PreservationBag,
}

pub struct Sheet {
    pub id: SheetId,
    pub title: String,
    pub root: Topic,
}

pub struct Topic {
    pub id: TopicId,
    pub title: String,
    pub note: Option<String>,
    pub labels: Vec<String>,
    pub markers: Vec<String>,
    pub hyperlink: Option<String>,
    pub image: Option<TopicImageRef>,
    pub children: Vec<Topic>,
    pub preservation: PreservationBag,
}
```

## Paths

`TopicPath` is canonical and relative to the selected sheet root.

Rules:

- root path is `/`,
- root topic title is not a path segment,
- path segments are unescaped inside the Rust type,
- display and parsing handle escaping,
- path comparison is exact and case-sensitive.

Suggested API:

```rust
pub struct TopicPath(Vec<PathSegment>);

impl TopicPath {
    pub fn root() -> Self;
    pub fn parse_selector_value(input: &str) -> Result<Self, PathError>;
    pub fn to_selector_value(&self) -> String;
    pub fn parent(&self) -> Option<Self>;
    pub fn join(&self, segment: PathSegment) -> Self;
}
```

## Selectors

```rust
pub enum Selector {
    Root,
    Id(TopicId),
    Path(TopicPath),
    Title(String),
    Query(QueryExpr),
}
```

Selector resolution should return a typed result:

```rust
pub enum ResolveOne<T> {
    Found(T),
    NotFound { selector: Selector },
    Ambiguous { selector: Selector, candidates: Vec<Candidate> },
}
```

Do not collapse selector errors into strings. The error renderer needs candidates and original selector text.

## Patch Ops

Patch aliases should normalize at parse time:

```rust
pub enum PatchOp {
    AssertExists { node: Selector },
    AssertNotExists { node: Selector },
    Add { parent: Selector, topic: NewTopic, position: Position },
    AddTree { parent: Selector, tree: TopicTreeInput, position: Position },
    Set { node: Selector, fields: TopicFieldPatch },
    Delete { node: Selector, mode: DeleteMode },
    Move { node: Selector, to: Selector, position: Position },
    Copy { node: Selector, to: Selector, position: Position, preserve_ids: bool },
    ReplaceTree { node: Selector, tree: TopicTreeInput },
    MergeTree { target: Selector, tree: TopicTreeInput, match_by: MatchBy, prune: bool },
    EnsurePath { path: TopicPath },
    SortChildren { node: Selector, by: SortKey, order: SortOrder, recursive: bool },
    SetTreeMetadata { node: Selector, patch: MetadataPatch, recursive: bool },
}
```

## Preservation Bag

XMind data outside the first supported edit surface should be preserved explicitly:

```rust
pub struct PreservationBag {
    pub raw_json_fields: serde_json::Map<String, serde_json::Value>,
    pub package_entries: Vec<PreservedEntryRef>,
}
```

The writer should merge known edited fields with preserved unknown fields rather than reconstructing the package from only known fields.

## DTO Boundary

Use separate DTOs for:

- input files,
- command output JSON,
- internal domain model,
- XMind storage structures.

Never expose internal XMind storage structs as public command output.
