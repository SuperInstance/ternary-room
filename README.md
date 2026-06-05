# ternary-room: Room abstraction for multi-agent ternary environments

Rooms contain agents and environment state. Doors with ternary access control (locked/open/one-way) connect rooms. A coordinator manages agent transfers between rooms with event history tracking.

## Why This Exists

Multi-agent systems need spatial organization. Agents don't float in a void — they occupy rooms, move between them through controlled doors, and interact with room-local environment variables. This crate provides that spatial layer, inspired by PLATO room-based systems and Codespace environments where context shifts when you enter a different space.

## Core Concepts

- **Room** — A named space containing agents and an environment (key-value map). Rooms record an event history of agent entries and exits.
- **Door** — A connection between two rooms with access control. Three access modes: Locked (nobody passes), Open (bidirectional), OneWay (one direction only).
- **RoomBuilder** — Fluent builder for constructing rooms with initial agents and environment.
- **RoomState** — A snapshot of a room's agents and environment at a point in time, used for save/restore.
- **RoomHistory** — An append-only event log. Each event has a tick, agent ID, kind (enter/leave), and detail string.
- **RoomCoordinator** — Manages multiple rooms and doors. Handles agent transfer between rooms, checking door access before allowing movement.

## Quick Start

```toml
[dependencies]
ternary-room = "0.1"
```

```rust
use ternary_room::*;

// Build two rooms connected by an open door
let mut coord = RoomCoordinator::new();
coord.add_room(RoomBuilder::new(1, "lobby").agent(100).build());
coord.add_room(Room::new(2, "vault"));
coord.add_door(Door::new(1, 1, 2, DoorAccess::Open));

// Transfer agent 100 from lobby to vault
coord.transfer(100, 1, 2).unwrap();
assert_eq!(coord.room(1).unwrap().agent_count(), 0);
assert_eq!(coord.room(2).unwrap().agent_count(), 1);
```

## API Overview

| Type | Description |
|------|-------------|
| `Room` | Contains agents + environment. Records history. |
| `RoomBuilder` | Fluent builder: `.agent(id).env("key", "val").build()`. |
| `RoomState` | Snapshot of agents + environment for save/restore. |
| `RoomHistory` | Append-only event log with kind filtering. |
| `RoomEvent` | tick, agent_id, kind, detail. |
| `Door` | Connects two rooms with DoorAccess control. |
| `DoorAccess` | Locked, Open, OneWay(from, to). |
| `RoomCoordinator` | Manages rooms + doors, handles transfers. |

## How It Works

Rooms are independent containers. Each room tracks its own agent list and environment map as a `HashMap<String, String>`. When an agent enters or leaves, a `RoomEvent` is appended to the room's history.

Doors are separate objects that reference two room IDs. The `can_pass` method checks the door's access mode against the direction of travel. One-way doors store the allowed source room.

The `RoomCoordinator` ties it all together. It holds rooms in a `HashMap<u64, Room>` and doors in a `Vec<Door>`. On transfer, it verifies the agent exists in the source room, checks that at least one door allows passage, then removes from source and adds to destination.

Snapshots clone the agent list and environment map. Restore replaces both. This is a full overwrite — partial restores aren't supported.

## Known Limitations

- Doors are checked by existence only — if multiple doors connect the same two rooms with different access modes, any open door allows passage.
- RoomEvent tick values default to 0 when recorded via add_agent/remove_agent. You need to set ticks manually if you want accurate time tracking.
- No maximum capacity on rooms — you can add unlimited agents.
- Snapshot/restore is a full replacement, not a merge. Any agents added between snapshot and restore are lost.
- The coordinator's `transfer` method is O(doors) — with thousands of doors this could be slow.

## Use Cases

- **MUD/MOO-style game** — rooms are locations, doors are exits with varying permissions, agents are players or NPCs.
- **Factory floor simulation** — rooms are workstations, doors are conveyors (one-way for assembly lines), environment tracks machine state.
- **Chat rooms with access control** — rooms are channels, doors represent permissions (locked = private, open = public, one-way = broadcast-only).
- **Container orchestration** — rooms are nodes, agents are workloads, doors represent network policies.

## Ecosystem Context

`ternary-room` builds on the agent concept from `ternary-agent`. It uses agent IDs (u64) to track which agents are where, but doesn't import the crate — the ID-based approach keeps it decoupled. `ternary-world` uses rooms as part of its simulation grid.

## See Also

- **ternary-agent** — Core agent types with ternary state
- **ternary-channel** — Typed channels for ternary message passing
- **ternary-bus** — Message bus for inter-agent communication
- **ternary-navigator** — Pathfinding in ternary spatial graphs
- **ternary-consensus** — Distributed consensus for ternary decisions
- **ternary-harbor** — Service discovery and agent docking

## License

MIT
