// simulation/src/organism.rs

use rand::Rng;
use rand::seq::SliceRandom; // Add this import
use rand::random;
use crate::CellStates;

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
    pub state: CellStates,
    pub x: i32,   // Relative x position from organism center
    pub y: i32,   // Relative y position from organism center
    pub direction: Option<Direction>, // For cells that have direction (like eyes)
}

impl OrganismCell {
    pub fn new(state: CellStates, x: i32, y: i32) -> Self {
        OrganismCell {
            state,
            x,
            y,
            direction: if state == CellStates::Eye { 
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
        organism.add_cell(CellStates::Mouth, 0, 0);
        
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
    pub fn add_cell(&mut self, state: CellStates, x: i32, y: i32) {
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
        self.cells.iter().any(|cell| cell.state == CellStates::Eye)
    }
    
    /// Check if this organism has mover cells
    pub fn has_movers(&self) -> bool {
        self.cells.iter().any(|cell| cell.state == CellStates::Mover)
    }
    
    /// Check if this organism has producer cells
    pub fn has_producers(&self) -> bool {
        self.cells.iter().any(|cell| cell.state == CellStates::Producer)
    }
    
    /// Get the amount of food needed to reproduce
    pub fn food_needed_to_reproduce(&self) -> u32 {
        if self.has_movers() {
            // In JS: this.anatomy.cells.length + Hyperparams.extraMoverFoodCost
            self.cells.len() as u32 + 1
        } else {
            self.cells.len() as u32
        }
    }
    
    /// Get the maximum lifespan of this organism
    pub fn max_lifespan(&self, lifespan_multiplier: u32) -> u32 {
        (self.cells.len() as u32 * lifespan_multiplier).max(1)
    }
    
    /// Try to reproduce (returns a new organism if successful)
    pub fn try_reproduce(&mut self) -> Option<Organism> {
        if self.food_collected >= self.food_needed_to_reproduce() {
            // Reduce the food collected
            self.food_collected -= self.food_needed_to_reproduce();
            
            // Try more directions including diagonals with more sophisticated positioning
            let directions = [
                (0, -1),   // Up
                (1, 0),    // Right
                (0, 1),    // Down
                (-1, 0),   // Left
                (1, -1),   // Up-Right
                (1, 1),    // Down-Right
                (-1, 1),   // Down-Left
                (-1, -1)   // Up-Left
            ];
            
            let mut rng = rand::thread_rng();
            
            // Randomize direction order manually
            let mut randomized_directions = directions.to_vec();
            for i in 0..randomized_directions.len() {
                let swap_idx = rng.gen_range(0..randomized_directions.len());
                randomized_directions.swap(i, swap_idx);
            }
            
            // Try each direction to find a suitable spot
            for &(dx, dy) in &randomized_directions {
                let birth_distance = self.calculate_birth_distance();
                
                // More sophisticated distance calculation with randomness
                let rand_offset = rng.gen_range(0..3) as i32; // Random offset 0-2
                let offset_x = dx * (birth_distance + rand_offset);
                let offset_y = dy * (birth_distance + rand_offset);
                
                let new_x = (self.x as i32 + offset_x).max(0) as u32;
                let new_y = (self.y as i32 + offset_y).max(0) as u32;
                
                // Create offspring at this position
                let mut offspring = Organism::new_from_parent(0, new_x, new_y, self);
                
                // Optionally adjust offspring rotation based on parent's movement
                if rng.gen_bool(0.5) {
                    // Option 1: Inherit parent's current rotation
                    offspring.rotation = self.rotation;
                } else if rng.gen_bool(0.5) {
                    // Option 2: Rotate towards the birth direction
                    offspring.rotation = match (dx, dy) {
                        (0, -1) => Direction::Up,
                        (1, 0) => Direction::Right,
                        (0, 1) => Direction::Down,
                        (-1, 0) => Direction::Left,
                        (1, -1) => Direction::Up,   // Bias towards Up for diagonal
                        (1, 1) => Direction::Down,  // Bias towards Down for diagonal
                        (-1, 1) => Direction::Down, // Bias towards Down for diagonal
                        (-1, -1) => Direction::Up,  // Bias towards Up for diagonal
                        _ => Direction::random(),
                    };
                } else {
                    // Option 3: Completely random rotation
                    offspring.rotation = Direction::random();
                }
                
                // Return the offspring - position checking will be done at grid level
                return Some(offspring);
            }
            
            // If we get here, all directions failed
            None
        } else {
            None
        }
    }

fn calculate_birth_distance(&self) -> i32 {
    // Find the maximum extent of the organism in any direction
    let mut max_extent = 0;
    for cell in &self.cells {
        let extent = cell.x.abs().max(cell.y.abs());
        max_extent = max_extent.max(extent);
    }
    
    // Birth distance needs to be at least the max extent plus a buffer
    (max_extent + 3) as i32
}
    
    /// Mutate this organism by adding, changing, or removing a cell
    pub fn mutate(&mut self) -> bool {
        let mut changed = false;
        
        // Get probabilities from settings
        let add_prob = 33; // This should be configurable
        let change_prob = 33; // This should be configurable
        let remove_prob = 33; // This should be configurable
        
        // Try to add a cell
        if random::<f32>() * 100.0 < add_prob as f32 {
            // ... existing code for adding cells
            changed = true;
        }
        
        // Try to change a cell type
        if random::<f32>() * 100.0 < change_prob as f32 {
            if self.cells.len() > 1 { // Protect the center cell
                let idx = (random::<f32>() * (self.cells.len() - 1) as f32) as usize + 1;
                // Make sure we get a cell different from the current one
                let mut new_state = random_cell_state();
                while new_state == self.cells[idx].state {
                    new_state = random_cell_state();
                }
                self.cells[idx].state = new_state;
                changed = true;
            }
        }
        
        // Try to remove a cell
        if random::<f32>() * 100.0 < remove_prob as f32 {
            if self.cells.len() > 1 { // Don't remove the last cell
                let idx = (random::<f32>() * (self.cells.len() - 1) as f32) as usize + 1;
                // Don't remove center cell
                if self.cells[idx].x != 0 || self.cells[idx].y != 0 {
                    self.cells.remove(idx);
                    changed = true;
                }
            }
        }
        
        return changed;
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
                  food_at_position: impl Fn(u32, u32) -> bool,
                  lifespan_multiplier: u32) {
        if !self.is_alive {
            return;
        }
        
        self.lifetime += 1;
        
        // Check if organism died of old age
        if self.lifetime >= self.max_lifespan(lifespan_multiplier) {
            self.is_alive = false;
            return;
        }
        
        // Try to eat food
        for cell in &self.cells {
            if cell.state == CellStates::Mouth {
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
fn random_cell_state() -> CellStates {
    let state_idx = rand::thread_rng().gen_range(0..6);
    match state_idx {
        0 => CellStates::Mouth,
        1 => CellStates::Producer,
        2 => CellStates::Mover,
        3 => CellStates::Killer,
        4 => CellStates::Armor,
        5 => CellStates::Eye,
        _ => CellStates::Mouth, // Won't happen due to range
    }
}