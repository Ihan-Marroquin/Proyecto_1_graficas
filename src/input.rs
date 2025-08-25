use raylib::prelude::*;
use std::f32::consts::PI;
use crate::player::Player;
use crate::maze::Maze;


pub fn process_events(
    window: &mut raylib::prelude::RaylibHandle,
    player: &mut Player,
    maze: &mut Maze,
    block_size: usize,
    dt: f32,
    goal_unlocked: &mut bool,
) -> Option<String> {
    const BASE_SPEED: f32 = 80.0; 
    const RUN_MULT: f32 = 1.6;
    const ROT_SPEED: f32 = PI; 

    let mut dt = dt.min(0.05);

    let mut msg: Option<String> = None;

    if window.is_key_down(KeyboardKey::KEY_LEFT) {
        player.a -= ROT_SPEED * dt;
    }
    if window.is_key_down(KeyboardKey::KEY_RIGHT) {
        player.a += ROT_SPEED * dt;
    }

    let mut speed = BASE_SPEED;
    let running = window.is_key_down(KeyboardKey::KEY_LEFT_SHIFT) && player.stamina > 5.0;
    if running {
        speed *= RUN_MULT;
        player.stamina = (player.stamina - 60.0 * dt).max(0.0);
        if player.stamina <= 10.0 {
            speed *= 0.5;
        }
    }

    let mut want_dx = 0.0f32;
    let mut want_dy = 0.0f32;

    if window.is_key_down(KeyboardKey::KEY_UP) {
        want_dx += player.a.cos() * speed * dt;
        want_dy += player.a.sin() * speed * dt;
    }
    if window.is_key_down(KeyboardKey::KEY_DOWN) {
        want_dx -= player.a.cos() * speed * dt;
        want_dy -= player.a.sin() * speed * dt;
    }

    let total_dist = (want_dx*want_dx + want_dy*want_dy).sqrt();
    if total_dist > 0.0 {
        let max_step = (block_size as f32) * 0.25;
        let steps = ((total_dist / max_step).ceil() as usize).max(1);
        let step_dx = want_dx / steps as f32;
        let step_dy = want_dy / steps as f32;

        for _ in 0..steps {
            let new_x = player.pos.x + step_dx;
            let new_y = player.pos.y + step_dy;

            let ci = (new_x as isize / block_size as isize) as isize;
            let cj = (new_y as isize / block_size as isize) as isize;

            if cj < 0 || ci < 0 || (cj as usize) >= maze.len() || (ci as usize) >= maze[0].len() {
                break;
            }

            match maze[cj as usize][ci as usize] {
                ' ' | 'p' => {
                    player.pos.x = new_x;
                    player.pos.y = new_y;
                },
                'g' => {
                    player.pos.x = new_x;
                    player.pos.y = new_y;
                },
                'D' => {
                    if player.has_key {
                        maze[cj as usize][ci as usize] = ' ';
                        player.has_key = false;
                        msg = Some("Usaste la llave para abrir una puerta.".to_string());
                        let neigh = [(1,0),(-1,0),(0,1),(0,-1)];
                        for &(dxg, dyg) in neigh.iter() {
                            let nx = ci + dxg;
                            let ny = cj + dyg;
                            if ny >= 0 && nx >= 0 && (ny as usize) < maze.len() && (nx as usize) < maze[0].len() {
                                if maze[ny as usize][nx as usize] == 'g' { *goal_unlocked = true; }
                            }
                        }
                    } else {
                        break;
                    }
                },
                '+' | '-' | '|' => {
                    break;
                },
                _ => {
                    break;
                }
            }
        } 
    }

    while player.a > std::f32::consts::PI { player.a -= 2.0 * std::f32::consts::PI; }
    while player.a < -std::f32::consts::PI { player.a += 2.0 * std::f32::consts::PI; }

    msg
}
