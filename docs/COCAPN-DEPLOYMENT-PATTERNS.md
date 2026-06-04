# What ternary-room Can Learn from cocapn Deployment Patterns

*Lessons from cocapn-runtime's 5-mode deployment architecture applied to ternary-room's room abstraction.*

---

## 1. Auto-Environment Detection

### What cocapn does

cocapn-runtime's `boot.sh` detects the runtime environment and selects the appropriate deployment mode automatically:

- Checks `CODESPACES` env var for GitHub Codespaces
- Checks `/.dockerenv` for Docker containers
- Checks `uname -m` for ARM64 (Jetson/Pi) vs x86_64
- Checks network connectivity for lighthouse vs offline
- Checks `/etc/nv_tegra_release` for Jetson-specific features

### What ternary-room should learn

Currently, ternary-room's `Room` is a generic struct with no awareness of its deployment environment. The `environment` HashMap carries metadata but nothing auto-populates it.

**Proposal: Auto-detect and populate room environment on creation.**

```rust
impl Room {
    /// Create a room with auto-detected environment configuration.
    pub fn autodetect(id: u64, name: &str) -> Self {
        let mut room = Room::new(id, name);

        // Auto-detect hardware tier
        let tier = detect_tier();
        room.set_env("tier", &format!("{:?}", tier));

        // Auto-detect construct-core layer
        let layer = detect_layer();
        room.set_env("layer", &format!("{:?}", layer));

        // Auto-detect network connectivity
        if check_lighthouse() {
            room.set_env("mode", "lighthouse");
        } else if std::env::var("CODESPACES").is_ok() {
            room.set_env("mode", "codespace");
        } else if std::path::Path::new("/.dockerenv").exists() {
            room.set_env("mode", "container");
        } else {
            room.set_env("mode", "offline");
        }

        // Auto-detect hardware specifics
        let arch = std::env::consts::ARCH;
        room.set_env("arch", arch);

        if std::path::Path::new("/etc/nv_tegra_release").exists() {
            room.set_env("gpu", "orin-nano");
            room.set_env("cuda_cores", "1024");
        }

        // Memory detection
        if let Ok(meminfo) = std::fs::read_to_string("/proc/meminfo") {
            if let Some(line) = meminfo.lines().find(|l| l.starts_with("MemTotal:")) {
                let kb: u64 = line.split_whitespace().nth(1)
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
                room.set_env("memory_mb", &format!("{}", kb / 1024));
            }
        }

        room
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Tier {
    Bare,       // ESP32, Cortex-M
    Edge,       // Jetson, Pi
    Cloud,      // Codespace, VPS
    Sandbox,    // Docker container
}
```

---

## 2. Five-Mode Deployment Flexibility

### What cocapn does

Every git-agent repo works in all 5 modes. The same repo boots differently depending on where it wakes up. This is fundamentally different from "deploy a different binary for each target."

### What ternary-room should learn

ternary-room currently has one `Room` struct that works everywhere but isn't specialized. It should adopt the pattern of **specialized room types that share a common interface**:

```rust
/// RoomKind — the deployment mode of a room.
/// Each kind has different capabilities and constraints.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoomKind {
    /// Always-on cloud room. Full PLATO sync, Holodeck, fleet coordination.
    Lighthouse,

    /// Ephemeral cloud room. Spins up on demand, suspends when idle.
    Codespace,

    /// Edge room with tender sync. Works offline, queues messages.
    Tender,

    /// Sandboxed room. Resource-limited, isolated.
    Container,

    /// Bare metal room. No heap, compiled policy, no dynamic loading.
    BareMetal,
}

impl Room {
    /// Get the room's deployment kind (auto-detected on creation).
    pub fn kind(&self) -> RoomKind {
        match self.get_env("mode") {
            Some("lighthouse") => RoomKind::Lighthouse,
            Some("codespace") => RoomKind::Codespace,
            Some("tender") | Some("offline") => RoomKind::Tender,
            Some("container") => RoomKind::Container,
            Some("bare-metal") => RoomKind::BareMetal,
            _ => RoomKind::Lighthouse, // Default
        }
    }

    /// Can this room dynamically load skills?
    pub fn supports_dynamic_skills(&self) -> bool {
        matches!(self.kind(), RoomKind::Lighthouse | RoomKind::Codespace | RoomKind::Container)
    }

    /// Can this room sync with PLATO?
    pub fn supports_plato_sync(&self) -> bool {
        matches!(self.kind(), RoomKind::Lighthouse | RoomKind::Codespace | RoomKind::Container)
    }

    /// Can this room run async operations?
    pub fn supports_async(&self) -> bool {
        matches!(self.kind(), RoomKind::Lighthouse | RoomKind::Codespace | RoomKind::Container)
    }
}
```

---

## 3. Tender Sync Pattern for Offline Rooms

### What cocapn does

Mode 3 (Tender) allows edge agents to work fully offline. A "tender" agent visits periodically via Bluetooth or local network, carrying updates in both directions. The edge agent queues all outbound messages and drains them when the tender arrives.

### What ternary-room should learn

ternary-room has no concept of message queuing or offline operation. A `Room` can't queue messages for later delivery. The `RoomCoordinator::transfer()` is synchronous and immediate.

**Proposal: Add sync queue support to rooms.**

```rust
/// A sync queue for rooms that can't deliver messages immediately.
pub struct SyncQueue {
    outbound: Vec<QueuedMessage>,
    inbound: Vec<QueuedMessage>,
    last_sync: Option<std::time::Instant>,
}

pub struct QueuedMessage {
    pub target_room: u64,
    pub message_type: MessageType,
    pub payload: Vec<u8>,
    pub queued_at: std::time::Instant,
    pub priority: Priority,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Low,      // Diary entries, general observations
    Normal,   // Task updates, skill results
    High,     // Anomaly alerts, fleet messages
    Critical, // System failures, safety events
}

impl Room {
    /// Queue a message for later delivery (for offline/tender rooms).
    pub fn queue_message(&mut self, target: u64, msg_type: MessageType, payload: Vec<u8>, priority: Priority) {
        if let Some(queue) = &mut self.sync_queue {
            queue.outbound.push(QueuedMessage {
                target_room: target,
                message_type: msg_type,
                payload,
                queued_at: std::time::Instant::now(),
                priority,
            });
        }
    }

    /// Drain queued messages (called when tender syncs).
    pub fn drain_outbound(&mut self) -> Vec<QueuedMessage> {
        if let Some(queue) = &mut self.sync_queue {
            let mut outbound = std::mem::take(&mut queue.outbound);
            outbound.sort_by(|a, b| b.priority.cmp(&a.priority)); // Priority order
            outbound
        } else {
            Vec::new()
        }
    }

    /// Accept inbound messages from a sync.
    pub fn accept_inbound(&mut self, messages: Vec<QueuedMessage>) {
        if let Some(queue) = &mut self.sync_queue {
            queue.inbound.extend(messages);
            queue.last_sync = Some(std::time::Instant::now());
        }
    }
}
```

### Integration with RoomCoordinator

```rust
impl RoomCoordinator {
    /// Sync two rooms via a tender agent. Only works for rooms with sync queues.
    pub fn tender_sync(&mut self, tender_agent: u64, room_id: u64) -> Result<SyncReport, String> {
        // Collect outbound from the room
        let room = self.rooms.get_mut(&room_id)
            .ok_or_else(|| format!("room {} not found", room_id))?;

        let outbound = room.drain_outbound();

        // The tender carries these messages to the fleet
        // (In practice, the tender has its own connection to lighthouse)

        Ok(SyncReport {
            room_id,
            items_outbound: outbound.len(),
            items_inbound: 0, // Will be populated when tender returns
        })
    }
}
```

---

## 4. Holodeck Spatial Coordination

### What cocapn does

The Holodeck MUD provides a spatial abstraction layer on top of the fleet. Agents can "walk" between rooms, "look" at their surroundings, and "talk" to other agents. The MUD doesn't change the underlying room mechanics — it adds a human-readable spatial framing.

### What ternary-room should learn

ternary-room has `RoomCoordinator` with rooms and doors, but no spatial description or human-facing interface. It's pure data structure.

**Proposal: Add spatial metadata to rooms and doors.**

```rust
impl Room {
    /// Set the MUD-style description for this room.
    pub fn set_description(&mut self, desc: &str) {
        self.set_env("mud_desc", desc);
    }

    /// Get the room's description.
    pub fn description(&self) -> &str {
        self.get_env("mud_desc").unwrap_or("An empty room.")
    }

    /// Generate a "look" description (room + agents + exits).
    pub fn look(&self) -> String {
        let mut output = String::new();
        output.push_str(self.description());
        output.push_str("\n\n");

        if !self.agents.is_empty() {
            output.push_str(&format!("Agents here: {}\n", self.agents.len()));
            for &agent in &self.agents {
                output.push_str(&format!("  • agent-{}\n", agent));
            }
        }

        output
    }
}

impl Door {
    /// Get a human-readable description of this door.
    pub fn describe(&self, from_room: &Room, to_room: &Room) -> String {
        let access_str = match &self.access {
            DoorAccess::Locked => "🔒 locked".to_string(),
            DoorAccess::Open => "🚪 open".to_string(),
            DoorAccess::OneWay(_, _) => "➡️ one-way".to_string(),
        };
        format!("{} → {} ({})", from_room.name, to_room.name, access_str)
    }
}

impl RoomCoordinator {
    /// Generate a full map of all rooms and doors (for visualization).
    pub fn map(&self) -> String {
        let mut output = String::from("Fleet Map:\n");
        for (&id, room) in &self.rooms {
            output.push_str(&format!("  [{}] {} ({} agents)\n", id, room.name, room.agent_count()));
        }
        output.push_str("\nDoors:\n");
        for door in &self.doors {
            let a_name = self.rooms.get(&door.room_a).map(|r| r.name.as_str()).unwrap_or("?");
            let b_name = self.rooms.get(&door.room_b).map(|r| r.name.as_str()).unwrap_or("?");
            let access = match &door.access {
                DoorAccess::Locked => "locked",
                DoorAccess::Open => "open",
                DoorAccess::OneWay(_, _) => "one-way",
            };
            output.push_str(&format!("  {} ↔ {} ({})\n", a_name, b_name, access));
        }
        output
    }
}
```

---

## 5. Boot Sequence Integration

### What cocapn does

The `boot.sh` script is the single entry point. It detects the environment, prints diagnostics, and hands off to the correct runtime with appropriate environment variables.

### What ternary-room should learn

ternary-room should have a "boot" function that mirrors boot.sh's logic in Rust:

```rust
/// Boot a room coordinator from the current environment.
/// Equivalent to boot.sh but in Rust.
pub fn boot_fleet() -> RoomCoordinator {
    let mut coord = RoomCoordinator::new();

    // Detect this machine's room type
    let room = Room::autodetect(0, "local");

    // Add this machine as a room
    coord.add_room(room);

    // If connected, discover other rooms from the fleet
    if check_lighthouse() {
        // Query the fleet registry for all known rooms
        if let Ok(fleet_rooms) = discover_fleet_rooms() {
            for room_info in fleet_rooms {
                coord.add_room(room_info.to_room());
            }
            // Connect doors based on fleet topology
            for door_info in discover_fleet_doors() {
                coord.add_door(door_info.to_door());
            }
        }
    }

    coord
}
```

---

## Summary of Changes

| Feature | Current ternary-room | With cocapn patterns |
|---|---|---|
| Environment detection | Manual env set | `Room::autodetect()` |
| Deployment modes | Generic Room | `RoomKind` enum with capability queries |
| Offline support | No queuing | `SyncQueue` with priority ordering |
| Spatial description | No metadata | `set_description()`, `look()`, `map()` |
| Boot sequence | Manual setup | `boot_fleet()` auto-discovery |
| Room types | One generic struct | Specialized by `RoomKind` |
| Door metadata | ID + access only | Human-readable descriptions |

These changes make ternary-room deployment-aware while preserving its existing API. All new features are additive — existing code continues to work.

---

*Written 2026-06-04 by synthesis-cocapn-fleet subagent.*
