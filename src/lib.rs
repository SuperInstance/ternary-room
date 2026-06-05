#![forbid(unsafe_code)]
//! Recursive room-tensor architecture.
//!
//! Every program is a room. Every room is a cell in the tensor.
//! Rooms contain rooms. Tiles are projections. Connections are alive.
//!
//! The recursion IS the architecture:
//! Dance floor → DJ board → instrument panel → signal path → code → metal → bits
//! Same shape at every scale.

use std::collections::HashMap;

// ============================================================
// Connection — the living link between two tiles
// ============================================================

/// A connection between two rooms. Not static — rises and falls.
#[derive(Debug, Clone)]
pub struct Connection {
    pub from: RoomId,
    pub to: RoomId,
    pub time_weight: f64,      // How long they've been connected (grows over time)
    pub distance: f64,         // Physical/logical distance (0 = same spot, 1 = far)
    pub familiarity: f64,      // How well they know each other (0 = stranger, 1 = known)
    pub attraction: f64,       // Affinity signal (0 = repelled, 1 = drawn)
    pub rhythm_sync: f64,      // Phase coherence (0 = off-beat, 1 = locked)
    pub strength: f64,         // Computed overall strength
    pub trend: f64,            // Positive = growing, negative = fading
    pub ticks_alive: u64,
}

impl Connection {
    pub fn new(from: RoomId, to: RoomId) -> Self {
        Self {
            from, to,
            time_weight: 0.0, distance: 0.5, familiarity: 0.0,
            attraction: 0.5, rhythm_sync: 0.5, strength: 0.0,
            trend: 0.0, ticks_alive: 0,
        }
    }

    /// Compute connection strength from all factors.
    /// Strength is emergent, not just a sum.
    pub fn compute_strength(&mut self) -> f64 {
        let time_factor = 1.0 - (-self.time_weight * 0.1).exp(); // Saturating growth
        let dist_factor = 1.0 - self.distance;
        let base = time_factor * dist_factor * self.familiarity * self.attraction;
        let sync_bonus = self.rhythm_sync * 0.3; // Rhythm adds extra on top
        self.strength = (base + sync_bonus).clamp(0.0, 1.0);
        self.strength
    }

    /// Tick the connection — age it, drift the trend.
    pub fn tick(&mut self) {
        self.ticks_alive += 1;
        self.time_weight += 0.01;
        // Trend drifts based on current strength
        self.trend = self.strength - 0.5; // Growing if strong, fading if weak
    }

    /// What kind of connection is this?
    pub fn flavor(&self) -> ConnectionFlavor {
        if self.attraction > 0.7 && self.familiarity < 0.3 {
            ConnectionFlavor::Electric   // Just met, drawn together — volatile
        } else if self.familiarity > 0.7 && self.rhythm_sync > 0.7 {
            ConnectionFlavor::Deep       // Known, synced — reliable
        } else if self.distance > 0.7 && self.rhythm_sync > 0.8 {
            ConnectionFlavor::Resonant   // Far apart but in phase — mysterious
        } else if self.strength < 0.2 {
            ConnectionFlavor::Fading     // Losing connection
        } else {
            ConnectionFlavor::Steady     // Normal, unremarkable (most connections)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionFlavor {
    Electric,  // Just met, high attraction — sparks
    Deep,      // Long-known, synced — foundation
    Resonant,  // Far but in phase — mysterious
    Fading,    // Losing touch
    Steady,    // Normal background connection
}

// ============================================================
// Tile — how one room appears in another room's view
// ============================================================

/// A tile is a room's projection into another room's perspective.
/// You don't see the full dancer — you see a tile of them.
#[derive(Debug, Clone)]
pub struct Tile {
    pub room_id: RoomId,
    pub brightness: f64,       // How prominent (distance-based)
    pub warmth: f64,           // Familiarity tint
    pub pulse_phase: f64,      // Rhythm sync visualization
    pub color: TileColor,      // Overall impression
    pub last_updated: u64,
}

#[derive(Debug, Clone, Copy)]
pub struct TileColor { pub r: f64, pub g: f64, pub b: f64 }

impl TileColor {
    pub fn new(r: f64, g: f64, b: f64) -> Self { Self { r: r.clamp(0.0, 1.0), g: g.clamp(0.0, 1.0), b: b.clamp(0.0, 1.0) } }
    pub fn warm() -> Self { Self::new(1.0, 0.6, 0.2) }    // Familiar
    pub fn cool() -> Self { Self::new(0.2, 0.5, 1.0) }    // Stranger
    pub fn hot() -> Self { Self::new(1.0, 0.2, 0.3) }     // Attracted
    pub fn dim() -> Self { Self::new(0.3, 0.3, 0.3) }     // Far away
    pub fn bright() -> Self { Self::new(0.9, 0.9, 1.0) }  // Close, synced
}

impl Tile {
    pub fn new(room_id: RoomId) -> Self {
        Self { room_id, brightness: 0.5, warmth: 0.5, pulse_phase: 0.0, color: TileColor::cool(), last_updated: 0 }
    }

    /// Update tile appearance from connection state.
    pub fn update_from_connection(&mut self, conn: &Connection, tick: u64) {
        self.brightness = 1.0 - conn.distance;
        self.warmth = conn.familiarity;
        self.pulse_phase = conn.rhythm_sync * std::f64::consts::TAU;

        self.color = if conn.attraction > 0.7 { TileColor::hot() }
            else if conn.familiarity > 0.7 { TileColor::warm() }
            else if conn.distance > 0.7 { TileColor::dim() }
            else if conn.rhythm_sync > 0.7 { TileColor::bright() }
            else { TileColor::cool() };

        self.last_updated = tick;
    }
}

// ============================================================
// Room — a perspective that contains a world
// ============================================================

pub type RoomId = usize;

/// Room depth — what layer of the recursion we're at.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RoomDepth {
    Floor,       // Dance floor — dancers
    Board,       // DJ control board — instruments
    Panel,       // Instrument panel — settings/presets
    Path,        // Signal path — effects/filters
    Code,        // Code — functions
    Metal,       // Metal — transistors/registers
}

impl RoomDepth {
    pub fn deeper(&self) -> Option<RoomDepth> {
        match self {
            RoomDepth::Floor => Some(RoomDepth::Board),
            RoomDepth::Board => Some(RoomDepth::Panel),
            RoomDepth::Panel => Some(RoomDepth::Path),
            RoomDepth::Path => Some(RoomDepth::Code),
            RoomDepth::Code => Some(RoomDepth::Metal),
            RoomDepth::Metal => None,
        }
    }
}

/// A room in the recursive tensor.
/// Contains tiles (projections of other rooms), connections (living links),
/// children (sub-rooms at the next depth), and its own state.
#[derive(Debug, Clone)]
pub struct Room {
    pub id: RoomId,
    pub depth: RoomDepth,
    pub state: i8,                              // Ternary state: -1, 0, +1
    pub tiles: HashMap<RoomId, Tile>,            // How other rooms appear here
    pub connections: HashMap<RoomId, Connection>, // Living connections to other rooms
    pub children: Vec<RoomId>,                   // Sub-rooms contained within
    pub position: (usize, usize),                // Grid position (x, y)
    pub tick: u64,
}

impl Room {
    pub fn new(id: RoomId, depth: RoomDepth) -> Self {
        Self {
            id, depth, state: 0,
            tiles: HashMap::new(), connections: HashMap::new(),
            children: Vec::new(), position: (0, 0), tick: 0,
        }
    }

    /// Place room at grid position.
    pub fn at(mut self, x: usize, y: usize) -> Self { self.position = (x, y); self }

    /// Connect to another room.
    pub fn connect_to(&mut self, other: RoomId) {
        let mut conn = Connection::new(self.id, other);
        conn.compute_strength();
        self.connections.insert(other, conn);
        self.tiles.insert(other, Tile::new(other));
    }

    /// Connect with specific initial parameters.
    pub fn connect_with(&mut self, other: RoomId, distance: f64, familiarity: f64, attraction: f64, rhythm_sync: f64) {
        let mut conn = Connection::new(self.id, other);
        conn.distance = distance;
        conn.familiarity = familiarity;
        conn.attraction = attraction;
        conn.rhythm_sync = rhythm_sync;
        conn.compute_strength();
        self.connections.insert(other, conn);
        self.tiles.insert(other, Tile::new(other));
    }

    /// Add a child room (go deeper).
    pub fn add_child(&mut self, child_id: RoomId) { self.children.push(child_id); }

    /// Tick the room — update all connections and tiles.
    pub fn tick(&mut self) {
        self.tick += 1;
        for conn in self.connections.values_mut() {
            conn.tick();
            conn.compute_strength();
        }
        for (other_id, tile) in self.tiles.iter_mut() {
            if let Some(conn) = self.connections.get(other_id) {
                tile.update_from_connection(conn, self.tick);
            }
        }
    }

    /// Get connections sorted by strength (strongest first).
    pub fn strongest_connections(&self) -> Vec<&Connection> {
        let mut conns: Vec<_> = self.connections.values().collect();
        conns.sort_by(|a, b| b.strength.partial_cmp(&a.strength).unwrap());
        conns
    }

    /// Get connections by flavor.
    pub fn connections_by_flavor(&self, flavor: ConnectionFlavor) -> Vec<&Connection> {
        self.connections.values().filter(|c| c.flavor() == flavor).collect()
    }

    /// Average connection strength.
    pub fn avg_strength(&self) -> f64 {
        if self.connections.is_empty() { return 0.0; }
        self.connections.values().map(|c| c.strength).sum::<f64>() / self.connections.len() as f64
    }

    /// Number of active (non-fading) connections.
    pub fn active_connections(&self) -> usize {
        self.connections.values().filter(|c| c.strength > 0.2).count()
    }

    /// Tile view — what this room "sees."
    pub fn tile_view(&self) -> Vec<&Tile> {
        let mut tiles: Vec<_> = self.tiles.values().collect();
        tiles.sort_by(|a, b| b.brightness.partial_cmp(&a.brightness).unwrap());
        tiles
    }
}

// ============================================================
// Tensor — the grid of all rooms at one depth
// ============================================================

/// A tensor layer — all rooms at the same depth, arranged in a grid.
pub struct TensorLayer {
    pub depth: RoomDepth,
    pub rooms: HashMap<RoomId, Room>,
    pub width: usize,
    pub height: usize,
}

impl TensorLayer {
    pub fn new(depth: RoomDepth, width: usize, height: usize) -> Self {
        Self { depth, rooms: HashMap::new(), width, height }
    }

    /// Add a room to the tensor at grid position.
    pub fn add_room(&mut self, room: Room) {
        self.rooms.insert(room.id, room);
    }

    /// Get room at grid position.
    pub fn room_at(&self, x: usize, y: usize) -> Option<&Room> {
        self.rooms.values().find(|r| r.position == (x, y))
    }

    /// Get room at grid position (mutable).
    pub fn room_at_mut(&mut self, x: usize, y: usize) -> Option<&mut Room> {
        self.rooms.values_mut().find(|r| r.position == (x, y))
    }

    /// Tick all rooms.
    pub fn tick(&mut self) {
        for room in self.rooms.values_mut() { room.tick(); }
    }

    /// Get neighbors of a room (adjacent in grid).
    pub fn neighbors(&self, room_id: RoomId) -> Vec<RoomId> {
        if let Some(room) = self.rooms.get(&room_id) {
            let (x, y) = room.position;
            let mut neighbors = Vec::new();
            for (dx, dy) in &[(-1i32, 0i32), (1, 0), (0, -1), (0, 1), (-1, -1), (-1, 1), (1, -1), (1, 1)] {
                let nx = (x as i32 + dx) as usize;
                let ny = (y as i32 + dy) as usize;
                if nx < self.width && ny < self.height {
                    if let Some(n) = self.room_at(nx, ny) { neighbors.push(n.id); }
                }
            }
            neighbors
        } else { vec![] }
    }

    /// Column view — all rooms at a given x position.
    pub fn column(&self, x: usize) -> Vec<&Room> {
        self.rooms.values().filter(|r| r.position.0 == x).collect()
    }

    /// Row view — all rooms at a given y position.
    pub fn row(&self, y: usize) -> Vec<&Room> {
        self.rooms.values().filter(|r| r.position.1 == y).collect()
    }

    /// Diagonal view — rooms where x == y (or offset).
    pub fn diagonal(&self, offset: i32) -> Vec<&Room> {
        self.rooms.values().filter(|r| r.position.0 as i32 - r.position.1 as i32 == offset).collect()
    }

    /// Total active connections in the layer.
    pub fn total_active_connections(&self) -> usize {
        self.rooms.values().map(|r| r.active_connections()).sum()
    }

    /// Layer health — average room avg_strength.
    pub fn health(&self) -> f64 {
        if self.rooms.is_empty() { return 0.0; }
        self.rooms.values().map(|r| r.avg_strength()).sum::<f64>() / self.rooms.len() as f64
    }
}

// ============================================================
// Recursive Tensor — the full stack of layers
// ============================================================

/// The full recursive tensor — layers stacked from Floor to Metal.
pub struct RecursiveTensor {
    pub layers: HashMap<RoomDepth, TensorLayer>,
    pub cross_layer_connections: Vec<CrossLayerLink>,
    pub tick: u64,
}

/// A connection between rooms at different depths.
/// The dancer IS connected to the instrument that produces their music.
#[derive(Debug, Clone)]
pub struct CrossLayerLink {
    pub upper_room: (RoomDepth, RoomId),
    pub lower_room: (RoomDepth, RoomId),
    pub link_type: CrossLinkType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrossLinkType {
    Contains,    // Upper room contains lower room (DJ board contains synth)
    Projects,    // Lower room projects into upper room (transistor produces the sound)
    Controls,    // Upper room controls lower room (DJ controls the synth)
    Realizes,    // Lower room realizes upper room's intent (code executes the algorithm)
}

impl RecursiveTensor {
    pub fn new() -> Self {
        Self { layers: HashMap::new(), cross_layer_connections: Vec::new(), tick: 0 }
    }

    /// Add a layer at a given depth.
    pub fn add_layer(&mut self, layer: TensorLayer) {
        self.layers.insert(layer.depth, layer);
    }

    /// Link rooms across layers.
    pub fn link_across(&mut self, link: CrossLayerLink) {
        self.cross_layer_connections.push(link);
    }

    /// Tick all layers.
    pub fn tick(&mut self) {
        self.tick += 1;
        for layer in self.layers.values_mut() { layer.tick(); }
    }

    /// Get the full vertical slice at position (x, y) across all layers.
    pub fn vertical_slice(&self, x: usize, y: usize) -> Vec<(RoomDepth, &Room)> {
        let mut slice = Vec::new();
        for (depth, layer) in &self.layers {
            if let Some(room) = layer.room_at(x, y) {
                slice.push((*depth, room));
            }
        }
        slice
    }

    /// Get all cross-layer links for a room.
    pub fn links_for_room(&self, depth: RoomDepth, room_id: RoomId) -> Vec<&CrossLayerLink> {
        self.cross_layer_connections.iter().filter(|l| {
            (l.upper_room.0 == depth && l.upper_room.1 == room_id) ||
            (l.lower_room.0 == depth && l.lower_room.1 == room_id)
        }).collect()
    }

    /// Total rooms across all layers.
    pub fn total_rooms(&self) -> usize {
        self.layers.values().map(|l| l.rooms.len()).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Connection tests
    #[test] fn test_connection_new() { let c = Connection::new(0, 1); assert_eq!(c.from, 0); assert_eq!(c.strength, 0.0); }
    #[test] fn test_connection_strength() { let mut c = Connection::new(0, 1); c.distance = 0.0; c.familiarity = 1.0; c.attraction = 1.0; c.rhythm_sync = 1.0; let s = c.compute_strength(); assert!(s > 0.1, "strength={}", s); }
    #[test] fn test_connection_strength_far() { let mut c = Connection::new(0, 1); c.distance = 1.0; c.familiarity = 0.5; c.attraction = 0.5; let s = c.compute_strength(); assert!(s < 0.3, "strength={}", s); }
    #[test] fn test_connection_tick() { let mut c = Connection::new(0, 1); c.tick(); assert_eq!(c.ticks_alive, 1); assert!(c.time_weight > 0.0); }
    #[test] fn test_connection_flavor_electric() { let c = Connection { attraction: 0.9, familiarity: 0.1, ..Connection::new(0,1) }; assert_eq!(c.flavor(), ConnectionFlavor::Electric); }
    #[test] fn test_connection_flavor_deep() { let c = Connection { familiarity: 0.9, rhythm_sync: 0.8, ..Connection::new(0,1) }; assert_eq!(c.flavor(), ConnectionFlavor::Deep); }
    #[test] fn test_connection_flavor_resonant() { let c = Connection { distance: 0.8, rhythm_sync: 0.9, familiarity: 0.5, attraction: 0.5, ..Connection::new(0,1) }; assert_eq!(c.flavor(), ConnectionFlavor::Resonant); }
    #[test] fn test_connection_flavor_fading() { let mut c = Connection::new(0,1); c.distance = 0.9; c.familiarity = 0.1; c.attraction = 0.1; c.compute_strength(); assert_eq!(c.flavor(), ConnectionFlavor::Fading); }

    // Tile tests
    #[test] fn test_tile_new() { let t = Tile::new(1); assert_eq!(t.room_id, 1); }
    #[test] fn test_tile_update() { let mut t = Tile::new(1); let c = Connection { distance: 0.0, familiarity: 1.0, attraction: 0.5, rhythm_sync: 0.8, ..Connection::new(0,1) }; t.update_from_connection(&c, 1); assert!(t.brightness > 0.9); }
    #[test] fn test_tile_color_warm() { assert!(TileColor::warm().r > 0.9); }
    #[test] fn test_tile_color_hot() { assert!(TileColor::hot().r > 0.9); }

    // Room tests
    #[test] fn test_room_new() { let r = Room::new(0, RoomDepth::Floor); assert_eq!(r.state, 0); }
    #[test] fn test_room_connect() { let mut r = Room::new(0, RoomDepth::Floor); r.connect_to(1); assert!(r.connections.contains_key(&1)); assert!(r.tiles.contains_key(&1)); }
    #[test] fn test_room_connect_with() { let mut r = Room::new(0, RoomDepth::Floor); r.connect_with(1, 0.1, 0.9, 0.8, 0.7); assert!(r.connections[&1].strength > 0.0, "str={}", r.connections[&1].strength); }
    #[test] fn test_room_add_child() { let mut r = Room::new(0, RoomDepth::Floor); r.add_child(10); assert!(r.children.contains(&10)); }
    #[test] fn test_room_tick() { let mut r = Room::new(0, RoomDepth::Floor); r.connect_to(1); r.tick(); assert_eq!(r.tick, 1); }
    #[test] fn test_room_strongest() { let mut r = Room::new(0, RoomDepth::Floor); r.connect_with(1, 0.1, 0.9, 0.9, 0.9); r.connect_with(2, 0.9, 0.1, 0.1, 0.1); let s = r.strongest_connections(); assert_eq!(s[0].to, 1); }
    #[test] fn test_room_avg_strength() { let mut r = Room::new(0, RoomDepth::Floor); r.connect_with(1, 0.1, 0.9, 0.9, 0.9); assert!(r.avg_strength() > 0.0); }
    #[test] fn test_room_active() { let mut r = Room::new(0, RoomDepth::Floor); r.connect_with(1, 0.9, 0.1, 0.1, 0.1); r.connect_with(2, 0.1, 0.9, 0.9, 0.9); assert!(r.active_connections() >= 1); }
    #[test] fn test_room_tile_view() { let mut r = Room::new(0, RoomDepth::Floor); r.connect_with(1, 0.1, 0.9, 0.9, 0.9); r.connect_with(2, 0.9, 0.1, 0.1, 0.1); r.tick(); let view = r.tile_view(); assert_eq!(view.len(), 2); assert!(view[0].brightness >= view[1].brightness); }
    #[test] fn test_room_at_position() { let r = Room::new(0, RoomDepth::Floor).at(3, 4); assert_eq!(r.position, (3, 4)); }

    // TensorLayer tests
    #[test] fn test_layer_new() { let l = TensorLayer::new(RoomDepth::Floor, 4, 4); assert_eq!(l.width, 4); }
    #[test] fn test_layer_add_room() { let mut l = TensorLayer::new(RoomDepth::Floor, 4, 4); l.add_room(Room::new(0, RoomDepth::Floor).at(0, 0)); assert!(l.rooms.contains_key(&0)); }
    #[test] fn test_layer_room_at() { let mut l = TensorLayer::new(RoomDepth::Floor, 4, 4); l.add_room(Room::new(0, RoomDepth::Floor).at(2, 3)); assert!(l.room_at(2, 3).is_some()); assert!(l.room_at(0, 0).is_none()); }
    #[test] fn test_layer_neighbors() { let mut l = TensorLayer::new(RoomDepth::Floor, 4, 4); l.add_room(Room::new(0, RoomDepth::Floor).at(1, 1)); l.add_room(Room::new(1, RoomDepth::Floor).at(0, 0)); l.add_room(Room::new(2, RoomDepth::Floor).at(2, 2)); let n = l.neighbors(0); assert!(!n.is_empty()); }
    #[test] fn test_layer_column() { let mut l = TensorLayer::new(RoomDepth::Floor, 4, 4); l.add_room(Room::new(0, RoomDepth::Floor).at(1, 0)); l.add_room(Room::new(1, RoomDepth::Floor).at(1, 2)); l.add_room(Room::new(2, RoomDepth::Floor).at(2, 1)); let col = l.column(1); assert_eq!(col.len(), 2); }
    #[test] fn test_layer_row() { let mut l = TensorLayer::new(RoomDepth::Floor, 4, 4); l.add_room(Room::new(0, RoomDepth::Floor).at(0, 1)); l.add_room(Room::new(1, RoomDepth::Floor).at(2, 1)); let row = l.row(1); assert_eq!(row.len(), 2); }
    #[test] fn test_layer_diagonal() { let mut l = TensorLayer::new(RoomDepth::Floor, 4, 4); l.add_room(Room::new(0, RoomDepth::Floor).at(0, 0)); l.add_room(Room::new(1, RoomDepth::Floor).at(1, 1)); l.add_room(Room::new(2, RoomDepth::Floor).at(0, 1)); let diag = l.diagonal(0); assert_eq!(diag.len(), 2); }
    #[test] fn test_layer_tick() { let mut l = TensorLayer::new(RoomDepth::Floor, 4, 4); let mut r = Room::new(0, RoomDepth::Floor); r.connect_to(1); l.add_room(r); l.tick(); }
    #[test] fn test_layer_health() { let mut l = TensorLayer::new(RoomDepth::Floor, 4, 4); let mut r = Room::new(0, RoomDepth::Floor); r.connect_with(1, 0.1, 0.9, 0.9, 0.9); l.add_room(r); assert!(l.health() > 0.0); }
    #[test] fn test_layer_total_active() { let mut l = TensorLayer::new(RoomDepth::Floor, 4, 4); let mut r = Room::new(0, RoomDepth::Floor); r.connect_with(1, 0.1, 0.9, 0.9, 0.9); l.add_room(r); assert!(l.total_active_connections() >= 1); }

    // RecursiveTensor tests
    #[test] fn test_recursive_new() { let t = RecursiveTensor::new(); assert!(t.layers.is_empty()); }
    #[test] fn test_recursive_add_layer() { let mut t = RecursiveTensor::new(); t.add_layer(TensorLayer::new(RoomDepth::Floor, 4, 4)); assert!(t.layers.contains_key(&RoomDepth::Floor)); }
    #[test] fn test_recursive_link_across() { let mut t = RecursiveTensor::new(); t.link_across(CrossLayerLink { upper_room: (RoomDepth::Floor, 0), lower_room: (RoomDepth::Board, 1), link_type: CrossLinkType::Contains }); assert_eq!(t.cross_layer_connections.len(), 1); }
    #[test] fn test_recursive_tick() { let mut t = RecursiveTensor::new(); t.add_layer(TensorLayer::new(RoomDepth::Floor, 4, 4)); t.tick(); assert_eq!(t.tick, 1); }
    #[test] fn test_recursive_vertical_slice() { let mut t = RecursiveTensor::new(); let mut fl = TensorLayer::new(RoomDepth::Floor, 4, 4); fl.add_room(Room::new(0, RoomDepth::Floor).at(1, 1)); t.add_layer(fl); let mut bl = TensorLayer::new(RoomDepth::Board, 4, 4); bl.add_room(Room::new(10, RoomDepth::Board).at(1, 1)); t.add_layer(bl); let slice = t.vertical_slice(1, 1); assert_eq!(slice.len(), 2); }
    #[test] fn test_recursive_links_for_room() { let mut t = RecursiveTensor::new(); t.link_across(CrossLayerLink { upper_room: (RoomDepth::Floor, 0), lower_room: (RoomDepth::Board, 1), link_type: CrossLinkType::Contains }); let links = t.links_for_room(RoomDepth::Floor, 0); assert_eq!(links.len(), 1); }
    #[test] fn test_recursive_total_rooms() { let mut t = RecursiveTensor::new(); let mut fl = TensorLayer::new(RoomDepth::Floor, 2, 2); fl.add_room(Room::new(0, RoomDepth::Floor)); fl.add_room(Room::new(1, RoomDepth::Floor)); t.add_layer(fl); assert_eq!(t.total_rooms(), 2); }

    // Depth recursion
    #[test] fn test_depth_deeper() { assert_eq!(RoomDepth::Floor.deeper(), Some(RoomDepth::Board)); }
    #[test] fn test_depth_metal_bottom() { assert_eq!(RoomDepth::Metal.deeper(), None); }
}
