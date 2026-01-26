use wasm_bindgen::prelude::*;
use std::cell::RefCell;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CellState { Dead, Alive }

#[derive(Clone, Copy)]
pub struct Position { pub x: i32, pub y: i32 }

#[derive(Clone)]
pub struct Cell { pub pos: Position, pub state: CellState, pub neighbors: u8 }

pub struct GridConfig { pub width: u32, pub height: u32, pub wrap_edges: bool }

pub struct Grid { pub width: u32, pub height: u32, pub cells: Vec<Cell>, pub generation: u64, pub wrap_edges: bool }

fn next_state(cell: &Cell) -> CellState {
  match cell.state {
    CellState::Alive => match cell.neighbors { 0..=1 | 4..=8 => CellState::Dead, _ => CellState::Alive },
    CellState::Dead => if cell.neighbors == 3 { CellState::Alive } else { CellState::Dead },
  }
}

fn create_grid(config: &GridConfig) -> Grid {
  let cells = (0..config.height).flat_map(|y| (0..config.width).map(move |x|
    Cell { pos: Position { x: x as i32, y: y as i32 }, state: CellState::Dead, neighbors: 0 }
  )).collect();
  Grid { width: config.width, height: config.height, cells, generation: 0, wrap_edges: config.wrap_edges }
}

fn wrap(v: i32, max: u32) -> i32 { ((v % max as i32) + max as i32) % max as i32 }

fn to_index(grid: &Grid, x: i32, y: i32) -> Option<usize> {
  let (ax, ay) = if grid.wrap_edges { (wrap(x, grid.width), wrap(y, grid.height)) } else { (x, y) };
  if ax < 0 || ay < 0 || ax >= grid.width as i32 || ay >= grid.height as i32 { None }
  else { Some((ay as u32 * grid.width + ax as u32) as usize) }
}

fn count_neighbors(grid: &Grid, x: i32, y: i32) -> u8 {
  [(-1,-1),(0,-1),(1,-1),(-1,0),(1,0),(-1,1),(0,1),(1,1)].iter()
    .filter_map(|(dx, dy)| to_index(grid, x + dx, y + dy))
    .filter(|&i| grid.cells[i].state == CellState::Alive).count() as u8
}

fn tick_grid(grid: &mut Grid) {
  let neighbors: Vec<u8> = grid.cells.iter().map(|c| count_neighbors(grid, c.pos.x, c.pos.y)).collect();
  for (i, cell) in grid.cells.iter_mut().enumerate() { cell.neighbors = neighbors[i]; cell.state = next_state(cell); }
  grid.generation += 1;
}

fn get_pattern_cells(name: &str) -> Vec<(i32, i32)> {
  match name {
    "block" => vec![(0,0),(1,0),(0,1),(1,1)],
    "blinker" => vec![(0,1),(1,1),(2,1)],
    "glider" => vec![(1,0),(2,1),(0,2),(1,2),(2,2)],
    "toad" => vec![(1,0),(2,0),(3,0),(0,1),(1,1),(2,1)],
    "beacon" => vec![(0,0),(1,0),(0,1),(3,2),(2,3),(3,3)],
    "lwss" => vec![(1,0),(4,0),(0,1),(0,2),(4,2),(0,3),(1,3),(2,3),(3,3)],
    "gun" => vec![(0,4),(0,5),(1,4),(1,5),(10,4),(10,5),(10,6),(11,3),(11,7),(12,2),(12,8),(13,2),(13,8),(14,5),(15,3),(15,7),(16,4),(16,5),(16,6),(17,5),(20,2),(20,3),(20,4),(21,2),(21,3),(21,4),(22,1),(22,5),(24,0),(24,1),(24,5),(24,6),(34,2),(34,3),(35,2),(35,3)],
    "r-pentomino" => vec![(1,0),(2,0),(0,1),(1,1),(1,2)],
    "acorn" => vec![(1,0),(3,1),(0,2),(1,2),(4,2),(5,2),(6,2)],
    _ => vec![],
  }
}

thread_local! {
  static GRID: RefCell<Option<Grid>> = RefCell::new(None);
  static CONFIG: RefCell<GridConfig> = RefCell::new(GridConfig { width: 100, height: 100, wrap_edges: true });
}

#[wasm_bindgen] pub fn init(width: u32, height: u32, wrap: bool) {
  CONFIG.with(|c| *c.borrow_mut() = GridConfig { width, height, wrap_edges: wrap });
  GRID.with(|g| CONFIG.with(|c| *g.borrow_mut() = Some(create_grid(&c.borrow()))));
}

#[wasm_bindgen] pub fn reset() { GRID.with(|g| if let Some(ref mut grid) = *g.borrow_mut() { for c in &mut grid.cells { c.state = CellState::Dead; c.neighbors = 0; } grid.generation = 0; }); }

#[wasm_bindgen] pub fn step() -> u64 { GRID.with(|g| if let Some(ref mut grid) = *g.borrow_mut() { tick_grid(grid); grid.generation } else { 0 }) }

#[wasm_bindgen] pub fn set_cell_state(x: i32, y: i32, alive: bool) { GRID.with(|g| if let Some(ref mut grid) = *g.borrow_mut() { if let Some(i) = to_index(grid, x, y) { grid.cells[i].state = if alive { CellState::Alive } else { CellState::Dead }; } }); }

#[wasm_bindgen] pub fn toggle_cell(x: i32, y: i32) { GRID.with(|g| if let Some(ref mut grid) = *g.borrow_mut() { if let Some(i) = to_index(grid, x, y) { grid.cells[i].state = match grid.cells[i].state { CellState::Alive => CellState::Dead, CellState::Dead => CellState::Alive }; } }); }

#[wasm_bindgen] pub fn load_pattern(name: &str, ox: i32, oy: i32) -> bool {
  let cells = get_pattern_cells(name);
  if cells.is_empty() { return false; }
  GRID.with(|g| if let Some(ref mut grid) = *g.borrow_mut() { for (dx, dy) in cells { if let Some(i) = to_index(grid, ox + dx, oy + dy) { grid.cells[i].state = CellState::Alive; } } });
  true
}

#[wasm_bindgen] pub fn randomize_grid(density: f64) {
  GRID.with(|g| if let Some(ref mut grid) = *g.borrow_mut() {
    let mut seed = grid.generation.wrapping_mul(12345).wrapping_add(67890);
    for c in &mut grid.cells { seed = seed.wrapping_mul(1103515245).wrapping_add(12345) % 2147483648; c.state = if (seed as f64 / 2147483648.0) < density { CellState::Alive } else { CellState::Dead }; }
    grid.generation = 0;
  });
}

#[wasm_bindgen] pub fn get_generation() -> u64 { GRID.with(|g| g.borrow().as_ref().map(|grid| grid.generation).unwrap_or(0)) }
#[wasm_bindgen] pub fn get_width() -> u32 { CONFIG.with(|c| c.borrow().width) }
#[wasm_bindgen] pub fn get_height() -> u32 { CONFIG.with(|c| c.borrow().height) }
#[wasm_bindgen] pub fn get_alive_count() -> u64 { GRID.with(|g| g.borrow().as_ref().map(|grid| grid.cells.iter().filter(|c| c.state == CellState::Alive).count() as u64).unwrap_or(0)) }
#[wasm_bindgen] pub fn get_cells() -> Vec<u8> { GRID.with(|g| g.borrow().as_ref().map(|grid| grid.cells.iter().map(|c| if c.state == CellState::Alive { 1 } else { 0 }).collect()).unwrap_or_default()) }
