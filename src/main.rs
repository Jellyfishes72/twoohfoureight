#![windows_subsystem = "windows"]

use rand::{Rng, thread_rng};
use raylib::prelude::*;
use std::fmt::{Display, Error, Formatter};

#[derive(Clone, Copy)]
struct Cell {
    value: u32,
    combined: bool,
}

impl Cell {
    fn empty() -> Self {
        Cell {
            value: 0,
            combined: false,
        }
    }

    fn occupied(value: u32) -> Self {
        Cell {
            value,
            combined: false,
        }
    }

    fn is_empty(&self) -> bool {
        self.value == 0
    }

    fn is_occupied(&self) -> bool {
        !self.is_empty()
    }
}

impl PartialEq for Cell {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl Display for Cell {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> { 
        if self.value == 0 {
            write!(f, "Empty")
        } else {
            write!(f, "Occupied(value={}, combined={}", self.value, self.combined)
        }
    }
}

#[derive(PartialEq)]
enum State {
    Playing,
    GameOwover,
    _Victory,
}

#[derive(Copy, Clone)]
struct Particle {
    x: f32,
    y: f32,
    width: u32,
    height: u32,
    vel_x: f32,
    vel_y: f32,
    color: Color,
    life: f32,
}

impl Particle {
    fn rand(x: f32, y: f32, color: Color) -> Self {
        let size = thread_rng().gen_range(5..10);
        Particle {
            x,
            y,
            width: size,
            height: size,
            vel_x: thread_rng().gen_range(-50.0..50.0),
            vel_y: thread_rng().gen_range(-50.0..50.0),
            color,
            life: thread_rng().gen_range(150.0..250.0),
        }
    }

    fn tick(&mut self, dt: f32) {
        self.x += self.vel_x * dt;
        self.y += self.vel_y * dt;
        self.life -= dt * PARTICLE_LIFE_DECAY;
        self.vel_x = decrease_abs(self.vel_x, PARTICLE_FRICTION * dt);
        self.vel_y = decrease_abs(self.vel_y, PARTICLE_FRICTION * dt);
    }

    fn render(&self, d: &mut RaylibDrawHandle) {
        if self.life > 0.0 {
            d.draw_rectangle(
                self.x as i32, 
                self.y as i32, 
                self.width as i32, 
                self.height as i32, 
                Color::new(self.color.r, self.color.g, self.color.b, ((self.life / PARTICLE_LIFE) * 255.0) as u8));
            d.draw_rectangle_lines(
                self.x as i32, 
                self.y as i32, 
                self.width as i32, 
                self.height as i32, 
                Color::new(0xff, 0xff, 0xff, ((self.life / PARTICLE_LIFE) * 255.0) as u8));
        }
    }

    fn is_dead(&self) -> bool {
        self.life < 0.0
    }

    fn is_alive(&self) -> bool {
        !self.is_dead()
    }
}

struct GameState {
    cells: [[Cell; CELL_DIM]; CELL_DIM],
    score: u32,
    state: State,
    particles: Vec<Particle>,
}

impl GameState {
    fn new() -> Self {
        GameState {
            cells: [[Cell::empty(); CELL_DIM]; CELL_DIM],
            score: 0,
            state: State::Playing,
            particles: Vec::new(),
        }
    }

    fn reset(&mut self) {
        self.cells = random_cells();
        self.score = 0;
        self.state = State::Playing;
        self.particles = Vec::new();
    }
}

const BOARD_SIZE: f32 = 400.0;
const CELL_DIM: usize = 4;
const CELL_PAD: f32 = 10.0;
const CELL_SIZE: f32 = (BOARD_SIZE - (CELL_PAD * (CELL_DIM + 1) as f32)) / CELL_DIM as f32;
const CELL_BASE_COLOR: Color = Color::new(0x44, 0x44, 0xff, 0xff);
const MAX_SCORE: u32 = 2048;
const COLORS: u32 = MAX_SCORE.ilog2() + 2;
const WIDTH: i32 = 500;
const HEIGHT: i32 = WIDTH;
const PARTICLE_LIFE: f32 = 200.0;
const PARTICLE_LIFE_DECAY: f32 = 200.0;
const PARTICLE_FRICTION: f32 = 20.0;

fn main() {
    let (mut rl, thread) = raylib::init()
    .size(WIDTH, HEIGHT)
    .title("Hello, World")
        .build();
    
    let board = Rectangle { x: 50.0, y: 50.0, width: BOARD_SIZE, height: BOARD_SIZE };
    // let mut cells = [[Cell::empty(); CELL_DIM]; CELL_DIM];
    // let mut score = 0;
    // let mut state = State::Playing;
    // let mut particles = Vec::new();
    let mut gs = GameState::new();
    gs.reset();

    while !rl.window_should_close() {
        // Reset
        if rl.is_key_pressed(KeyboardKey::KEY_R) {
            gs.reset();
        }

        if gs.state == State::Playing {
            let mut moved = false;
            // Movement
            if rl.is_key_released(KeyboardKey::KEY_RIGHT) {
                moved = true;
                gs.score += slide_right(&mut gs.cells, &mut gs.particles, board);
            } else if rl.is_key_released(KeyboardKey::KEY_LEFT) {
                moved = true;
                gs.score += slide_left(&mut gs.cells, &mut gs.particles, board);
            } else if rl.is_key_released(KeyboardKey::KEY_DOWN) {
                moved = true;
                gs.score += slide_down(&mut gs.cells, &mut gs.particles, board);
            } else if rl.is_key_released(KeyboardKey::KEY_UP) {
                moved = true;
                gs.score += slide_up(&mut gs.cells, &mut gs.particles, board);
            }
    
            for y in 0..gs.cells.len() {
                for x in 0..gs.cells[y].len() {
                    gs.cells[y][x].combined = false;
                }
            }
            
            if moved { 
                let mut has_empty_cell = false;
                for y in 0..gs.cells.len() {
                    for x in 0..gs.cells[y].len() {
                        has_empty_cell |= gs.cells[y][x].is_empty();
                    }
                }
                if has_empty_cell {
                    loop {
                        // TODO: Shouldn't generate a new random cell when sliding doesn't move any cells
                        let x = thread_rng().gen_range(0..CELL_DIM);
                        let y = thread_rng().gen_range(0..CELL_DIM);
                        if gs.cells[y][x].is_empty() {
                           gs.cells[y][x] = Cell::occupied(2_i32.pow(thread_rng().gen::<u32>() % 2 + 1) as u32);
                           break;
                        }
                    }
                } else {
                    gs.state = State::GameOwover;
                }
            }
        }

        for particle in gs.particles.iter_mut() {
            particle.tick(rl.get_frame_time());
        }

        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::from_hex("181818").unwrap());
        let text_size = d.get_font_default().measure_text(format!("{}", gs.score).as_str(), 30.0, 2.0);
        d.draw_text(
            format!("{}", gs.score).as_str(), 
            (WIDTH as f32 / 2.0 - text_size.x / 2.0) as i32,
            (10.0) as i32,
            30, 
            Color::BEIGE
        );
        draw_board(&mut d, gs.cells, board);

        if gs.state == State::GameOwover {
            d.draw_rectangle_rounded(board, 0.05, 10, Color::new(64, 64, 128, 196));
            let text_size = d.get_font_default().measure_text("Game OwOver", 50.0, 2.0);
            d.draw_text(
                "Game OwOver",
                (board.x + board.width / 2.0 - text_size.x / 2.0) as i32, 
                (board.y + board.height / 2.0 - text_size.y / 2.0) as i32, 
                50, 
                Color::BEIGE
            );       
        }
        
        for particle in gs.particles.iter() {
            particle.render(&mut d);
        }
        gs.particles.retain(|p| p.is_alive()); // Remove any particles which have 'died'
    }
}

fn draw_board(d: &mut RaylibDrawHandle, cells: [[Cell; CELL_DIM]; CELL_DIM], board: Rectangle) {
    d.draw_rectangle_rounded_lines(board, 0.05, 10, 2.0, Color::BEIGE);
    for y in 0..cells.len() {
        for x in 0..cells[0].len() {
            let cell_x = board.x + CELL_PAD * (x as f32 + 1.0) + x as f32 * CELL_SIZE;
            let cell_y = board.y + CELL_PAD * (y as f32 + 1.0) + y as f32 * CELL_SIZE;
            let cell = cells[y][x];
            if !cell.is_empty() {
                d.draw_rectangle_rounded(
                    Rectangle { 
                        x: cell_x, 
                        y: cell_y, 
                        width: CELL_SIZE, 
                        height: CELL_SIZE 
                    }, 
                    0.1,
                    10,
                    get_cell_color(cell.value),
                );
            }
            d.draw_rectangle_rounded_lines(
                Rectangle { 
                    x: cell_x, 
                    y: cell_y, 
                    width: CELL_SIZE, 
                    height: CELL_SIZE 
                }, 
                0.1,
                10,
                2.0,
                Color::BEIGE
            );
            if !cell.is_empty() {
                let text_size = d.get_font_default().measure_text(format!("{}", cell.value).as_str(), 30.0, 2.0);
                d.draw_text(
                    format!("{}", cell.value).as_str(), 
                    (cell_x + CELL_SIZE / 2.0 - text_size.x / 2.0) as i32,
                    (cell_y + CELL_SIZE / 2.0 - text_size.y / 2.0) as i32,
                    30, 
                    Color::BEIGE
                );
            }
        }
    }
}

fn get_cell_color(cell_value: u32) -> Color {
    let mut hsv = CELL_BASE_COLOR.color_to_hsv();
    hsv.z = 1.0 - (hsv.z / COLORS as f32 * cell_value.ilog2() as f32);
    Color::color_from_hsv(hsv.x, hsv.y, hsv.z)
}

fn random_cells() -> [[Cell; CELL_DIM]; CELL_DIM] {
    let mut cells = [[Cell::empty(); CELL_DIM]; CELL_DIM];
    let num_cells = thread_rng().gen_range(2..=3);
    let mut generated = 0;
    while generated < num_cells {
        let x = thread_rng().gen_range(0..CELL_DIM);
        let y = thread_rng().gen_range(0..CELL_DIM);
        if cells[y][x].is_empty() {
            let cell_value = 2_i32.pow(thread_rng().gen::<u32>() % 2 + 1) as u32;
            cells[y][x] = Cell::occupied(cell_value);
            generated += 1;
        }
    }
    cells
}

fn slide_right(cells: &mut [[Cell; CELL_DIM]; CELL_DIM], particles: &mut Vec<Particle>, board: Rectangle) -> u32 {
    let mut score = 0;
    for y in 0..cells.len() {
        for x in (0..(cells[0].len() - 1)).rev() {
            if cells[y][x].is_empty() {
                continue;
            }
            let mut cell_x = x;
            while cell_x < cells[0].len() - 1 {
                if cells[y][cell_x + 1].is_occupied() {
                    if cells[y][cell_x + 1] == cells[y][x] && !cells[y][cell_x + 1].combined && !cells[y][x].combined {
                        score += cells[y][x].value * 2;
                        cells[y][cell_x + 1] = Cell { value: cells[y][x].value * 2, combined: true};
                        let cell_px_x = board.x + CELL_PAD * ((cell_x + 1) as f32 + 1.0) + (cell_x + 1) as f32 * CELL_SIZE;
                        let cell_px_y = board.y + CELL_PAD * (y as f32 + 1.0) + y as f32 * CELL_SIZE;
                        particles.append(&mut generate_particles(cell_px_x + CELL_SIZE / 2.0, cell_px_y + CELL_SIZE / 2.0, get_cell_color(cells[y][x].value), 20));
                        cells[y][x] = Cell::empty();
                    }
                    break;
                } 
                cell_x += 1;
            }
            if cell_x != x {
                cells[y][cell_x] = cells[y][x];
                cells[y][x] = Cell::empty();
            }
        }
    }
    score
}

fn slide_left(cells: &mut [[Cell; CELL_DIM]; CELL_DIM], particles: &mut Vec<Particle>, board: Rectangle) -> u32 {
    let mut score = 0;
    for y in 0..cells.len() {
        for x in 1..cells[0].len() {
            if cells[y][x].is_empty() {
                continue;
            }
            let mut cell_x = x;
            while cell_x > 0 {
                if cells[y][cell_x - 1].is_occupied() {
                    if cells[y][cell_x - 1] == cells[y][x] && !cells[y][cell_x - 1].combined && !cells[y][x].combined {
                        score += cells[y][x].value * 2;
                        cells[y][cell_x - 1] = Cell { value: cells[y][x].value * 2, combined: true};
                        let cell_px_x = board.x + CELL_PAD * ((cell_x - 1) as f32 + 1.0) + (cell_x - 1) as f32 * CELL_SIZE;
                        let cell_px_y = board.y + CELL_PAD * (y as f32 + 1.0) + y as f32 * CELL_SIZE;
                        particles.append(&mut generate_particles(cell_px_x + CELL_SIZE / 2.0, cell_px_y + CELL_SIZE / 2.0, get_cell_color(cells[y][x].value), 20));
                        cells[y][x] = Cell::empty();
                    }
                    break;
                }
                cell_x -= 1;
            }
            if cell_x != x {
                cells[y][cell_x] = cells[y][x];
                cells[y][x] = Cell::empty();
            }
        }
    }
    score
}

fn slide_down(cells: &mut [[Cell; CELL_DIM]; CELL_DIM], particles: &mut Vec<Particle>, board: Rectangle) -> u32 {
    let mut score = 0;
    for y in (0..(cells.len() - 1)).rev() {
        for x in 0..cells[0].len() {
            if cells[y][x].is_empty() {
                continue;
            }
            let mut cell_y = y;
            while cell_y < cells.len() - 1 {
                if cells[cell_y + 1][x].is_occupied() {
                    if cells[cell_y + 1][x] == cells[y][x] && !cells[cell_y + 1][x].combined && !cells[y][x].combined {
                        score += cells[y][x].value * 2;
                        cells[cell_y + 1][x] = Cell { value: cells[y][x].value * 2, combined: true};
                        let cell_px_x = board.x + CELL_PAD * (x as f32 + 1.0) + x as f32 * CELL_SIZE;
                        let cell_px_y = board.y + CELL_PAD * ((cell_y + 1) as f32 + 1.0) + (cell_y + 1) as f32 * CELL_SIZE;
                        particles.append(&mut generate_particles(cell_px_x + CELL_SIZE / 2.0, cell_px_y + CELL_SIZE / 2.0, get_cell_color(cells[y][x].value), 20));
                        cells[y][x] = Cell::empty();
                    }
                    break;
                }
                cell_y += 1;
            }
            if cell_y != y {
                cells[cell_y][x] = cells[y][x];
                cells[y][x] = Cell::empty();
            }
        }
    }
    score
}

fn slide_up(cells: &mut [[Cell; CELL_DIM]; CELL_DIM], particles: &mut Vec<Particle>, board: Rectangle) -> u32 {
    let mut score = 0;
    for y in 1..cells.len() {
        for x in 0..cells[0].len() {
            if cells[y][x].is_empty() {
                continue;
            }
            let mut cell_y = y;
            while cell_y > 0 {
                if cells[cell_y - 1][x].is_occupied() {
                    if cells[cell_y - 1][x] == cells[y][x] && !cells[cell_y - 1][x].combined && !cells[y][x].combined {
                        score += cells[y][x].value * 2;
                        cells[cell_y - 1][x] = Cell { value: cells[y][x].value * 2, combined: true};
                        let cell_px_x = board.x + CELL_PAD * (x as f32 + 1.0) + x as f32 * CELL_SIZE;
                        let cell_px_y = board.y + CELL_PAD * ((cell_y - 1) as f32 + 1.0) + (cell_y - 1) as f32 * CELL_SIZE;
                        particles.append(&mut generate_particles(cell_px_x + CELL_SIZE / 2.0, cell_px_y + CELL_SIZE / 2.0, get_cell_color(cells[y][x].value), 20));
                        cells[y][x] = Cell::empty();
                    }
                    break;
                }
                cell_y -= 1;
            }
            if cell_y != y {
                cells[cell_y][x] = cells[y][x];
                cells[y][x] = Cell::empty();
            }
        }
    }
    score
}

fn decrease_abs(mut x: f32, amount: f32) -> f32 {
    if x > 0.0 {
        x -= amount;
        if x < 0.0 {
            return 0.0;
        }
        return x;
    } else if x < 0.0 {
        x += amount;
        if x > 0.0 {
            return 0.0;
        }
        return x;
    }
    0.0
}

fn generate_particles(x: f32, y: f32, color: Color, count: u32) -> Vec<Particle> {
    let mut particles = Vec::new();
    for _ in 0..count {
        particles.push(Particle::rand(x, y, color));
    }
    particles
}