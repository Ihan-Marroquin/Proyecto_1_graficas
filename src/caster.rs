use raylib::color::Color;
use crate::framebuffer::Framebuffer;
use crate::player::Player;
use crate::maze::Maze;

pub struct Intersect {
    pub distance: f32,
    pub impact: char,
    pub hit_x: f32, 
    pub hit_y: f32,
    pub cell_i: usize,
    pub cell_j: usize,
}

pub fn cast_ray(
    _framebuffer: &mut Framebuffer,
    maze: &Maze,
    player: &Player,
    angle: f32,
    block_size: usize,
    _draw_line: bool,
) -> Intersect {
    let px = player.pos.x;
    let py = player.pos.y;
    let map_x_f = px / (block_size as f32);
    let map_y_f = py / (block_size as f32);

    let ray_dir_x = angle.cos();
    let ray_dir_y = angle.sin();

    let mut map_x = map_x_f.floor() as i32;
    let mut map_y = map_y_f.floor() as i32;

    let mut delta_dist_x = if ray_dir_x.abs() < 1e-6 { f32::INFINITY } else { (1.0 / ray_dir_x.abs()) };
    let mut delta_dist_y = if ray_dir_y.abs() < 1e-6 { f32::INFINITY } else { (1.0 / ray_dir_y.abs()) };

    let mut step_x = 0i32;
    let mut step_y = 0i32;
    let mut side_dist_x = 0.0f32;
    let mut side_dist_y = 0.0f32;

    if ray_dir_x < 0.0 {
        step_x = -1;
        side_dist_x = (map_x_f - map_x_f.floor()) * delta_dist_x;
    } else {
        step_x = 1;
        side_dist_x = (1.0 - (map_x_f - map_x_f.floor())) * delta_dist_x;
    }
    if ray_dir_y < 0.0 {
        step_y = -1;
        side_dist_y = (map_y_f - map_y_f.floor()) * delta_dist_y;
    } else {
        step_y = 1;
        side_dist_y = (1.0 - (map_y_f - map_y_f.floor())) * delta_dist_y;
    }

    if delta_dist_x.is_infinite() { side_dist_x = f32::INFINITY; }
    if delta_dist_y.is_infinite() { side_dist_y = f32::INFINITY; }

    let mut hit = false;
    let mut side = 0; 
    let max_iter = 2000usize;
    let mut iter = 0usize;

    while !hit && iter < max_iter {
        if side_dist_x < side_dist_y {
            side_dist_x += delta_dist_x;
            map_x += step_x;
            side = 0;
        } else {
            side_dist_y += delta_dist_y;
            map_y += step_y;
            side = 1;
        }

        if map_y < 0 || map_x < 0 || (map_y as usize) >= maze.len() || (map_x as usize) >= maze[0].len() {
            hit = true;
            break;
        }

        let ch = maze[map_y as usize][map_x as usize];
        if ch != ' ' {
            hit = true;
            break;
        }

        iter += 1;
    }

    let perp_dist = if hit {
        if side == 0 {
            let offset = (map_x as f32 - map_x_f + ((1 - step_x) as f32) / 2.0);
            if ray_dir_x.abs() < 1e-6 { (offset).abs() * block_size as f32 } else { offset.abs() / ray_dir_x.abs() * (block_size as f32) }
        } else {
            let offset = (map_y as f32 - map_y_f + ((1 - step_y) as f32) / 2.0);
            if ray_dir_y.abs() < 1e-6 { (offset).abs() * block_size as f32 } else { offset.abs() / ray_dir_y.abs() * (block_size as f32) }
        }
    } else {
        (maze.len() + maze[0].len()) as f32 * block_size as f32
    };

    let dist_world = perp_dist;
    let hit_x = px + ray_dir_x * (dist_world);
    let hit_y = py + ray_dir_y * (dist_world);

    let ci = if map_x < 0 { 0 } else { map_x as usize }.min(maze[0].len().saturating_sub(1));
    let cj = if map_y < 0 { 0 } else { map_y as usize }.min(maze.len().saturating_sub(1));
    let impact_char = if cj < maze.len() && ci < maze[0].len() { maze[cj][ci] } else { '+' };

    Intersect {
        distance: dist_world.max(0.0),
        impact: impact_char,
        hit_x,
        hit_y,
        cell_i: ci,
        cell_j: cj,
    }
}
