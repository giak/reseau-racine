# EPIC 3 — MLS Forward Secrecy & Key Rotation

**Status:** Draft  
**Date:** 2026-05-25  
**Depends on:** EPIC 2 (Groupes & Cellules)  

## Motivation

EPIC 2 introduced E2EE group messaging on Nostr via a static shared group key (NIP-44 self-DH). Three weaknesses remain:

1. **No forward secrecy** — a leaked group key decrypts all past messages
2. **No key rotation** — a removed member keeps access to future messages
3. **No member departure** — `remove_member` in `CellStore` exists but is cosmetic; the shared key is unchanged

EPIC 3 replaces the static key scheme with **MLS (RFC 9420)** via the **Marmot Protocol** and its Rust implementation **MDK (Marmot Development Kit) v0.8**, providing:

- Forward secrecy (past messages stay secure after key leak)
- Post-compromise security (future messages stay secure after compromise)
- Efficient member add/remove (O(log N) TreeKEM)
- Scalability from 2 to thousands of members

## Architecture

```
CellTransport (unchanged public API)
    |
    v
MlsBackend (new, wraps MDK)
    |
    v
MDK (mdk-core + mdk-sqlite-storage)
    |
    v
OpenMLS (RFC 9420 TreeKEM)
```

### Principle

`CellTransport` keeps its public interface (`create_cell`, `invite_member`, `send_message`, `listen`, `remove_member`). The implementation switches from a static NIP-44 self-DH key to an MLS group managed by MDK. Consumers (CLI, future Tauri) see no API change.

## MlsBackend Interface

```rust
// crates/rr-core/src/mls_backend.rs

pub struct MlsBackend {
    mdk: MDK<MdkSqliteStorage>,
    keys: Keys,
}

impl MlsBackend {
    /// Create MLS group from member KeyPackage events
    pub async fn create_group(
        &self,
        members: &[Event],        // kind 443 KeyPackage events
        config: GroupConfig,      // label, relays, admin_pubkeys
    ) -> Result<CreateResult, CellTransportError>;

    /// Create MLS group from Cell (migration path)
    pub async fn create_group_from_cell(
        &self,
        cell: &Cell,
    ) -> Result<CreateResult, CellTransportError>;

    /// Propose + commit add member
    pub async fn add_member(
        &self,
        group_id: &[u8],           // MLS group ID
        key_package_event: &Event, // kind 443
    ) -> Result<Vec<Event>, CellTransportError>;  // commits + welcomes

    /// Propose + commit remove member
    pub async fn remove_member(
        &self,
        group_id: &[u8],
        member_pubkey: &PublicKey,
    ) -> Result<Event, CellTransportError>;

    /// Encrypt application message
    pub async fn create_message(
        &self,
        group_id: &[u8],
        rumor: UnsignedEvent,    // unsigned kind 9
    ) -> Result<Event, CellTransportError>;  // kind 445

    /// Decrypt received message
    pub async fn process_message(
        &self,
        group_event: &Event,     // kind 445
    ) -> Result<MessageResult, CellTransportError>;

    /// Process welcome (on first receipt after invite)
    pub async fn process_welcome(
        &self,
        welcome_event: &Event,   // kind 1059 → unwrapped kind 444
    ) -> Result<MlsGroupId, CellTransportError>;

    /// List pending welcomes
    pub fn pending_welcomes(&self) -> Result<Vec<MlsGroupId>, CellTransportError>;
}
```

### GroupConfig

```rust
pub struct GroupConfig {
    pub label: String,
    pub relays: Vec<RelayUrl>,
    pub admin_pubkeys: Vec<PublicKey>,
}
```

### Cell Kind

```rust
pub enum CellKind {
    Static,  // EPIC 2 — NIP-44 self-DH, no FS
    Mls,     // EPIC 3 — MLS via MDK, FS + PCS
}
```

`Cell` gains `kind: CellKind` and `mls_group_id: Option<Vec<u8>>`.

## CellTransport Changes

### create_cell

1. Validates member pubkeys
2. Fetches KeyPackage events (kind 443) from relays for each member
3. Calls `MlsBackend::create_group(members, config)`
4. Wraps each `Welcome` (kind 444) in gift-wrap (kind 1059) per member
5. Publishes gift-wrapped welcomes
6. Publishes group metadata as replaceable event (user's own store)
7. Returns `Cell { kind: Mls, mls_group_id, members, ... }`

### invite_member

1. Fetches KeyPackage for new member
2. `MlsBackend::add_member(group_id, key_package)`
3. Publishes resulting Commit events (kind 445) to cell's relay
4. Gift-wraps the Welcome (kind 444) for the new member
5. Updates cell members list in store + publishes group metadata

### send_message

1. Build rumor: unsigned Nostr event (kind 9, content = plaintext, pubkey = self)
2. `MlsBackend::create_message(mls_group_id, rumor)` → kind 445 event
3. Gift-wrap the kind 445 for each member → kind 1059
4. Publish each gift-wrap to relay

### listen

1. Subscribe to kind 1059 where `p` tag = self pubkey
2. For each received 1059: `MlsBackend::process_welcome()` if kind 444 inside
3. For each received 1059: unwrap → kind 445 → `MlsBackend::process_message()`
4. If 445 is Commit: apply MLS state change automatically (MDK handles)
5. If 445 is Application: extract unsigned kind 9, display

### remove_member

1. `MlsBackend::remove_member(group_id, member_pubkey)`
2. Publishes resulting Commit
3. Removes member from Cell.members
4. Remaining members receive Commit via listen → MDK processes → key rotated

## Migration Plan

### Phase 1 — MlsBackend side-by-side (current branch)

- Add `mdk-core` + `mdk-sqlite-storage` dependencies
- Create `MlsBackend` (initially only `create_group` + key package fetch)
- `Cell` gains `kind: CellKind` field (defaults to `Static` for existing cells)
- Existing `cell_transport.rs` unchanged
- SQLite file at `{RR_DATA_DIR}/mdk.db`

### Phase 2 — MLS creation path (this EPIC)

- `CellTransport::create_cell` creates MLS cell when all members publish KeyPackages
- `CellTransport::invite_member` uses MLS add path
- `CellTransport::send_message` uses MLS encryption (kind 445 → gift-wrap)
- `CellTransport::listen` processes MLS messages
- `CellTransport::remove_member` works via MLS Commit

### Phase 3 — Legacy migration (optional future)

- `rr group upgrade <CELL_ID>` — reads static cell, creates MLS group with same members, publishes migration message
- Old cell marked `kind: Static, migrated_to: <new_cell_id>`
- `listen` shows both until user confirms migration complete

### Database

`cells.json` is replaced by MDK's SQLite database at `{RR_DATA_DIR}/mdk.db`. The `Cell` metadata (label, members, kind, mls_group_id) is stored in a local table alongside MLS state.

```sql
CREATE TABLE cells (
    id TEXT PRIMARY KEY,
    label TEXT NOT NULL,
    kind TEXT NOT NULL DEFAULT 'mls',
    mls_group_id BLOB,          -- MLS group ID (private)
    nostr_group_id TEXT,         -- public group ID for h tag
    created_at_secs INTEGER NOT NULL
);

CREATE TABLE cell_members (
    cell_id TEXT NOT NULL REFERENCES cells(id),
    pubkey TEXT NOT NULL,
    label TEXT,
    added_at_secs INTEGER NOT NULL,
    PRIMARY KEY (cell_id, pubkey)
);
```

MDK handles its own MLS state tables (`mls_group`, `key_package`, `epoch`, etc.) internally.

## Nostr Event Kinds

| Kind | Usage | NIP |
|------|-------|-----|
| 443 | KeyPackage Event | MIP-00 |
| 444 | Welcome Event (gift-wrapped inside 1059) | MIP-02 |
| 445 | Group Event (Commit, Application) | MIP-03 |
| 10051 | KeyPackage Relays List | MIP-00 |
| 1059 | Gift Wrap (outer envelope) | NIP-59 |

All group messages (kind 445) use `exporter_secret` → conversation_key NIP-44 encryption, then gift-wrapped in kind 1059 per member.

## CLI Changes

No new CLI commands — existing `group` subcommands unchanged in behavior:

- `rr group create --label <L> --members <M>` → creates MLS group
- `rr group invite <ID> --member <M>` → MLS add member
- `rr group remove <ID> --member <M>` → MLS remove member (new)
- `rr group send <ID> --message <T>` → MLS encrypted message
- `rr group listen <ID>` → MLS message processing

## Error Handling

`CellTransportError` gains MLS-specific variants:

```rust
pub enum CellTransportError {
    // Existing
    Crypto(String),
    Store(String),
    Network(String),
    Relay(String),

    // MLS
    EpochMismatch,
    InvalidKeyPackage(String),
    MemberNotFound,
    CommitRaceCondition,
    WelcomeProcessing(String),
    GroupState(String),
}
```

## Testing

| Test | Type | Description |
|------|------|-------------|
| `mls_create_group` | unit (mock MDK) | Create group, verify Welcome events |
| `mls_send_receive` | unit (mock MDK) | Send message, process message, verify decryption |
| `mls_add_member` | unit (mock MDK) | Add member, verify Commit + Welcome |
| `mls_remove_member` | unit (mock MDK) | Remove member, verify Commit, key rotated |
| `mls_backend_integration` | integration (relay) | Full create → welcome → send → listen cycle |
| `cell_transport_no_regression` | existing | All existing CellTransport tests still pass |

## Risks

- **MDK v0.8 alpha** — API may break. Pin exact version, wrap behind `MlsBackend` trait that we control.
- **KeyPackage availability** — all members must publish kind 443 to reachable relays. `create_cell` fails with clear error if KeyPackages not found.
- **Commit race conditions** — NIP-EE spec defines tie-break by `created_at` + `id`. Implement same logic.
- **Storage migration** — SQLite replaces `cells.json`. Existing cells preserved as `Static` — no data loss.
