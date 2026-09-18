# Task 004 Persistence Feasibility

**Status:** APPROVED FEASIBILITY DECISION — planning checkpoint only; does not authorize Task 004 implementation, migrations, dependency changes, runtime changes, or `CODEX_TASK_004.md`

**Starting checkpoint:** `3b1b060` (`Approve NOUS Task 004 persistence architecture`)

**Review branch:** `planning/task-004-feasibility`

This note answers the narrow technical questions required before Task 004 implementation. It does not implement a repository, add a dependency, create a migration, change runtime behavior, or replace the approved architecture in `TASK_004_DESIGN.md`.

The labels **Verified**, **Recommendation**, and **Deferred** distinguish inspected facts from proposed implementation choices and details that still belong to the Task 004 implementation plan.

## 1. Conclusion

**Verified:** The current `tauri-plugin-sql` stack is sufficient for startup migration registration and for its JavaScript command bridge. It is not, by itself, a practical Rust application repository API: the plugin's connection, query, transaction, and migration-execution methods are crate-private.

**Recommendation:** Add one narrow, direct Rust dependency on the same SQLx release already present transitively, then use SQLx's SQLite API behind the approved concrete repository boundary. Prefer reusing the plugin-managed `SqlitePool` so startup migrations and repository operations use the same pool and the same resolved database. Do not add an ORM, a generic repository framework, a generic transaction framework, or a second production migration mechanism.

This is the smallest approach found that supports parameter binding, repository-owned atomic operations, `BEGIN IMMEDIATE`, useful database-error inspection, and isolated on-disk integration tests while preserving the approved architecture.

## 2. Current database stack

### Verified versions and features

Inspection of `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock`, `cargo tree`, and the locally installed crate sources found:

| Component | Resolved version | Relevant configuration |
| --- | --- | --- |
| `tauri` | `2.11.5` | Current desktop runtime |
| `tauri-plugin-sql` | `2.4.1` | Direct dependency with feature `sqlite` |
| `sqlx` | `0.8.6` | Transitive through the plugin; SQLite and Tokio runtime support are enabled |
| `sqlx-core` | `0.8.6` | Pool, connection, transaction, query, and database-error APIs |
| `sqlx-sqlite` | `0.8.6` | SQLite driver and connect options |
| `libsqlite3-sys` | `0.30.1` | Native SQLite binding used by the resolved driver stack; its `bundled` feature is enabled |

The resolved feature graph confirms `tauri-plugin-sql/sqlite -> sqlx/sqlite -> sqlx-sqlite/bundled -> libsqlite3-sys/bundled`, so the current stack uses bundled SQLite. The plugin also enables `sqlx/runtime-tokio`, and its own SQLx dependency enables SQLx's default feature set plus `json`, `time`, `uuid`, and `rust_decimal`. Task 004 does not need to request those optional data features directly merely because the plugin requires them transitively.

### Verified application setup

- `src-tauri/src/lib.rs` registers migration version 1 with `tauri-plugin-sql` and preloads `sqlite:nous.db`.
- `src-tauri/migrations/0001_initialize.sql` creates `app_metadata` and records schema version 1.
- `src-tauri/tauri.conf.json` also identifies the preloaded database as `sqlite:nous.db`.
- `src-tauri/capabilities/default.json` currently grants `sql:allow-load` and `sql:allow-select`, but not `sql:allow-execute`.
- `src/app/database.ts` loads the database through the JavaScript plugin and reads `app_metadata` with a raw `SELECT`.

## 3. What `tauri-plugin-sql` provides

### A. Migration registration

**Verified:** The plugin exposes a public builder and migration descriptors. The application already uses them to register versioned startup migrations. The plugin applies registered migrations while opening a preloaded database.

This is a suitable production migration authority and should remain the only production migration authority in Task 004.

### B. Frontend JavaScript bridge

**Verified:** The plugin registers Tauri commands for loading a database and forwarding query strings from JavaScript. The installed permission metadata controls which command names the frontend may invoke; it does not scope those commands to particular tables or SQL statement forms.

### C. Rust-side repository API

**Verified:** The installed `tauri-plugin-sql` source contains the underlying query and connection machinery, but its useful `connect`, `migrate`, `execute`, `select`, and `close` methods are `pub(crate)`. Application Rust code cannot call those methods as its repository API.

The plugin does publicly expose:

- `DbInstances`, whose public field contains the database-instance map; and
- `DbPool`, including its public `Sqlite(Pool<Sqlite>)` variant.

That public state appears sufficient to obtain and clone the plugin-managed SQLite pool, but actual repository queries and transactions still require a direct SQLx dependency. A transitive crate is not a stable or usable direct dependency merely because it is present in `Cargo.lock`.

## 4. Frontend SQL permission boundary

**Verified:** The frontend currently has permission to invoke the plugin's `load` and `select` commands. It does not have the separately named `execute` command permission.

**Security finding:** `sql:allow-select` is not a trustworthy read-only database boundary. In `tauri-plugin-sql` 2.4.1, the `select` command accepts a caller-supplied SQL string and forwards it to `sqlx::query(...).fetch_all(...)`; the inspected command path does not validate that the statement is a `SELECT`. SQLite data-changing statements can be prepared through the same query API and need not return rows. Therefore the current command permission exposes an unscoped raw-SQL path, despite the command's name. This conclusion comes from static source inspection; it was not exercised against the application database in this review.

**Recommendation:** Task 004 must remove frontend access to an unscoped SQL command path for the domain database. The existing infrastructure-only `app_metadata` read is not precedent for frontend domain SQL. Replace that readiness/version read with a narrow Rust-owned command or state boundary, then remove SQL bridge permissions that are no longer required. If any plugin command remains exposed, it must not allow caller-selected SQL against Self Model tables.

**Deferred:** The exact capability edit and the exact replacement for `src/app/database.ts` should be selected during implementation after checking application startup behavior. This feasibility note records the required security outcome without changing permissions now.

## 5. Recommended Rust persistence approach

### Direct dependency

**Recommendation:** Declare SQLx directly for Task 004 with an exact `=0.8.6` version requirement and only the direct features NOUS itself needs, conceptually:

```toml
sqlx = { version = "=0.8.6", default-features = false, features = ["sqlite", "runtime-tokio"] }
```

Cargo feature unification may still enable additional SQLx features required transitively by `tauri-plugin-sql`; disabling default features on NOUS's direct declaration does not disable features selected elsewhere in the graph. Use runtime queries such as `sqlx::query(...).bind(...)`; Task 004 does not require SQLx macros, an ORM, or a generic abstraction layer.

This is a new direct manifest dependency, but not a new database stack: the same SQLx release and SQLite driver are already compiled for `tauri-plugin-sql`. Matching the release avoids duplicate SQLx types and allows the application to use the plugin's public `SqlitePool` value.

### Production pool ownership

**Recommendation:** Obtain the plugin-managed `DbInstances` from Tauri state, select the entry for `sqlite:nous.db`, require its `DbPool::Sqlite` variant, and give a clone of that `SqlitePool` to a concrete `SqliteSelfModelRepository` (or equivalent narrow concrete boundary).

This does not create a generic repository or a Unit of Work. The application service still chooses and orchestrates a user-intent operation. Each intention-revealing repository method owns the SQL transaction that makes its persistence operation atomic.

**Deferred:** Compile-prove the exact state extraction and setup ordering before implementation is accepted. The installed plugin types are public, but the plugin has no purpose-built public pool accessor, and commented-out accessor code in its source suggests this surface deserves an explicit compatibility check.

## 6. Transaction feasibility

**Verified:** SQLx 0.8.6 exposes `Pool::begin_with(statement)` and SQLite supports a custom begin statement. A repository operation can use:

```text
BEGIN IMMEDIATE
  -> parameterized INSERT/SELECT statements on the returned transaction
  -> COMMIT on success
  -> ROLLBACK on failure or transaction drop
```

All statements executed through the returned transaction use the same acquired connection. This supports the approved atomic operations, including entity creation plus initial revision and append plus repository-generated revision number.

`BEGIN IMMEDIATE` obtains SQLite's reserved write lock before reading the current maximum revision number. Together with the required uniqueness constraints, this supports deterministic `MAX(revision_number) + 1` allocation under concurrent writers. Busy/locked outcomes must remain explicit persistence errors rather than being silently retried into different semantics.

**Verified:** SQLx provides parameter binding and exposes database errors through `sqlx::Error::Database`, including codes and broad error kinds such as unique, foreign-key, not-null, and check violations. Repository code can translate expected constraint failures into narrow persistence/domain-facing errors without exposing raw SQL details to the frontend.

## 7. Migration and repository coexistence

**Conclusion:** Using the plugin for startup migration registration and SQLx for Rust repository operations against the same `nous.db` is safe and practical when they share the plugin-managed pool. In that arrangement:

- the plugin remains the single production migration authority;
- repository commands cannot run until plugin setup and preload migration have completed;
- both paths use the same resolved database location;
- no second pool is introduced;
- SQLite locking is handled by the same driver pool; and
- Task 004 does not need to introduce WAL mode merely to make this architecture work.

SQLite foreign-key enforcement is connection-specific. The inspected SQLx SQLite options enable foreign keys by default, but Task 004 should verify the required pragma on repository connections and fail clearly if the invariant is not active. A one-time `PRAGMA foreign_keys = ON` executed on an arbitrary pooled connection is not sufficient proof for every connection.

**Fallback, not preferred:** If compile work shows that consuming the plugin-managed pool is too brittle, an application-owned SQLx pool can open the exact Tauri application-data path for `nous.db` after plugin migrations finish. Two pools can safely target one SQLite file under SQLite locking, but this adds path-resolution, readiness, busy-timeout, per-connection foreign-key, and shutdown coordination. It must not add a second migration authority. Use this only if the shared-pool approach fails a concrete implementation test.

## 8. Test database feasibility

**Verified:** SQLx SQLite connect options support file databases, `create_if_missing`, configurable busy timeout and journal mode, and explicit pool close. Task 004 integration tests can therefore:

1. create a unique temporary directory and SQLite file;
2. open an isolated test pool (preferably with a deliberately small connection count);
3. execute the actual versioned migration SQL used by production;
4. construct the concrete repository with that pool;
5. exercise atomic success, rollback, history invariants, foreign keys, and constraint translation;
6. close every pool handle;
7. reopen the same file and verify persisted history; and
8. remove the temporary directory after handles are closed.

Migration compatibility can be tested from Rust by executing the same checked-in SQL migration files against the temporary database. That test harness is not a second production migration system. The exact helper structure and whether it shares a migration registry with application setup are deferred to implementation.

In-memory SQLite is available but should not replace the approved temporary on-disk tests: separate in-memory connections can represent separate databases, and they do not prove close/reopen persistence.

## 9. Important implementation constraints

Task 004 implementation should preserve all of the following:

- Keep the direction `Rust application service -> narrow repository boundary -> SQLite`.
- Keep transaction mechanics in the concrete SQLite repository.
- Make create-with-initial-revision and append-revision repository operations atomic.
- Generate persisted revision numbers in the repository operation, not in `RevisionNumber` or the frontend.
- Use `BEGIN IMMEDIATE` for revision appends and validate uniqueness constraints as a second line of defense.
- Enforce persisted origin history: revision 1 is `InitialUserEntry`; later revisions are only `UserUpdate` or `UserCorrection`; append rejects `InitialUserEntry`.
- Preserve provenance and immutable revision history.
- Keep the plugin registration as the only production migration authority.
- Ensure repository readiness follows successful startup migrations.
- Use parameter binding for values; do not build SQL from user input.
- Verify foreign-key enforcement for every connection used by repository operations.
- Keep frontend code from caller-selected SQL against Self Model tables.
- Use temporary on-disk integration databases and include close/reopen coverage.
- Do not add Task 005/006 behavior, an ORM, generic repositories, mocks, Unit of Work, or a generic transaction framework.

## 10. Alternatives considered

### Use `tauri-plugin-sql` alone

Rejected. Its migration builder and frontend bridge do not expose the Rust query and transaction API required by the repository. Reimplementing around crate-private methods is not viable.

### Use frontend plugin commands for domain persistence

Rejected. This would invert the approved Rust boundary, expose raw domain SQL to React, and make repository-owned transactions and invariant enforcement ineffective.

### Add `rusqlite`

Not recommended for this repository. `rusqlite` is a capable narrow SQLite library, but it is not currently part of the resolved stack. Adding it would introduce a second SQLite API, synchronous connection management, and possible native SQLite linkage/version coordination alongside `libsqlite3-sys` already used by SQLx. It would also make sharing the plugin-managed pool impossible. Direct SQLx is smaller in the context of this application because SQLx is already present.

### Open an independent production SQLx pool immediately

Not the first choice. It is technically workable, but creates avoidable coordination around database-path resolution, migration timing, connection pragmas, and locking. Retain it only as the bounded fallback described above.

### Add an ORM or generic database abstraction

Rejected. Task 004 has a small, explicit SQLite persistence surface and no demonstrated need for either abstraction.

## 11. Remaining uncertainty

The following points require implementation-time proof but do not block the feasibility conclusion:

1. Compile-prove external access to the plugin-managed `DbInstances` map and `DbPool::Sqlite` pool with the direct, version-matched SQLx dependency.
2. Confirm the exact database-map key and Tauri setup/state ordering used when constructing or resolving the repository.
3. Add a security regression test showing that no frontend invoke path accepts caller-selected domain SQL. The current `allow-select` risk is established from source inspection, not a runtime write probe.
4. Select the narrow Rust readiness/schema-version command that replaces the frontend `app_metadata` raw query, then remove only the plugin permissions made unnecessary by that replacement.
5. Choose the smallest test helper that runs the checked-in migration SQL without becoming a second production migration registry.
6. Confirm busy-timeout behavior under the repository's concurrent append tests. Do not introduce WAL mode unless those tests demonstrate a concrete need.

No temporary compile probe was necessary for this review; the installed source exposed the relevant API visibility, transaction, configuration, and error behavior directly.

## 12. Feasibility decision

Task 004's approved persistence architecture is technically feasible with the current stack plus one narrowly declared, exact-pinned direct dependency on SQLx `=0.8.6`. The preferred implementation reuses the plugin-managed SQLite pool, keeps startup migrations in `tauri-plugin-sql`, performs all Self Model domain SQL inside the Rust repository boundary, and removes the frontend's unscoped raw-SQL access path.

This decision does not authorize implementation. A bounded Task 004 specification and its separate approval/checkpoint remain required before migrations, dependencies, repositories, capability changes, or runtime behavior are modified.
