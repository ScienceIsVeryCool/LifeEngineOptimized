// simulation/src/lib.rs

use rand::random;
mod organism;
pub use organism::{Organism, Direction, OrganismCell};

// Special RNG initialization for WASM targets
// This could be added at the top of organism.rs or lib.rs in the simulation crate

#[cfg(target_arch = "wasm32")]
fn init_random() {
    // In WASM, we need to use the js-sys Random function
    // This is handled by the getrandom crate with the "js" feature
    // which we've added in Cargo.toml
}

#[cfg(not(target_arch = "wasm32"))]
fn init_random() {
    // For non-WASM targets, we don't need to do anything special
}

// Call this function in the Grid::new or some initialization function
pub fn initialize() {
    init_random();
}
/// Different types of cells in the simulation
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CellState {
    Empty,
    Food,
    Wall,
    Mouth,
    Producer,
    Mover,
    Killer,
    Armor,
    Eye,
}

impl CellState {
    /// Convert a cell state to a color representation
    pub fn to_color(&self) -> u32 {
        match self {
            CellState::Empty => 0x0E1318,   // Dark blue
            CellState::Food => 0x2F7AB7,    // Bluish
            CellState::Wall => 0x808080,    // Gray
            CellState::Mouth => 0xDEB14D,   // Orange
            CellState::Producer => 0x15DE59, // Green
            CellState::Mover => 0x60D4FF,   // Light blue
            CellState::Killer => 0xF82380,  // Red
            CellState::Armor => 0x7230DB,   // Purple
            CellState::Eye => 0xB6C1EA,     // Light purple
        }
    }
}

/// Cell in the grid, includes state and owner
#[derive(Clone)]
pub struct Cell {
    pub state: CellState,
    pub owner: Option<usize>, // Index of the owning organism, if any
}

/// The core Grid business logic with no WASM/browser dependencies.
pub struct Grid {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u32>,
    pub cells: Vec<Cell>,
    pub food_production_prob: f32, // Probability of food production
    pub organisms: Vec<Organism>,  // All organisms in the simulation
    pub next_organism_id: usize,   // Next ID to assign to a new organism
    pub max_organisms: usize,      // Maximum number of organisms allowed
    pub lifespan_multiplier: u32,  // Multiplier for organism lifespan
    pub insta_kill: bool,          // Whether organisms die instantly when hit by a killer
}

impl Grid {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            pixels: vec![0; (width * height) as usize],
            cells: vec![Cell { state: CellState::Empty, owner: None }; (width * height) as usize],
            food_production_prob: 0.005, // 0.5% chance by default
            organisms: Vec::new(),
            next_organism_id: 0,
            max_organisms: 1000,       // Default max organisms
            lifespan_multiplier: 100,  // Default lifespan multiplier
            insta_kill: false,         // Default to not insta-kill
        }
    }

    /// Set the color of a specific pixel.
    /// Color is a 24-bit value in the form 0xRRGGBB.
    pub fn set_pixel(&mut self, x: u32, y: u32, color: u32) {
        if x < self.width && y < self.height {
            let idx = (y * self.width + x) as usize;
            self.pixels[idx] = color;
        }
    }
    
    pub fn get_pixel(&self, x: u32, y: u32) -> u32 {
        if x < self.width && y < self.height {
            let idx = (y * self.width + x) as usize;
            self.pixels[idx]
        } else {
            0x000000 // return black for invalid coordinates
        }
    }

    /// Set a cell's state and owner
    pub fn set_cell(&mut self, x: u32, y: u32, state: CellState, owner: Option<usize>) {
        if x < self.width && y < self.height {
            let idx = (y * self.width + x) as usize;
            self.cells[idx] = Cell { state, owner };
            self.pixels[idx] = state.to_color();
        }
    }

    /// Get a reference to a cell at the specified coordinates
    pub fn get_cell(&self, x: u32, y: u32) -> Option<&Cell> {
        if x < self.width && y < self.height {
            let idx = (y * self.width + x) as usize;
            Some(&self.cells[idx])
        } else {
            None
        }
    }
    
    /// Check if a position is clear (empty or food)
    pub fn is_position_clear(&self, x: u32, y: u32) -> bool {
        if let Some(cell) = self.get_cell(x, y) {
            cell.state == CellState::Empty || cell.state == CellState::Food
        } else {
            false
        }
    }
    
    /// Check if a position has food
    pub fn has_food_at(&self, x: u32, y: u32) -> bool {
        if let Some(cell) = self.get_cell(x, y) {
            cell.state == CellState::Food
        } else {
            false
        }
    }
    
    /// Add a new organism to the grid
    pub fn add_organism(&mut self, mut organism: Organism) -> bool {
        if self.organisms.len() >= self.max_organisms && self.max_organisms > 0 {
            return false;
        }
        
        // Update organism's ID if not already set
        if organism.id == 0 {
            organism.id = self.next_organism_id;
            self.next_organism_id += 1;
        }
        
        // Check if all cells can be placed
        let can_place = organism.cells.iter().all(|cell| {
            let (x, y) = organism.get_cell_position(cell);
            x < self.width && y < self.height && self.is_position_clear(x, y)
        });
        
        if can_place {
            // Place all cells
            for cell in &organism.cells {
                let (x, y) = organism.get_cell_position(cell);
                self.set_cell(x, y, cell.state, Some(organism.id));
            }
            
            self.organisms.push(organism);
            true
        } else {
            false
        }
    }
    
    /// Create a new basic organism at a position
    pub fn create_basic_organism(&mut self, x: u32, y: u32) -> bool {
        if x >= self.width || y >= self.height {
            return false;
        }
        
        let mut organism = Organism::new(self.next_organism_id, x, y);
        
        // Add some basic cells
        organism.add_cell(CellState::Mouth, 0, 0); // Center
        organism.add_cell(CellState::Producer, 1, 1); // Up Right
        organism.add_cell(CellState::Producer, -1, -1); // Down Left
        
        self.add_organism(organism)
    }
    
    /// Remove an organism from the grid
    fn remove_organism(&mut self, org_id: usize) {
        if let Some(index) = self.organisms.iter().position(|org| org.id == org_id) {
            // First, collect all the cells to convert to food
            let mut cells_to_food = Vec::new();
            {
                let organism = &self.organisms[index];
                for cell in &organism.cells {
                    let (x, y) = organism.get_cell_position(cell);
                    if x < self.width && y < self.height {
                        cells_to_food.push((x, y));
                    }
                }
            }
            
            // Now turn those cells into food
            for (x, y) in cells_to_food {
                self.set_cell(x, y, CellState::Food, None);
            }
            
            // Remove the organism
            self.organisms.remove(index);
        }
    }
    
    /// Remove dead organisms
    fn remove_dead_organisms(&mut self) {
        let dead_ids: Vec<usize> = self.organisms.iter()
            .filter(|org| !org.is_alive)
            .map(|org| org.id)
            .collect();
            
        for id in dead_ids {
            self.remove_organism(id);
        }
    }

    /// Try to produce food in adjacent empty cells
    fn try_produce_food(&self, x: u32, y: u32, new_cells: &mut [Cell]) {
        // Define adjacent cells (up, down, left, right)
        let adjacent = [(0, -1), (0, 1), (-1, 0), (1, 0)];
        
        for (dx, dy) in adjacent.iter() {
            let nx = x as i32 + dx;
            let ny = y as i32 + dy;
            
            // Check bounds
            if nx >= 0 && ny >= 0 && nx < self.width as i32 && ny < self.height as i32 {
                let nidx = (ny as u32 * self.width + nx as u32) as usize;
                
                // Only produce food in empty cells with some probability
                if self.cells[nidx].state == CellState::Empty && random::<f32>() < 0.1 {
                    new_cells[nidx].state = CellState::Food;
                }
            }
        }
    }
    
    /// Process killer cells damaging other organisms
    fn process_killer_cells(&mut self) {
        // Track which organisms take damage
        let mut damage_map: std::collections::HashMap<usize, u32> = std::collections::HashMap::new();
        
        // Check each organism's killer cells
        for org in &self.organisms {
            if !org.is_alive {
                continue;
            }
            
            for cell in &org.cells {
                if cell.state != CellState::Killer {
                    continue;
                }
                
                let (cx, cy) = org.get_cell_position(cell);
                let adjacents = [(0, 1), (1, 0), (0, -1), (-1, 0)];
                
                for (dx, dy) in adjacents.iter() {
                    let nx = (cx as i32 + dx).max(0).min(self.width as i32 - 1) as u32;
                    let ny = (cy as i32 + dy).max(0).min(self.height as i32 - 1) as u32;
                    
                    if let Some(target_cell) = self.get_cell(nx, ny) {
                        // If cell belongs to another organism and is not armor
                        if let Some(target_id) = target_cell.owner {
                            if target_id != org.id && target_cell.state != CellState::Armor {
                                *damage_map.entry(target_id).or_insert(0) += 1;
                            }
                        }
                    }
                }
            }
        }
        
        // Apply damage to organisms
        for (org_id, damage) in damage_map {
            if let Some(index) = self.organisms.iter().position(|org| org.id == org_id) {
                if self.insta_kill {
                    self.organisms[index].is_alive = false;
                } else {
                    for _ in 0..damage {
                        self.organisms[index].harm();
                    }
                }
            }
        }
    }
    
// Fix for process_reproduction function
// Updated process_reproduction function to avoid borrowing conflict

fn process_reproduction(&mut self) {
    let mut new_organisms = Vec::new();
    let max_organisms = self.max_organisms;
    let current_organism_count = self.organisms.len();
    let width = self.width;
    let height = self.height;
    
    // Store organisms that will attempt reproduction
    let mut reproduction_candidates = Vec::new();
    
    // First get a list of organisms eligible for reproduction
    for i in 0..self.organisms.len() {
        if !self.organisms[i].is_alive {
            continue;
        }
        
        reproduction_candidates.push(i);
    }
    
    // Process reproduction without borrowing self.organisms directly
    for org_idx in reproduction_candidates {
        // Check if we can add more organisms
        if current_organism_count + new_organisms.len() < max_organisms || max_organisms == 0 {
            // Try to reproduce by temporarily borrowing
            if let Some(offspring) = self.organisms[org_idx].try_reproduce() {
                // Check if offspring can be placed
                let can_place = {
                    let mut is_valid = true;
                    for cell in &offspring.cells {
                        let (x, y) = offspring.get_cell_position(cell);
                        if x >= width || y >= height || !self.is_position_clear(x, y) {
                            is_valid = false;
                            break;
                        }
                    }
                    is_valid
                };
                
                if can_place {
                    // Ensure proper ID
                    let mut final_offspring = offspring;
                    final_offspring.id = self.next_organism_id;
                    self.next_organism_id += 1;
                    
                    new_organisms.push(final_offspring);
                }
            }
        }
    }
    
    // Add all new organisms
    for org in new_organisms {
        self.add_organism(org);
    }
}

    
    
// Function with fixed borrowing in process_eating method 
fn process_eating(&mut self) {
    // First collect all eating actions to avoid borrowing conflicts
    let mut food_eaten = Vec::new();
    let mut org_food_collected = Vec::new();
    
    // Collect all eating actions
    for (org_idx, org) in self.organisms.iter().enumerate() {
        if !org.is_alive {
            continue;
        }
        
        for cell in &org.cells {
            if cell.state != CellState::Mouth {
                continue;
            }
            
            let (cx, cy) = org.get_cell_position(cell);
            let adjacents = [(0, 1), (1, 0), (0, -1), (-1, 0)];
            
            for (dx, dy) in adjacents.iter() {
                let nx = (cx as i32 + dx).max(0).min(self.width as i32 - 1) as u32;
                let ny = (cy as i32 + dy).max(0).min(self.height as i32 - 1) as u32;
                
                // If there's food 
                if self.has_food_at(nx, ny) {
                    food_eaten.push((nx, ny));
                    org_food_collected.push(org_idx);
                }
            }
        }
    }
    
    // Apply food collection to organisms
    for org_idx in org_food_collected {
        self.organisms[org_idx].food_collected += 1;
    }
    
    // Remove all eaten food
    for (x, y) in food_eaten {
        self.set_cell(x, y, CellState::Empty, None);
    }
}

// Fixed update_organisms method to resolve borrowing issues
fn update_organisms(&mut self) {
    // Process eating
    self.process_eating();
    
    // Process killer cells
    self.process_killer_cells();
    
    // First clear all organisms from the grid
    {
        let mut cells_to_clear = Vec::new();
        
        for org in &self.organisms {
            if !org.is_alive {
                continue;
            }
            
            for cell in &org.cells {
                let (x, y) = org.get_cell_position(cell);
                if x < self.width && y < self.height {
                    cells_to_clear.push((x, y, org.id));
                }
            }
        }
        
        // Clear cells
        for (x, y, org_id) in cells_to_clear {
            let idx = (y * self.width + x) as usize;
            if self.cells[idx].owner == Some(org_id) {
                self.cells[idx] = Cell { state: CellState::Empty, owner: None };
            }
        }
    }
    
    // Update organisms
    let mut updated_organisms = Vec::new();
    let width = self.width;
    let height = self.height;
    
    for org in &self.organisms {
        if !org.is_alive {
            updated_organisms.push(org.clone());
            continue;
        }
        
        // Clone the organism for the update
        let mut updated_org = org.clone();
        
        // Save the grid state for checking clear positions
        let is_position_clear = |x: u32, y: u32| -> bool {
            if x >= width || y >= height {
                return false;
            }
            let idx = (y * width + x) as usize;
            let cell = &self.cells[idx];
            cell.state == CellState::Empty || cell.state == CellState::Food
        };
        
        let has_food_at = |x: u32, y: u32| -> bool {
            if x >= width || y >= height {
                return false;
            }
            let idx = (y * width + x) as usize;
            let cell = &self.cells[idx];
            cell.state == CellState::Food
        };
        
        // Update the organism with the closures
        updated_org.update(width, height, is_position_clear, has_food_at);
        
        updated_organisms.push(updated_org);
    }
    
    // Replace the old organisms with the updated ones
    self.organisms = updated_organisms;
    
    // Re-place all organisms on the grid
    let mut cells_to_set = Vec::new();
    for org in &self.organisms {
        if !org.is_alive {
            continue;
        }
        
        for cell in &org.cells {
            let (x, y) = org.get_cell_position(cell);
            if x < self.width && y < self.height {
                cells_to_set.push((x, y, cell.state, org.id));
            }
        }
    }
    
    // Now actually set the cells
    for (x, y, state, org_id) in cells_to_set {
        self.set_cell(x, y, state, Some(org_id));
    }
    
    // Process reproduction
    self.process_reproduction();
    
    // Remove dead organisms
    self.remove_dead_organisms();
}

    /// Main step function to update the entire simulation
    pub fn step(&mut self) {
        // Update organisms
        self.update_organisms();
        
        // Randomly produce food in empty cells
        for y in 0..self.height {
            for x in 0..self.width {
                let idx = (y * self.width + x) as usize;
                
                if self.cells[idx].state == CellState::Empty && random::<f32>() < self.food_production_prob {
                    self.set_cell(x, y, CellState::Food, None);
                }
            }
        }
        
        // Process producer cells
        let mut new_food_positions = Vec::new();
        
        for org in &self.organisms {
            if !org.is_alive {
                continue;
            }
            
            for cell in &org.cells {
                if cell.state != CellState::Producer {
                    continue;
                }
                
                let (cx, cy) = org.get_cell_position(cell);
                let adjacents = [(0, 1), (1, 0), (0, -1), (-1, 0)];
                
                for (dx, dy) in adjacents.iter() {
                    let nx = (cx as i32 + dx).max(0).min(self.width as i32 - 1) as u32;
                    let ny = (cy as i32 + dy).max(0).min(self.height as i32 - 1) as u32;
                    
                    if let Some(cell) = self.get_cell(nx, ny) {
                        if cell.state == CellState::Empty && random::<f32>() < 0.1 {
                            new_food_positions.push((nx, ny));
                        }
                    }
                }
            }
        }
        
        // Add new food
        for (x, y) in new_food_positions {
            self.set_cell(x, y, CellState::Food, None);
        }
        
        // Update the pixels based on cell states
        for y in 0..self.height {
            for x in 0..self.width {
                let idx = (y * self.width + x) as usize;
                self.pixels[idx] = self.cells[idx].state.to_color();
            }
        }
    }
    
    /// Create an initial organism (the "origin of life")
    pub fn origin_of_life(&mut self) {
        let x = self.width / 2;
        let y = self.height / 2;
        self.create_basic_organism(x, y);
    }
    
    /// Reset the grid to initial state
    pub fn reset(&mut self, clear_walls: bool) {
        // Clear all cells except walls if specified
        for y in 0..self.height {
            for x in 0..self.width {
                let idx = (y * self.width + x) as usize;
                if clear_walls || self.cells[idx].state != CellState::Wall {
                    self.cells[idx] = Cell { state: CellState::Empty, owner: None };
                }
            }
        }
        
        // Clear all organisms
        self.organisms.clear();
        
        // Reset organism ID counter
        self.next_organism_id = 0;
        
        // Update pixels
        for (idx, cell) in self.cells.iter().enumerate() {
            self.pixels[idx] = cell.state.to_color();
        }
    }
}