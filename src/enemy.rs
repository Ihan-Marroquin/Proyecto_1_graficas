use raylib::prelude::*;
use std::collections::VecDeque;
use crate::maze::Maze;

pub struct Enemy {
    pub spawn_cell: (usize, usize),
    pub pos: Vector2,
    pub tex_index: usize,
    pub speed: f32,
    pub path: Vec<(usize, usize)>,
    pub path_timer: f32,
    pub attack_cooldown: f32,
}

impl Enemy {
    pub fn new(spawn_cell: (usize, usize), block_size: usize, tex_index: usize, speed: f32) -> Self {
        let cx = (spawn_cell.0 * block_size) as f32 + (block_size as f32) / 2.0;
        let cy = (spawn_cell.1 * block_size) as f32 + (block_size as f32) / 2.0;
        Self {
            spawn_cell,
            pos: Vector2::new(cx, cy),
            tex_index,
            speed,
            path: Vec::new(),
            path_timer: 0.0,
            attack_cooldown: 0.0,
        }
    }

    pub fn reset_to_spawn(&mut self, block_size: usize) {
        let cx = (self.spawn_cell.0 * block_size) as f32 + (block_size as f32) / 2.0;
        let cy = (self.spawn_cell.1 * block_size) as f32 + (block_size as f32) / 2.0;
        self.pos = Vector2::new(cx, cy);
        self.path.clear();
        self.path_timer = 0.0;
    }

    fn current_cell(&self, block_size: usize, maze: &Maze) -> (usize, usize) {
        let cols = maze[0].len();
        let rows = maze.len();
        let cx = ((self.pos.x as usize) / block_size).min(cols.saturating_sub(1));
        let cy = ((self.pos.y as usize) / block_size).min(rows.saturating_sub(1));
        (cx, cy)
    }

    fn bfs_path(maze: &Maze, start: (usize, usize), target: (usize, usize)) -> Vec<(usize, usize)> {
        let rows = maze.len();
        let cols = maze[0].len();
        let mut q = VecDeque::new();
        let mut visited = vec![vec![false; cols]; rows];
        let mut parent = vec![vec![None; cols]; rows];

        q.push_back(start);
        visited[start.1][start.0] = true;

        let dirs = [(1isize,0isize), (-1,0), (0,1), (0,-1)];

        while let Some((cx, cy)) = q.pop_front() {
            if (cx, cy) == target { break; }
            for &(dx, dy) in dirs.iter() {
                let nx = cx as isize + dx;
                let ny = cy as isize + dy;
                if nx >= 0 && ny >= 0 && (nx as usize) < cols && (ny as usize) < rows {
                    let ux = nx as usize;
                    let uy = ny as usize;
                    if !visited[uy][ux] {
                        let ch = maze[uy][ux];
                        if ch == ' ' || ch == 'p' || ch == 'g' {
                            visited[uy][ux] = true;
                            parent[uy][ux] = Some((cx, cy));
                            q.push_back((ux, uy));
                        }
                    }
                }
            }
        }

        let mut path = Vec::new();
        if !visited[target.1][target.0] {
            return path;
        }

        let mut cur = target;
        path.push(cur);
        while cur != start {
            if let Some(prev) = parent[cur.1][cur.0] {
                cur = prev;
                path.push(cur);
            } else { break; }
        }
        path.reverse();
        path
    }

    pub fn update(&mut self, maze: &Maze, block_size: usize, player_pos: &Vector2, dt: f32) -> bool {
        if self.attack_cooldown > 0.0 {
            self.attack_cooldown -= dt;
        }

        self.path_timer -= dt;
        let player_cell = ((player_pos.x as usize) / block_size, (player_pos.y as usize) / block_size);
        let my_cell = self.current_cell(block_size, maze);

        if self.path_timer <= 0.0 || self.path.is_empty() {
            self.path = Enemy::bfs_path(maze, my_cell, player_cell);
            self.path_timer = 1.0; 
        }

        if self.path.len() >= 2 {
            let next = self.path[1];
            let target_x = (next.0 * block_size) as f32 + (block_size as f32) / 2.0;
            let target_y = (next.1 * block_size) as f32 + (block_size as f32) / 2.0;
            let dx = target_x - self.pos.x;
            let dy = target_y - self.pos.y;
            let dist = (dx*dx + dy*dy).sqrt();
            if dist > 1.0 {
                let step = (self.speed * dt).min(dist);
                self.pos.x += dx / dist * step;
                self.pos.y += dy / dist * step;
            } else {
                if !self.path.is_empty() { self.path.remove(0); }
            }
        }

        let dxp = player_pos.x - self.pos.x;
        let dyp = player_pos.y - self.pos.y;
        let distp = (dxp*dxp + dyp*dyp).sqrt();
        let attack_radius = (block_size as f32) * 0.5;
        if distp < attack_radius && self.attack_cooldown <= 0.0 {
            self.attack_cooldown = 1.0;
            self.reset_to_spawn(block_size);
            return true;
        }

        false
    }
}
