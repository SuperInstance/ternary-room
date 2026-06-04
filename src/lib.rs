#![forbid(unsafe_code)]

//! Room abstraction for multi-agent ternary environments.
//!
//! A `Room` contains agents and an environment state. Rooms are connected by
//! `Door` objects with ternary access (locked, open, one-way). A `RoomCoordinator`
//! manages agent transitions between rooms. `RoomHistory` records events.

use std::collections::HashMap;

// ── Door ───────────────────────────────────────────────────────────────────

/// Access state of a door connecting two rooms.
///
/// - `Locked`: no agent may pass
/// - `Open`: agents may pass in both directions
/// - `OneWay(from, to)`: agents may only pass from `from` to `to`
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DoorAccess {
    Locked,
    Open,
    OneWay(u64, u64), // from_room, to_room
}

/// A connection between two rooms with controlled access.
#[derive(Debug, Clone)]
pub struct Door {
    pub id: u64,
    pub room_a: u64,
    pub room_b: u64,
    pub access: DoorAccess,
}

impl Door {
    pub fn new(id: u64, room_a: u64, room_b: u64, access: DoorAccess) -> Self {
        Self { id, room_a, room_b, access }
    }

    /// Check whether an agent in `from_room` can pass to the other side.
    pub fn can_pass(&self, from_room: u64) -> bool {
        match &self.access {
            DoorAccess::Locked => false,
            DoorAccess::Open => from_room == self.room_a || from_room == self.room_b,
            DoorAccess::OneWay(src, _) => from_room == *src,
        }
    }

    /// Get the destination room if the agent in `from_room` can pass.
    pub fn destination(&self, from_room: u64) -> Option<u64> {
        if !self.can_pass(from_room) {
            return None;
        }
        if from_room == self.room_a {
            Some(self.room_b)
        } else if from_room == self.room_b {
            Some(self.room_a)
        } else {
            None
        }
    }

    /// Lock the door.
    pub fn lock(&mut self) {
        self.access = DoorAccess::Locked;
    }

    /// Open the door in both directions.
    pub fn open(&mut self) {
        self.access = DoorAccess::Open;
    }
}

// ── Room Event / History ───────────────────────────────────────────────────

/// An event that occurred in a room.
#[derive(Debug, Clone)]
pub struct RoomEvent {
    pub tick: u64,
    pub agent_id: u64,
    pub kind: String,
    pub detail: String,
}

/// Event log for a room.
#[derive(Debug, Clone)]
pub struct RoomHistory {
    events: Vec<RoomEvent>,
}

impl RoomHistory {
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }

    /// Record an event.
    pub fn record(&mut self, event: RoomEvent) {
        self.events.push(event);
    }

    /// Get all events.
    pub fn events(&self) -> &[RoomEvent] {
        &self.events
    }

    /// Number of recorded events.
    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Filter events by kind.
    pub fn filter_by_kind(&self, kind: &str) -> Vec<&RoomEvent> {
        self.events.iter().filter(|e| e.kind == kind).collect()
    }
}

impl Default for RoomHistory {
    fn default() -> Self {
        Self::new()
    }
}

// ── Room State Snapshot ────────────────────────────────────────────────────

/// A snapshot of a room's agents and environment at a point in time.
#[derive(Debug, Clone)]
pub struct RoomState {
    pub room_id: u64,
    pub agents: Vec<u64>,
    pub environment: HashMap<String, String>,
}

// ── Room ───────────────────────────────────────────────────────────────────

/// A room containing agents and an environment map.
#[derive(Debug)]
pub struct Room {
    pub id: u64,
    pub name: String,
    agents: Vec<u64>,
    environment: HashMap<String, String>,
    history: RoomHistory,
}

impl Room {
    pub fn new(id: u64, name: &str) -> Self {
        Self {
            id,
            name: name.to_string(),
            agents: Vec::new(),
            environment: HashMap::new(),
            history: RoomHistory::new(),
        }
    }

    /// Add an agent to the room. Returns false if already present.
    pub fn add_agent(&mut self, agent_id: u64) -> bool {
        if self.agents.contains(&agent_id) {
            return false;
        }
        self.agents.push(agent_id);
        self.history.record(RoomEvent {
            tick: 0,
            agent_id,
            kind: "enter".to_string(),
            detail: format!("agent {} entered room {}", agent_id, self.id),
        });
        true
    }

    /// Remove an agent from the room. Returns true if the agent was present.
    pub fn remove_agent(&mut self, agent_id: u64) -> bool {
        if let Some(pos) = self.agents.iter().position(|&a| a == agent_id) {
            self.agents.remove(pos);
            self.history.record(RoomEvent {
                tick: 0,
                agent_id,
                kind: "leave".to_string(),
                detail: format!("agent {} left room {}", agent_id, self.id),
            });
            true
        } else {
            false
        }
    }

    /// Get the list of agents in the room.
    pub fn agents(&self) -> &[u64] {
        &self.agents
    }

    /// Set an environment variable.
    pub fn set_env(&mut self, key: &str, value: &str) {
        self.environment.insert(key.to_string(), value.to_string());
    }

    /// Get an environment variable.
    pub fn get_env(&self, key: &str) -> Option<&str> {
        self.environment.get(key).map(|s| s.as_str())
    }

    /// Take a snapshot of the current room state.
    pub fn snapshot(&self) -> RoomState {
        RoomState {
            room_id: self.id,
            agents: self.agents.clone(),
            environment: self.environment.clone(),
        }
    }

    /// Restore room state from a snapshot.
    pub fn restore(&mut self, state: RoomState) {
        self.agents = state.agents;
        self.environment = state.environment;
    }

    /// Access the room's event history.
    pub fn history(&self) -> &RoomHistory {
        &self.history
    }

    pub fn history_mut(&mut self) -> &mut RoomHistory {
        &mut self.history
    }

    /// Number of agents in the room.
    pub fn agent_count(&self) -> usize {
        self.agents.len()
    }
}

// ── Room Builder ───────────────────────────────────────────────────────────

/// Builder for constructing rooms with initial state.
pub struct RoomBuilder {
    id: u64,
    name: String,
    agents: Vec<u64>,
    environment: HashMap<String, String>,
}

impl RoomBuilder {
    pub fn new(id: u64, name: &str) -> Self {
        Self {
            id,
            name: name.to_string(),
            agents: Vec::new(),
            environment: HashMap::new(),
        }
    }

    /// Add an initial agent.
    pub fn agent(mut self, agent_id: u64) -> Self {
        self.agents.push(agent_id);
        self
    }

    /// Set an environment variable.
    pub fn env(mut self, key: &str, value: &str) -> Self {
        self.environment.insert(key.to_string(), value.to_string());
        self
    }

    /// Build the room.
    pub fn build(self) -> Room {
        let mut room = Room::new(self.id, &self.name);
        for aid in self.agents {
            room.add_agent(aid);
        }
        for (k, v) in self.environment {
            room.set_env(&k, &v);
        }
        room
    }
}

// ── Room Coordinator ───────────────────────────────────────────────────────

/// Manages agent transitions between rooms via doors.
pub struct RoomCoordinator {
    rooms: HashMap<u64, Room>,
    doors: Vec<Door>,
}

impl RoomCoordinator {
    pub fn new() -> Self {
        Self {
            rooms: HashMap::new(),
            doors: Vec::new(),
        }
    }

    /// Add a room.
    pub fn add_room(&mut self, room: Room) {
        self.rooms.insert(room.id, room);
    }

    /// Add a door connecting two rooms.
    pub fn add_door(&mut self, door: Door) {
        self.doors.push(door);
    }

    /// Try to move `agent_id` from `from_room` to `to_room`.
    ///
    /// Returns `Ok(())` on success or an error string explaining why not.
    pub fn transfer(&mut self, agent_id: u64, from_room: u64, to_room: u64) -> Result<(), String> {
        // Check source room has the agent
        let src = self.rooms.get(&from_room)
            .ok_or_else(|| format!("room {} does not exist", from_room))?;
        if !src.agents().contains(&agent_id) {
            return Err(format!("agent {} not in room {}", agent_id, from_room));
        }
        // Check destination exists
        if !self.rooms.contains_key(&to_room) {
            return Err(format!("room {} does not exist", to_room));
        }
        // Find a door that allows passage
        let can_pass = self.doors.iter().any(|d| {
            (d.room_a == from_room && d.room_b == to_room || d.room_b == from_room && d.room_a == to_room)
                && d.can_pass(from_room)
        });
        if !can_pass {
            return Err(format!("no open door from room {} to room {}", from_room, to_room));
        }
        // Perform the transfer
        let src = self.rooms.get_mut(&from_room).unwrap();
        src.remove_agent(agent_id);
        let dst = self.rooms.get_mut(&to_room).unwrap();
        dst.add_agent(agent_id);
        Ok(())
    }

    /// Get a reference to a room.
    pub fn room(&self, id: u64) -> Option<&Room> {
        self.rooms.get(&id)
    }

    /// Get a mutable reference to a room.
    pub fn room_mut(&mut self, id: u64) -> Option<&mut Room> {
        self.rooms.get_mut(&id)
    }

    /// Number of rooms.
    pub fn room_count(&self) -> usize {
        self.rooms.len()
    }

    /// Number of doors.
    pub fn door_count(&self) -> usize {
        self.doors.len()
    }
}

impl Default for RoomCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn door_locked_cannot_pass() {
        let d = Door::new(1, 10, 20, DoorAccess::Locked);
        assert!(!d.can_pass(10));
        assert!(!d.can_pass(20));
        assert!(d.destination(10).is_none());
    }

    #[test]
    fn door_open_both_ways() {
        let d = Door::new(1, 10, 20, DoorAccess::Open);
        assert!(d.can_pass(10));
        assert!(d.can_pass(20));
        assert_eq!(d.destination(10), Some(20));
        assert_eq!(d.destination(20), Some(10));
    }

    #[test]
    fn door_one_way() {
        let d = Door::new(1, 10, 20, DoorAccess::OneWay(10, 20));
        assert!(d.can_pass(10));
        assert!(!d.can_pass(20));
        assert_eq!(d.destination(10), Some(20));
        assert_eq!(d.destination(20), None);
    }

    #[test]
    fn door_lock_and_open() {
        let mut d = Door::new(1, 10, 20, DoorAccess::Open);
        d.lock();
        assert_eq!(d.access, DoorAccess::Locked);
        d.open();
        assert_eq!(d.access, DoorAccess::Open);
    }

    #[test]
    fn room_add_remove_agent() {
        let mut room = Room::new(1, "lobby");
        assert!(room.add_agent(42));
        assert!(!room.add_agent(42)); // duplicate
        assert_eq!(room.agent_count(), 1);
        assert!(room.remove_agent(42));
        assert!(!room.remove_agent(42)); // already gone
        assert_eq!(room.agent_count(), 0);
    }

    #[test]
    fn room_environment() {
        let mut room = Room::new(1, "lab");
        room.set_env("temperature", "22");
        assert_eq!(room.get_env("temperature"), Some("22"));
        assert_eq!(room.get_env("humidity"), None);
    }

    #[test]
    fn room_snapshot_restore() {
        let mut room = Room::new(1, "test");
        room.add_agent(1);
        room.add_agent(2);
        room.set_env("light", "on");
        let snap = room.snapshot();
        room.remove_agent(1);
        room.set_env("light", "off");
        room.restore(snap);
        assert_eq!(room.agent_count(), 2);
        assert_eq!(room.get_env("light"), Some("on"));
    }

    #[test]
    fn room_history_records_events() {
        let mut room = Room::new(1, "hub");
        room.add_agent(10);
        room.remove_agent(10);
        let events = room.history().events();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].kind, "enter");
        assert_eq!(events[1].kind, "leave");
    }

    #[test]
    fn room_history_filter() {
        let mut h = RoomHistory::new();
        h.record(RoomEvent { tick: 1, agent_id: 1, kind: "enter".into(), detail: "".into() });
        h.record(RoomEvent { tick: 2, agent_id: 1, kind: "speak".into(), detail: "".into() });
        h.record(RoomEvent { tick: 3, agent_id: 1, kind: "enter".into(), detail: "".into() });
        assert_eq!(h.filter_by_kind("enter").len(), 2);
    }

    #[test]
    fn room_builder() {
        let room = RoomBuilder::new(5, "bridge")
            .agent(1)
            .agent(2)
            .env("alert", "red")
            .build();
        assert_eq!(room.id, 5);
        assert_eq!(room.name, "bridge");
        assert_eq!(room.agent_count(), 2);
        assert_eq!(room.get_env("alert"), Some("red"));
    }

    #[test]
    fn coordinator_transfer_success() {
        let mut coord = RoomCoordinator::new();
        let mut r1 = Room::new(1, "A");
        r1.add_agent(100);
        coord.add_room(r1);
        coord.add_room(Room::new(2, "B"));
        coord.add_door(Door::new(1, 1, 2, DoorAccess::Open));
        assert!(coord.transfer(100, 1, 2).is_ok());
        assert_eq!(coord.room(1).unwrap().agent_count(), 0);
        assert_eq!(coord.room(2).unwrap().agent_count(), 1);
    }

    #[test]
    fn coordinator_transfer_locked_door() {
        let mut coord = RoomCoordinator::new();
        let mut r1 = Room::new(1, "A");
        r1.add_agent(100);
        coord.add_room(r1);
        coord.add_room(Room::new(2, "B"));
        coord.add_door(Door::new(1, 1, 2, DoorAccess::Locked));
        assert!(coord.transfer(100, 1, 2).is_err());
    }

    #[test]
    fn coordinator_transfer_agent_not_present() {
        let mut coord = RoomCoordinator::new();
        coord.add_room(Room::new(1, "A"));
        coord.add_room(Room::new(2, "B"));
        coord.add_door(Door::new(1, 1, 2, DoorAccess::Open));
        assert!(coord.transfer(999, 1, 2).is_err());
    }

    #[test]
    fn coordinator_transfer_room_not_found() {
        let mut coord = RoomCoordinator::new();
        coord.add_room(Room::new(1, "A"));
        assert!(coord.transfer(1, 1, 99).is_err());
    }

    #[test]
    fn coordinator_counts() {
        let mut coord = RoomCoordinator::new();
        coord.add_room(Room::new(1, "A"));
        coord.add_room(Room::new(2, "B"));
        coord.add_door(Door::new(1, 1, 2, DoorAccess::Open));
        assert_eq!(coord.room_count(), 2);
        assert_eq!(coord.door_count(), 1);
    }

    #[test]
    fn coordinator_room_mut() {
        let mut coord = RoomCoordinator::new();
        coord.add_room(Room::new(1, "A"));
        coord.room_mut(1).unwrap().set_env("x", "y");
        assert_eq!(coord.room(1).unwrap().get_env("x"), Some("y"));
    }

    #[test]
    fn history_default_empty() {
        let h = RoomHistory::default();
        assert!(h.is_empty());
    }

    #[test]
    fn room_state_snapshot_fields() {
        let snap = RoomState {
            room_id: 7,
            agents: vec![1, 2, 3],
            environment: {
                let mut m = HashMap::new();
                m.insert("key".into(), "val".into());
                m
            },
        };
        assert_eq!(snap.room_id, 7);
        assert_eq!(snap.agents.len(), 3);
    }

    #[test]
    fn coordinator_default() {
        let c = RoomCoordinator::default();
        assert_eq!(c.room_count(), 0);
    }

    #[test]
    fn door_destination_unknown_room() {
        let d = Door::new(1, 10, 20, DoorAccess::Open);
        assert!(d.destination(99).is_none());
    }

    #[test]
    fn room_agents_slice() {
        let mut room = Room::new(1, "test");
        room.add_agent(5);
        room.add_agent(10);
        assert_eq!(room.agents(), &[5, 10]);
    }

    #[test]
    fn coordinator_one_way_door_transfer() {
        let mut coord = RoomCoordinator::new();
        let mut r1 = Room::new(1, "entry");
        r1.add_agent(50);
        coord.add_room(r1);
        coord.add_room(Room::new(2, "vault"));
        coord.add_door(Door::new(1, 1, 2, DoorAccess::OneWay(1, 2)));
        // Forward pass works
        assert!(coord.transfer(50, 1, 2).is_ok());
        // Reverse does not
        assert!(coord.transfer(50, 2, 1).is_err());
    }
}
