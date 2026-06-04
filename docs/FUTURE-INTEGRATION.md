# Future Integration: ternary-room

## Current State

ternary-room provides the room abstraction for multi-agent ternary environments. `Room` contains agents (by ID), an environment map (`HashMap<String, String>`), and `RoomHistory` recording `RoomEvent` entries (enter/leave events with tick, agent_id, kind, detail). `Door` connects two rooms with `DoorAccess` states (Locked, Open, OneWay). `RoomCoordinator` manages agent transfers between rooms via doors, enforcing access control. `RoomBuilder` provides fluent construction with initial agents and environment variables. `RoomState` snapshots enable save/restore of room configuration.

## Integration Opportunities

### PLATO Room-as-Codespace (Primary Integration)

The `Room` struct is the foundation of the PLATO room-as-codespace architecture. Current `Room` is in-memory and generic. To become a codespace-backed room:

- **`Room::environment`** → hardware tier configuration. Set `env("tier", "codespace")` vs `env("tier", "esp32")`. The room's environment drives which `construct-core` layer is used.
- **`RoomCoordinator::transfer()`** → agent walking between rooms. When an agent transfers from room A to room B, room A's `remove_agent()` triggers ensign unloading + trigger extraction, and room B's `add_agent()` triggers ensign loading + PLATO tile sync.
- **`RoomHistory`** → audit trail for all room interactions. In PLATO, this becomes the vessel diary — a permanent record of what happened in each room.

### Codespace/ESP32/Jetson Room Implementations

The `Room` struct needs a `RoomType` enum to specialize behavior:

- **`CodespaceRoom`**: `Room` with `env("backend", "codespace")`, `DoorAccess::Open` to PLATO proxy room, full construct-core Layer 2.
- **`EdgeRoom`**: `Room` with `env("backend", "jetson")`, local model inference, Layer 1.
- **`BareRoom`**: `Room` with `env("backend", "esp32")`, no doors (compiled-in), Layer 0. `RoomCoordinator::transfer()` always fails for bare rooms.

### Door → PLATO Tile Sync Channel

`Door` with `DoorAccess::OneWay(from, to)` models the PLATO proxy pattern: API keys flow one direction (PLATO → room), responses flow back. Adding a `sync_channel()` method to `Door` that enables tile synchronization between connected rooms. Tiles flow from CodespaceRoom → PLATO → EdgeRoom through door-mediated channels.

### ternary-cell → Room Tick

Each `Room` should contain a `CellGrid` (from ternary-cell). When `RoomCoordinator` ticks all rooms, each room's grid runs `tick_all()`. The `RoomState` snapshot includes the grid state. Agents in the room interact with cells: agent decisions modify cell values, cell signals influence agent behavior.

## Potential in Mature Systems

The room abstraction unifies all compute environments. A mature PLATO deployment has dozens of rooms: `Room { name: "engine-monitor", environment: { "tier": "edge", "ensigns": "kalman,sensor" } }`, `Room { name: "music-theory", environment: { "tier": "codespace", "ensigns": "music,algebra" } }`. `RoomCoordinator` is the campus — agents walk between rooms through doors. The `OneWay` door prevents unauthorized room transitions (a sentinelle agent can't walk into a capitaine-only room).

## Cross-Pollination Ideas

- **Room snapshots → git commits**: `RoomState` snapshots serialize to JSON and commit to git. Room history IS git log. `Room::restore(snap)` = `git checkout`.
- **Doors as I2I bottles**: `Door` between rooms in different orgs (SuperInstance ↔ Lucineer) carries I2I messages. `DoorAccess::OneWay` = fork-PR pattern (code flows one way).
- **Room environment → ternary-spreadsheet cells**: Each `Room` IS a spreadsheet cell. `environment` = cell metadata, `agents` = cell formulas, `history` = cell audit trail.

## Dependencies for Next Steps

1. `RoomType` enum and `Room::room_type()` method
2. `Room` → `CellGrid` composition (one grid per room)
3. `RoomCoordinator` → PLATO proxy integration for cross-room tile sync
4. `Door` enhancement with sync channels and bandwidth tracking
5. Serialization of `RoomState` → JSON for git-based persistence
