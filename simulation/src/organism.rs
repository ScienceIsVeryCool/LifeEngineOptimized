// simulation/src/organism.rs

use rand::Rng;

use crate::CellState;

/// Direction for movement and facing
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Direction {
    Up = 0,
    Right = 1, 
    Down = 2,
    Left = 3,
}

impl Direction {
    /// Get a random direction
    pub fn random() -> Self {
        let dir = rand::thread_rng().gen_range(0..4);
        match dir {
            0 => Direction::Up,
            1 => Direction::Right,
            2 => Direction::Down,
            3 => Direction::Left,
            _ => Direction::Up, // This won't happen due to range
        }
    }
    
    /// Get the opposite direction
    pub fn opposite(&self) -> Self {
        match self {
            Direction::Up => Direction::Down,
            Direction::Right => Direction::Left,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
        }
    }
    
    /// Convert direction to movement delta (dx, dy)
    pub fn to_delta(&self) -> (i32, i32) {
        match self {
            Direction::Up => (0, -1),
            Direction::Right => (1, 0),
            Direction::Down => (0, 1),
            Direction::Left => (-1, 0),
        }
    }
}

/// A cell in an organism, with its state and relative position to the organism center
#[derive(Clone, Debug)]
pub struct OrganismCell {
    pub state: CellState,
    pub x: i32,   // Relative x position from organism center
    pub y: i32,   // Relative y position from organism center
    pub direction: Option<Direction>, // For cells that have direction (like eyes)
}

impl OrganismCell {
    pub fn new(state: CellState, x: i32, y: i32) -> Self {
        OrganismCell {
            state,
            x,
            y,
            direction: if state == CellState::Eye { 
                Some(Direction::random()) 
            } else { 
                None 
            },
        }
    }
    
    /// Get the rotated position of this cell
    pub fn get_rotated_position(&self, rotation: Direction) -> (i32, i32) {
        match rotation {
            Direction::Up => (self.x, self.y),
            Direction::Right => (self.y, -self.x),
            Direction::Down => (-self.x, -self.y),
            Direction::Left => (-self.y, self.x),
        }
    }
    
    /// Get the absolute direction this cell is facing (for directional cells)
    pub fn get_absolute_direction(&self, organism_rotation: Direction) -> Option<Direction> {
        self.direction.map(|dir| {
            // Calculate the absolute direction by adding the organism's rotation
            let absolute_dir = (dir as u8 + organism_rotation as u8) % 4;
            match absolute_dir {
                0 => Direction::Up,
                1 => Direction::Right,
                2 => Direction::Down,
                3 => Direction::Left,
                _ => Direction::Up, // Won't happen due to modulo
            }
        })
    }
}

/// Represents a collection of cells that form a living organism
#[derive(Clone, Debug)]
pub struct Organism {
    pub id: usize,              // Unique identifier
    pub x: u32,                 // Position X
    pub y: u32,                 // Position Y
    pub rotation: Direction,    // Facing direction
    pub move_direction: Direction, // Movement direction
    pub cells: Vec<OrganismCell>, // Cells that make up the organism
    pub food_collected: u32,    // Amount of food collected
    pub health: u32,            // Current health
    pub lifetime: u32,          // How many steps this organism has lived
    pub mutability: u8,         // How likely this organism is to mutate (0-100)
    pub move_range: u32,        // How many steps in one direction before changing
    pub move_counter: u32,      // Counter for current movement
    pub is_alive: bool,         // Whether the organism is alive
}

impl Organism {
    /// Create a new basic organism
    pub fn new(id: usize, x: u32, y: u32) -> Self {
        let mut organism = Organism {
            id,
            x,
            y,
            rotation: Direction::Up,
            move_direction: Direction::random(),
            cells: Vec::new(),
            food_collected: 0,
            health: 0,
            lifetime: 0,
            mutability: 5,  // 5% mutation chance by default
            move_range: 4,  // Move 4 steps before changing direction
            move_counter: 0,
            is_alive: true,
        };
        
        // Add a default mouth cell at the center
        organism.add_cell(CellState::Mouth, 0, 0);
        
        organism
    }
    
    /// Create a new organism from a parent (with possible mutations)
    pub fn new_from_parent(id: usize, x: u32, y: u32, parent: &Organism) -> Self {
        let mut organism = Organism {
            id,
            x,
            y,
            rotation: Direction::random(), // Random rotation for offspring
            move_direction: Direction::random(),
            cells: parent.cells.clone(),
            food_collected: 0,
            health: 0,
            lifetime: 0,
            mutability: parent.mutability,  // Inherit mutability
            move_range: parent.move_range,  // Inherit move range
            move_counter: 0,
            is_alive: true,
        };
        
        // Mutate with probability based on mutability
        let mut rng = rand::thread_rng();
        if rng.gen_range(0..100) < organism.mutability {
            organism.mutate();
            
            // Also sometimes mutate the move_range
            if rng.gen_range(0..100) < 10 {
                organism.move_range = (organism.move_range as i32 + rng.gen_range(-2..3))
                    .max(1) as u32; // Ensure move_range is at least 1
            }
            
            // And sometimes mutate the mutability itself
            if rng.gen_range(0..100) < 10 {
                organism.mutability = (organism.mutability as i32 + rng.gen_range(-1..2))
                    .max(1).min(100) as u8;
            }
        }
        
        organism
    }
    
    /// Add a cell to the organism
    pub fn add_cell(&mut self, state: CellState, x: i32, y: i32) {
        self.cells.push(OrganismCell::new(state, x, y));
        self.health = self.cells.len() as u32; // Health equals number of cells
    }
    
    /// Check if we can add a cell at the specific relative position
    pub fn can_add_cell_at(&self, x: i32, y: i32) -> bool {
        !self.cells.iter().any(|cell| cell.x == x && cell.y == y)
    }
    
    /// Get the absolute position of a cell in the grid
    pub fn get_cell_position(&self, cell: &OrganismCell) -> (u32, u32) {
        let (dx, dy) = cell.get_rotated_position(self.rotation);
        ((self.x as i32 + dx) as u32, (self.y as i32 + dy) as u32)
    }
    
    /// Check if this organism has eyes
    pub fn has_eyes(&self) -> bool {
        self.cells.iter().any(|cell| cell.state == CellState::Eye)
    }
    
    /// Check if this organism has mover cells
    pub fn has_movers(&self) -> bool {
        self.cells.iter().any(|cell| cell.state == CellState::Mover)
    }
    
    /// Check if this organism has producer cells
    pub fn has_producers(&self) -> bool {
        self.cells.iter().any(|cell| cell.state == CellState::Producer)
    }
    
    /// Get the amount of food needed to reproduce
    pub fn food_needed_to_reproduce(&self) -> u32 {
        let base_food = self.cells.len() as u32;
        if self.has_movers() {
            base_food + 1 // Movers need more food to reproduce
        } else {
            base_food
        }
    }
    
    /// Get the maximum lifespan of this organism
    pub fn max_lifespan(&self) -> u32 {
        self.cells.len() as u32 * 100 // 100 ticks per cell
    }
    
    /// Try to reproduce (returns a new organism if successful)
    pub fn try_reproduce(&mut self) -> Option<Organism> {
        if self.food_collected >= self.food_needed_to_reproduce() {
            // Reduce the food collected
            self.food_collected -= self.food_needed_to_reproduce();
            
            // Determine where the offspring should go
            let direction = Direction::random();
            let (dx, dy) = direction.to_delta();
            let birth_distance = 4; // Base distance
            
            let offset_x = dx * (birth_distance + rand::thread_rng().gen_range(1..4));
            let offset_y = dy * (birth_distance + rand::thread_rng().gen_range(1..4));
            
            let new_x = (self.x as i32 + offset_x).max(0) as u32;
            let new_y = (self.y as i32 + offset_y).max(0) as u32;
            
            // Create offspring
            let new_id = self.id + 1; // This is simplistic; in reality needs to be managed globally
            Some(Organism::new_from_parent(new_id, new_x, new_y, self))
        } else {
            None
        }
    }
    
    /// Mutate this organism by adding, changing, or removing a cell
    pub fn mutate(&mut self) {
        let mut rng = rand::thread_rng();
        
        // Equal chance of add/change/remove
        let mutation_type = rng.gen_range(0..3);
        
        match mutation_type {
            0 => {
                // Add a cell
                if self.cells.len() > 0 {
                    // Pick a random existing cell as attachment point
                    let parent_idx = rng.gen_range(0..self.cells.len());
                    let parent = &self.cells[parent_idx];
                    
                    // Try adjacent positions
                    let directions = [(0, 1), (1, 0), (0, -1), (-1, 0)];
                    let mut valid_positions = Vec::new();
                    
                    for (dx, dy) in directions.iter() {
                        let new_x = parent.x + dx;
                        let new_y = parent.y + dy;
                        if self.can_add_cell_at(new_x, new_y) {
                            valid_positions.push((new_x, new_y));
                        }
                    }
                    
                    if !valid_positions.is_empty() {
                        let (new_x, new_y) = valid_positions[rng.gen_range(0..valid_positions.len())];
                        let new_state = random_cell_state();
                        self.add_cell(new_state, new_x, new_y);
                    }
                }
            },
            1 => {
                // Change a cell type
                if self.cells.len() > 1 { // Don't change the center cell
                    let idx = rng.gen_range(1..self.cells.len());
                    self.cells[idx].state = random_cell_state();
                    if self.cells[idx].state == CellState::Eye {
                        self.cells[idx].direction = Some(Direction::random());
                    } else {
                        self.cells[idx].direction = None;
                    }
                }
            },
            2 => {
                // Remove a cell (never remove the center cell)
                if self.cells.len() > 1 {
                    let idx = rng.gen_range(1..self.cells.len());
                    self.cells.remove(idx);
                    self.health = self.cells.len() as u32;
                }
            },
            _ => {}
        }
    }
    
    /// Try to move in the current direction
    pub fn try_move(&mut self, grid_width: u32, grid_height: u32, 
                   is_position_clear: impl Fn(u32, u32) -> bool) -> bool {
        // Only organisms with mover cells can move
        if !self.has_movers() {
            return false;
        }
        
        let (dx, dy) = self.move_direction.to_delta();
        let new_x = (self.x as i32 + dx).max(0).min(grid_width as i32 - 1) as u32;
        let new_y = (self.y as i32 + dy).max(0).min(grid_height as i32 - 1) as u32;
        
        // Check if all cells can move to their new positions
        let can_move = self.cells.iter().all(|cell| {
            let (cell_dx, cell_dy) = cell.get_rotated_position(self.rotation);
            let cell_x = (new_x as i32 + cell_dx).max(0).min(grid_width as i32 - 1) as u32;
            let cell_y = (new_y as i32 + cell_dy).max(0).min(grid_height as i32 - 1) as u32;
            
            // Check if the new position is clear (or belongs to this organism)
            let current_pos = self.get_cell_position(cell);
            (cell_x, cell_y) == current_pos || is_position_clear(cell_x, cell_y)
        });
        
        if can_move {
            self.x = new_x;
            self.y = new_y;
            self.move_counter += 1;
            
            // Change direction after move_range steps
            if self.move_counter >= self.move_range {
                self.move_direction = Direction::random();
                self.move_counter = 0;
            }
            
            true
        } else {
            // If blocked, we might want to change direction
            if rand::thread_rng().gen_bool(0.5) {
                self.move_direction = Direction::random();
                self.move_counter = 0;
            }
            false
        }
    }
    
    /// Try to rotate to a new orientation
    pub fn try_rotate(&mut self, 
                     is_position_clear: impl Fn(u32, u32) -> bool) -> bool {
        let new_rotation = Direction::random();
        
        // Check if all cells can be in their new rotated positions
        let can_rotate = self.cells.iter().all(|cell| {
            let (cell_dx, cell_dy) = cell.get_rotated_position(new_rotation);
            let cell_x = (self.x as i32 + cell_dx).max(0) as u32;
            let cell_y = (self.y as i32 + cell_dy).max(0) as u32;
            
            // Check if the new position is clear
            let current_pos = self.get_cell_position(cell);
            (cell_x, cell_y) == current_pos || is_position_clear(cell_x, cell_y)
        });
        
        if can_rotate {
            self.rotation = new_rotation;
            true
        } else {
            false
        }
    }
    
    /// Reduce health when harmed
    pub fn harm(&mut self) {
        if self.health > 0 {
            self.health -= 1;
        }
        if self.health == 0 {
            self.is_alive = false;
        }
    }
    
    /// Update the organism for one time step
    pub fn update(&mut self, grid_width: u32, grid_height: u32,
                  is_position_clear: impl Fn(u32, u32) -> bool,
                  food_at_position: impl Fn(u32, u32) -> bool) {
        if !self.is_alive {
            return;
        }
        
        self.lifetime += 1;
        
        // Check if organism died of old age
        if self.lifetime >= self.max_lifespan() {
            self.is_alive = false;
            return;
        }
        
        // Try to eat food
        for cell in &self.cells {
            if cell.state == CellState::Mouth {
                // Check adjacent positions for food
                let (cx, cy) = self.get_cell_position(cell);
                let adjacents = [(0, 1), (1, 0), (0, -1), (-1, 0)];
                
                for (dx, dy) in adjacents.iter() {
                    let fx = (cx as i32 + dx).max(0).min(grid_width as i32 - 1) as u32;
                    let fy = (cy as i32 + dy).max(0).min(grid_height as i32 - 1) as u32;
                    
                    if food_at_position(fx, fy) {
                        self.food_collected += 1;
                    }
                }
            }
        }
        
        // Try to move or rotate
        if self.has_movers() {
            let moved = self.try_move(grid_width, grid_height, |x, y| is_position_clear(x, y));
            
            if !moved {
                // If couldn't move, try to rotate
                self.try_rotate(|x, y| is_position_clear(x, y));
            }
        }
    }
}

/// Get a random cell state (excluding Empty, Food, and Wall which are environment states)
fn random_cell_state() -> CellState {
    let state_idx = rand::thread_rng().gen_range(0..6);
    match state_idx {
        0 => CellState::Mouth,
        1 => CellState::Producer,
        2 => CellState::Mover,
        3 => CellState::Killer,
        4 => CellState::Armor,
        5 => CellState::Eye,
        _ => CellState::Mouth, // Won't happen due to range
    }
}