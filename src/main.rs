use raylib::prelude::*;
use raylib::consts::KeyboardKey;
use std::time::Instant;
use std::f32::consts::PI;

mod framebuffer;
mod line;
mod maze;
mod player;
mod caster;
mod generator;
mod input;
mod enemy;
mod textures;
mod audio;

use framebuffer::Framebuffer;
use maze::Maze;
use player::Player;
use caster::cast_ray;
use input::process_events;
use generator::generate_maze_text;
use enemy::Enemy;
use textures::TextureManager;
use audio::AudioManager;

struct Medkit { cell: (usize, usize), taken: bool }
struct PickupKey { cell: (usize, usize), taken: bool }
struct PickupBinocular { cell: (usize, usize), taken: bool }

enum AppState {
    Menu { selected: usize },
    SoundMenu { volume: f32, previous_selected: usize },
    Playing,
    Victory,
    GameOver,
    Exiting,
}

fn draw_menu(window: &mut RaylibHandle, raylib_thread: &RaylibThread, selected: usize) {
    let mut d = window.begin_drawing(raylib_thread);
    d.clear_background(Color::RAYWHITE);
    d.draw_text("Proyecto 1 - Mansion embrujada - Escapa de los Mimikyus -Ihan Marroquin", 60, 40, 44, Color::DARKGRAY);
    let options = ["Empezar a jugar", "Sonido", "Salir"];
    let mut y = 160;
    for (i,&opt) in options.iter().enumerate() {
        let color = if i==selected { Color::RED } else { Color::BLACK };
        d.draw_text(opt, 120, y, 30, color);
        y += 60;
    }
    d.draw_text("Usa ARRIBA/ABAJO para navegar, ENTER seleccionar", 60, 420, 20, Color::DARKGRAY);
}

fn draw_sound_menu(window: &mut RaylibHandle, raylib_thread: &RaylibThread, volume: f32) {
    let mut d = window.begin_drawing(raylib_thread);
    d.clear_background(Color::RAYWHITE);
    d.draw_text("Ajustes de Sonido", 60, 40, 44, Color::DARKGRAY);
    d.draw_text("Volumen:", 120, 180, 28, Color::BLACK);

    let x = 120;
    let y = 220;
    let w = 600;
    let h = 24;
    d.draw_rectangle(x, y, w, h, Color::LIGHTGRAY);
    let fill = ((volume.clamp(0.0,1.0) * w as f32).round() as i32).max(0);
    d.draw_rectangle(x, y, fill, h, Color::GREEN);
    d.draw_rectangle_lines(x, y, w, h, Color::DARKGRAY);

    let pct_text = format!("{:.0}%", (volume.clamp(0.0,1.0) * 100.0));
    d.draw_text(&pct_text, x + w + 16, y, 22, Color::DARKGRAY);

    d.draw_text("Izquierda/Derecha para ajustar - ENTER para volver", 120, 280, 20, Color::DARKGRAY);
}

fn draw_victory(window: &mut RaylibHandle, raylib_thread: &RaylibThread, win_tex: Option<&Texture2D>) {
    let mut d = window.begin_drawing(raylib_thread);
    d.clear_background(Color::RAYWHITE);
    if let Some(tex) = win_tex {
        let screen_w = d.get_screen_width() as f32;
        let screen_h = d.get_screen_height() as f32;

        let tw = tex.width() as f32;
        let th = tex.height() as f32;

        let scale_x = (screen_w / tw).max(0.0);
        let scale_y = (screen_h / th).max(0.0);

        d.draw_texture_ex(tex, Vector2::new(0.0, 0.0), 0.0, scale_x.max(scale_y), Color::WHITE);
    } else {
        d.draw_text("GANASTE! Escapaste de la casa embrujada", 80, 200, 36, Color::GREEN);
        d.draw_text("Presiona ENTER para volver al menu", 80, 300, 22, Color::DARKGRAY);
    }
}

fn draw_gameover(window: &mut RaylibHandle, raylib_thread: &RaylibThread, game_over_tex: Option<&Texture2D>) {
    let mut d = window.begin_drawing(raylib_thread);
    d.clear_background(Color::RAYWHITE);

    if let Some(tex) = game_over_tex {
        let screen_w = d.get_screen_width() as f32;
        let screen_h = d.get_screen_height() as f32;

        let tw = tex.width() as f32;
        let th = tex.height() as f32;

        let scale_x = (screen_w / tw).max(0.0);
        let scale_y = (screen_h / th).max(0.0);

        d.draw_texture_ex(tex, Vector2::new(0.0, 0.0), 0.0, scale_x.max(scale_y), Color::WHITE);

    } else {
        d.draw_text("GAME OVER", 220, 200, 64, Color::RED);
        d.draw_text("Has muerto. Presiona ENTER para volver al menu.", 110, 300, 22, Color::DARKGRAY);
    }
}


fn expand_maze(maze: &Maze, factor: usize) -> Maze {
    let rows = maze.len();
    let cols = maze[0].len();
    let mut out: Maze = Vec::with_capacity(rows * factor);
    for r in 0..rows {
        let mut new_rows: Vec<Vec<char>> = vec![Vec::with_capacity(cols * factor); factor];
        for c in 0..cols {
            let ch = maze[r][c];
            for fr in 0..factor {
                for _fc in 0..factor {
                    new_rows[fr].push(ch);
                }
            }
        }
        for nr in new_rows.into_iter() { out.push(nr); }
    }
    out
}

fn render_world_textured(
    framebuffer: &mut Framebuffer,
    maze: &Maze,
    player: &Player,
    block_size: usize,
    render_scale: usize,
    texmgr: &TextureManager,
) -> Vec<f32> {
    let screen_w_px = framebuffer.width() as usize;
    let screen_h = framebuffer.height() as usize;
    let hh = framebuffer.height() as f32 / 2.0;
    let num_cols = (screen_w_px / render_scale).max(1);
    let proj_plane_dist = (num_cols as f32 / 2.0) / (player.fov / 2.0).tan();
    let mut wall_distances = vec![f32::INFINITY; num_cols];

    let sky_top = Color::new(40, 10, 60, 255);
    let sky_bottom = Color::new(140, 50, 160, 255);
    let floor_col = Color::new(90, 45, 20, 255);

    for y in 0..(screen_h/2) {
        let t = y as f32 / (screen_h as f32 / 2.0);
        let r = ((1.0 - t) * (sky_top.r as f32) + t * (sky_bottom.r as f32)) as u8;
        let g = ((1.0 - t) * (sky_top.g as f32) + t * (sky_bottom.g as f32)) as u8;
        let b = ((1.0 - t) * (sky_top.b as f32) + t * (sky_bottom.b as f32)) as u8;
        framebuffer.set_current_color(Color::new(r,g,b,255));
        for x in 0..screen_w_px {
            if (x as u32) < framebuffer.width() { framebuffer.set_pixel(x as u32, y as u32); }
        }
    }
    for y in (screen_h/2)..screen_h {
        framebuffer.set_current_color(floor_col);
        for x in 0..screen_w_px {
            if (x as u32) < framebuffer.width() { framebuffer.set_pixel(x as u32, y as u32); }
        }
    }

    for col in 0..num_cols {
        let current = if num_cols == 1 { 0.5 } else { col as f32 / (num_cols - 1) as f32 };
        let angle = player.a - (player.fov/2.0) + (player.fov * current);
        let inter = cast_ray(framebuffer, maze, player, angle, block_size, false);
        let delta = angle - player.a;
        let corrected_dist = inter.distance * delta.cos().abs().max(1e-6);
        wall_distances[col] = corrected_dist;

        if corrected_dist <= 0.0 || !corrected_dist.is_finite() { continue; }

        let wall_h = block_size as f32;
        let stake_h = (wall_h / corrected_dist) * proj_plane_dist;
        if !stake_h.is_finite() || stake_h <= 0.0 { continue; }

        let top_f = hh - (stake_h / 2.0);
        let bottom_f = hh + (stake_h / 2.0);
        let top = top_f.max(0.0) as isize;
        let bottom = bottom_f.min(screen_h as f32) as isize;

        let local_x = inter.hit_x - (inter.cell_i as f32 * block_size as f32);
        let local_y = inter.hit_y - (inter.cell_j as f32 * block_size as f32);

        let eps = 1.0;
        let frac = if local_x < eps {
            (local_y / block_size as f32).fract()
        } else if local_x > (block_size as f32 - eps) {
            (local_y / block_size as f32).fract()
        } else if local_y < eps {
            (local_x / block_size as f32).fract()
        } else if local_y > (block_size as f32 - eps) {
            (local_x / block_size as f32).fract()
        } else {
            (local_x / block_size as f32).fract()
        };

        let mut tex_char = inter.impact;
        if tex_char == 'p' { tex_char = ' '; } 

        let (tex_w, tex_h) = texmgr.tex_size(tex_char).unwrap_or((1usize,1usize));
        let tx_index = if tex_w > 1 { ((frac * ((tex_w - 1) as f32)).round() as usize).min(tex_w - 1) } else { 0usize };

        let x_px_start = (col * render_scale) as u32;
        for sy in top..bottom {
            let y_rel = (sy as f32 - top_f) / stake_h; 
            let color = if tex_w == 1 || tex_h == 1 {
                Color::GRAY
            } else {
                let ty_i = ((y_rel * ((tex_h - 1) as f32)).round() as usize).min(tex_h - 1);
                let u = (tx_index as f32) / (tex_w as f32 - 1.0).max(1.0);
                let v = (ty_i as f32) / (tex_h as f32 - 1.0).max(1.0);
                let mut c = texmgr.sample_char(tex_char, u, v);
                c.a = 255;
                c
            };

            framebuffer.set_current_color(color);
            for dx in 0..render_scale {
                let px = x_px_start + dx as u32;
                if px < framebuffer.width() && (sy as u32) < framebuffer.height() {
                    framebuffer.set_pixel(px, sy as u32);
                }
            }
        }
    }

    wall_distances
}

fn draw_minimap_with_fog(
    framebuffer: &mut Framebuffer,
    maze: &Maze,
    fog: &Vec<Vec<bool>>,
    player: &Player,
    enemies: &Vec<Enemy>,
    medkits: &Vec<Medkit>,
    keys: &Vec<PickupKey>,
    bins: &Vec<PickupBinocular>,
    map_w: usize,
    map_h: usize,
    offset_x: usize,
    offset_y: usize,
    block_size_world: usize,
) {
    let cols = maze[0].len();
    let rows = maze.len();
    let block_x = (map_w / cols).max(1);
    let block_y = (map_h / rows).max(1);
    let block = block_x.min(block_y);

    framebuffer.set_current_color(Color::BLACK);
    for y in 0..(block * rows) {
        for x in 0..(block * cols) {
            let px = offset_x + x;
            let py = offset_y + y;
            if (px as u32) < framebuffer.width() && (py as u32) < framebuffer.height() {
                framebuffer.set_pixel(px as u32, py as u32);
            }
        }
    }

    for j in 0..rows {
        for i in 0..cols {
            let revealed = fog[j][i];
            let cell = maze[j][i];
            let base_color = if !revealed {
                Color::BLACK
            } else {
                match cell {
                    '+'|'-'|'|' => Color::DARKGRAY,
                    'p' => Color::RED,
                    'D' => Color::new(150,75,0,255),
                    'g' => Color::GREEN,
                    _ => Color::WHITE,
                }
            };
            framebuffer.set_current_color(base_color);
            let xo = offset_x + i * block;
            let yo = offset_y + j * block;
            for dy in 0..block {
                for dx in 0..block {
                    let px = xo + dx;
                    let py = yo + dy;
                    if (px as u32) < framebuffer.width() && (py as u32) < framebuffer.height() {
                        framebuffer.set_pixel(px as u32, py as u32);
                    }
                }
            }
        }
    }

    for m in medkits.iter() {
        if m.taken { continue; }
        let show = fog[m.cell.1][m.cell.0] || player.binocular_timer > 0.0;
        if show {
            let xo = offset_x + m.cell.0 * block;
            let yo = offset_y + m.cell.1 * block;
            framebuffer.set_current_color(Color::SKYBLUE);
            let cx = (xo + block/2) as i32;
            let cy = (yo + block/2) as i32;
            for oy in -1..=1 { for ox in -1..=1 {
                let sx = cx + ox; let sy = cy + oy;
                if sx >= 0 && sy >= 0 {
                    let su = sx as u32; let sv = sy as u32;
                    if su < framebuffer.width() && sv < framebuffer.height() { framebuffer.set_pixel(su, sv); }
                }
            } }
        }
    }

    for b in bins.iter() { if b.taken { continue; } let xo = offset_x + b.cell.0 * block; let yo = offset_y + b.cell.1 * block; framebuffer.set_current_color(Color::PURPLE); let cx = (xo + block/2) as i32; let cy = (yo + block/2) as i32; for oy in -1..=1 { for ox in -1..=1 { let sx = cx + ox; let sy = cy + oy; if sx >= 0 && sy >= 0 { let su = sx as u32; let sv = sy as u32; if su < framebuffer.width() && sv < framebuffer.height() { framebuffer.set_pixel(su, sv); } } } } }

    for k in keys.iter() {
        if k.taken { continue; }
        let xo = offset_x + k.cell.0 * block;
        let yo = offset_y + k.cell.1 * block;
        framebuffer.set_current_color(Color::GOLD);
        let cx = (xo + block/2) as i32;
        let cy = (yo + block/2) as i32;
        for oy in -1..=1 { for ox in -1..=1 {
            let sx = cx + ox; let sy = cy + oy;
            if sx >= 0 && sy >= 0 {
                let su = sx as u32; let sv = sy as u32;
                if su < framebuffer.width() && sv < framebuffer.height() { framebuffer.set_pixel(su, sv); }
            }
        } }
    }

    for j in 0..rows {
        for i in 0..cols {
            if maze[j][i] == 'g' {
                let xo = offset_x + i * block;
                let yo = offset_y + j * block;
                let cx = (xo + block/2) as i32;
                let cy = (yo + block/2) as i32;
                framebuffer.set_current_color(Color::GREEN);
                for oy in -1..=1 { for ox in -1..=1 {
                    let sx = cx + ox; let sy = cy + oy;
                    if sx >= 0 && sy >= 0 {
                        let su = sx as u32; let sv = sy as u32;
                        if su < framebuffer.width() && sv < framebuffer.height() { framebuffer.set_pixel(su, sv); }
                    }
                } }
            }
        }
    }

    for e in enemies.iter() {
        let ex = ((e.pos.x as usize) / block_size_world).min(cols.saturating_sub(1));
        let ey = ((e.pos.y as usize) / block_size_world).min(rows.saturating_sub(1));
        let revealed = fog[ey][ex];
        let should_draw = revealed || (player.binocular_timer > 0.0);
        if should_draw {
            let xo = offset_x + ex * block;
            let yo = offset_y + ey * block;
            let cx = (xo + block/2) as i32;
            let cy = (yo + block/2) as i32;
            let col = if revealed { Color::ORANGE } else { Color::YELLOW };
            framebuffer.set_current_color(col);
            for oy in -1..=1 { for ox in -1..=1 {
                let sx = cx + ox; let sy = cy + oy;
                if sx >= 0 && sy >= 0 {
                    let su = sx as u32; let sv = sy as u32;
                    if su < framebuffer.width() && sv < framebuffer.height() { framebuffer.set_pixel(su, sv); }
                }
            } }
        }
    }

    let px = ((player.pos.x as usize) / block_size_world).min(cols.saturating_sub(1));
    let py = ((player.pos.y as usize) / block_size_world).min(rows.saturating_sub(1));
    if fog[py][px] {
        let xo = offset_x + px * block;
        let yo = offset_y + py * block;
        let cx = (xo + block/2) as i32;
        let cy = (yo + block/2) as i32;
        framebuffer.set_current_color(Color::MAGENTA);
        for oy in -1..=1 { for ox in -1..=1 {
            let sx = cx + ox; let sy = cy + oy;
            if sx >= 0 && sy >= 0 {
                let su = sx as u32; let sv = sy as u32;
                if su < framebuffer.width() && sv < framebuffer.height() { framebuffer.set_pixel(su, sv); }
            }
        } }
    }
}

fn main() {
    const WINDOW_W: i32 = 1820;
    const WINDOW_H: i32 = 980;
    const RENDER_SCALE: usize = 3;
    const FIXED_DT: f32 = 1.0/60.0;

    let (mut window, raylib_thread) = raylib::init().size(WINDOW_W, WINDOW_H).title("Ihan Marroquin - 23108").build();

    let mimikyu_tex = Image::load_image("assets/mimikyu_1.png")
        .and_then(|img| window.load_texture_from_image(&raylib_thread, &img))
        .expect("assets/mimikyu_1.png missing");
    let medkit_tex = Image::load_image("assets/medkit.png")
        .and_then(|img| window.load_texture_from_image(&raylib_thread, &img))
        .expect("assets/medkit.png missing");
    let key_tex = Image::load_image("assets/key.png")
        .and_then(|img| window.load_texture_from_image(&raylib_thread, &img))
        .expect("assets/key.png missing");
    let binocular_tex = Image::load_image("assets/binoculars.png")
        .and_then(|img| window.load_texture_from_image(&raylib_thread, &img))
        .expect("assets/binoculars.png missing");
    let door_tex = Image::load_image("assets/door.png")
        .and_then(|img| window.load_texture_from_image(&raylib_thread, &img))
        .expect("assets/door.png missing");
    let game_over_tex: Option<Texture2D> = Image::load_image("assets/game_over.png")
        .ok()
        .and_then(|img| window.load_texture_from_image(&raylib_thread, &img).ok());
    let win_tex: Option<Texture2D> = Image::load_image("assets/win.png")
        .ok()
        .and_then(|img| window.load_texture_from_image(&raylib_thread, &img).ok());


    let texmgr = TextureManager::new(&mut window, &raylib_thread);

    let mut music_volume = 0.45_f32;
    let mut audio = AudioManager::new_loop("assets/music.ogg", music_volume, 0.4);

    let mut state = AppState::Menu { selected: 0 };
    let mut last = Instant::now();
    let mut accumulator = 0.0f32;
    let mut fps = 0.0f32;

    let mut maze: Maze = Vec::new();
    let mut player: Option<Player> = None;
    let mut block_size: usize = 0usize;
    let mut enemies: Vec<Enemy> = Vec::new();
    let mut medkits: Vec<Medkit> = Vec::new();
    let mut keys: Vec<PickupKey> = Vec::new();
    let mut bins: Vec<PickupBinocular> = Vec::new();
    let mut fog: Vec<Vec<bool>> = Vec::new();
    let mut message: Option<(String,f32)> = None;
    let mut goal_unlocked: bool = false;

    let mut framebuffer = Framebuffer::new(WINDOW_W as u32, WINDOW_H as u32, Color::RAYWHITE);

    'main_loop: loop {
        if window.window_should_close() { break 'main_loop; }

        let now = Instant::now();
        let mut frame_dt = now.duration_since(last).as_secs_f32();
        last = now;
        if frame_dt > 0.25 { frame_dt = 0.25; }
        fps = if frame_dt > 0.0 { 1.0/frame_dt } else { fps };
        accumulator += frame_dt;

        if let Some((_, ref mut t)) = message { *t -= frame_dt; if *t <= 0.0 { message = None; } }

        match &mut state {
            AppState::Menu { selected } => {
                if window.is_key_pressed(KeyboardKey::KEY_DOWN) { *selected = (*selected + 1) % 3; }
                if window.is_key_pressed(KeyboardKey::KEY_UP) { if *selected == 0 { *selected = 2 } else { *selected -= 1; } }
                if window.is_key_pressed(KeyboardKey::KEY_ENTER) {
                    match *selected {
                        0 => { message = Some(("Iniciando partida...".to_string(), 0.5)); }
                        1 => {
                            state = AppState::SoundMenu { volume: music_volume, previous_selected: *selected };
                        }
                        2 => { state = AppState::Exiting; }
                        _ => {}
                    }
                }
            }
            AppState::SoundMenu { volume, previous_selected } => {
                if window.is_key_down(KeyboardKey::KEY_LEFT) {
                    *volume = (*volume - 0.6 * frame_dt).clamp(0.0, 1.0);
                    audio.set_music_volume(*volume);
                    music_volume = *volume;
                }
                if window.is_key_down(KeyboardKey::KEY_RIGHT) {
                    *volume = (*volume + 0.6 * frame_dt).clamp(0.0, 1.0);
                    audio.set_music_volume(*volume);
                    music_volume = *volume;
                }
                if window.is_key_pressed(KeyboardKey::KEY_ENTER) {
                    state = AppState::Menu { selected: *previous_selected };
                }
            }
            AppState::Playing => {
                if let Some(pl) = player.as_mut() {
                    let input_dt = frame_dt.min(FIXED_DT);
                    if let Some(msg) = process_events(&mut window, pl, &mut maze, block_size, input_dt, &mut goal_unlocked, &audio) {
                        message = Some((msg, 2.0));
                    }
                }
            }
            AppState::Victory => {
                if window.is_key_pressed(KeyboardKey::KEY_ENTER) { state = AppState::Menu { selected: 0 }; }
            }
            AppState::GameOver => {
                if window.is_key_pressed(KeyboardKey::KEY_ENTER) { state = AppState::Menu { selected: 0 }; }
            }
            AppState::Exiting => { break 'main_loop; }
        }

        while accumulator >= FIXED_DT {
            match &mut state {
                AppState::Menu { selected } => {
                    if window.is_key_pressed(KeyboardKey::KEY_ENTER) && *selected == 0 {
                        let w = 12usize; let h = 10usize;
                        let raw = generate_maze_text(w,h);
                        maze = expand_maze(&raw, 2);

                        block_size = {
                            let bw = (WINDOW_W as usize) / maze[0].len();
                            let bh = (WINDOW_H as usize) / maze.len();
                            std::cmp::max(6, std::cmp::min(bw, bh))
                        };

                        fog = vec![vec![false; maze[0].len()]; maze.len()];

                        let mut orig_px = 1usize; let mut orig_py = 1usize;
                        'find_p: for (r,row) in maze.iter().enumerate() {
                            for (c,&ch) in row.iter().enumerate() {
                                if ch == 'p' { orig_px = c; orig_py = r; break 'find_p; }
                            }
                        }

                        let rows = maze.len();
                        let cols = maze[0].len();
                        let mut reachable = vec![vec![false; cols]; rows];
                        use std::collections::VecDeque;
                        let mut q: VecDeque<(usize,usize)> = VecDeque::new();
                        for r in 0..rows {
                            for c in 0..cols {
                                if r==0 || r==rows-1 || c==0 || c==cols-1 {
                                    if maze[r][c] == ' ' {
                                        reachable[r][c] = true;
                                        q.push_back((r,c));
                                    }
                                }
                            }
                        }
                        while let Some((r,c)) = q.pop_front() {
                            let neigh = [(1isize,0),( -1,0),(0,1),(0,-1)];
                            for (dy,dx) in neigh.iter() {
                                let nr = r as isize + dy;
                                let nc = c as isize + dx;
                                if nr >= 0 && nc >= 0 && (nr as usize) < rows && (nc as usize) < cols {
                                    let (nr, nc) = (nr as usize, nc as usize);
                                    if !reachable[nr][nc] && maze[nr][nc] == ' ' {
                                        reachable[nr][nc] = true;
                                        q.push_back((nr,nc));
                                    }
                                }
                            }
                        }

                        let mut spawn_x = orig_px;
                        let mut spawn_y = orig_py;
                        if !reachable[orig_py][orig_px] {
                            let mut best: Option<(usize,usize)> = None;
                            let mut bestd = usize::MAX;
                            for r in 0..rows {
                                for c in 0..cols {
                                    if reachable[r][c] {
                                        let d = ((r as isize - orig_py as isize).abs() + (c as isize - orig_px as isize).abs()) as usize;
                                        if d < bestd {
                                            bestd = d;
                                            best = Some((c,r));
                                        }
                                    }
                                }
                            }
                            if let Some((cx,cy)) = best {
                                spawn_x = cx; spawn_y = cy;
                            } else {
                                'fallback_find: for r in 0..rows {
                                    for c in 0..cols {
                                        if maze[r][c] == ' ' { spawn_x = c; spawn_y = r; break 'fallback_find; }
                                    }
                                }
                            }
                        }

                        maze[spawn_y][spawn_x] = ' ';

                        let goal_x = if cols >= 3 { cols - 2 } else { cols.saturating_sub(1) };
                        let goal_y = if rows >= 3 { rows - 2 } else { rows.saturating_sub(1) };
                        for r in 0..rows { for c in 0..cols { if maze[r][c] == 'g' { maze[r][c] = ' '; } } }
                        maze[goal_y][goal_x] = 'g';
                        let neigh = [(1isize,0isize),(-1,0),(0,1),(0,-1)];
                        for &(dx,dy) in neigh.iter() {
                            let nx = goal_x as isize + dx;
                            let ny = goal_y as isize + dy;
                            if nx >= 0 && ny >= 0 && (ny as usize) < rows && (nx as usize) < cols {
                                let (nxu, nyu) = (nx as usize, ny as usize);
                                if maze[nyu][nxu] != 'g' { maze[nyu][nxu] = 'D'; }
                            }
                        }

                        let cx = (spawn_x * block_size) as f32 + (block_size as f32)/2.0;
                        let cy = (spawn_y * block_size) as f32 + (block_size as f32)/2.0;
                        player = Some(Player::new(cx, cy, 0.0, PI/3.0));
                        goal_unlocked = false;

                        enemies.clear(); medkits.clear(); keys.clear(); bins.clear();
                        let mut spawn1 = (cols.saturating_sub(3), 1usize);
                        'outer1: for r in 1..rows-1 { for c in (1..cols-1).rev() { if maze[r][c] == ' ' { spawn1=(c,r); break 'outer1; } } }
                        let mut spawn2 = (1usize, rows.saturating_sub(3));
                        'outer2: for r in (1..rows-1).rev() { for c in 1..cols-1 { if maze[r][c] == ' ' { spawn2=(c,r); break 'outer2; } } }
                        enemies.push(Enemy::new(spawn1, block_size, 0, 18.0));
                        enemies.push(Enemy::new(spawn2, block_size, 0, 18.0));

                        use rand::{thread_rng, Rng};
                        let mut rng = thread_rng();
                        let mut placed = 0; let want_medkits = 6usize;
                        while placed < want_medkits {
                            let r = rng.gen_range(1..rows-1);
                            let c = rng.gen_range(1..cols-1);
                            if maze[r][c] == ' ' {
                                if !medkits.iter().any(|m| m.cell == (c,r)) {
                                    medkits.push(Medkit { cell:(c,r), taken:false });
                                    placed += 1;
                                }
                            }
                        }

                        let mut placed_key = false;
                        for _ in 0..300 {
                            let r = rng.gen_range(1..rows-1);
                            let c = rng.gen_range(1..cols-1);
                            if maze[r][c] == ' ' { keys.push(PickupKey { cell:(c,r), taken:false }); placed_key=true; break; }
                        }
                        if !placed_key { keys.push(PickupKey { cell:(cols/2, rows/2), taken:false }); }

                        let mut placed_bin=false;
                        for _ in 0..300 {
                            let r = rng.gen_range(1..rows-1); let c = rng.gen_range(1..cols-1);
                            if maze[r][c] == ' ' { bins.push(PickupBinocular { cell:(c,r), taken:false }); placed_bin=true; break; }
                        }
                        if !placed_bin { bins.push(PickupBinocular { cell:(cols/2+1, rows/2), taken:false }); }

                        if let Some(pl) = &player {
                            let pr = (pl.pos.y as usize) / block_size;
                            let pc = (pl.pos.x as usize) / block_size;
                            reveal_fog(&mut fog, pc, pr, &maze, 2);
                        }

                        message = Some(("Recoge la llave antes de abrir la puerta que protege la meta".to_string(), 4.0));
                        state = AppState::Playing;
                    }
                }

                AppState::Playing => {
                    if let Some(pl) = player.as_mut() {
                        pl.update_timers(FIXED_DT);

                        let pr = (pl.pos.y as usize) / block_size;
                        let pc = (pl.pos.x as usize) / block_size;
                        reveal_fog(&mut fog, pc, pr, &maze, 2);

                        for e in enemies.iter_mut() {
                            let attacked = e.update(&maze, block_size, &pl.pos, FIXED_DT);
                            if attacked {
                                pl.apply_damage(50.0);
                                audio.play_sfx("assets/sfx_hurt.ogg", 0.3);
                            }
                        }

                        let player_cell = ((pl.pos.x as usize) / block_size, (pl.pos.y as usize) / block_size);
                        for m in medkits.iter_mut() {
                            if !m.taken && m.cell == player_cell {
                                m.taken = true; pl.pickup_medkit(25.0); message = Some(("Medkit recogido".to_string(), 2.0));
                            }
                        }
                        for k in keys.iter_mut() {
                            if !k.taken && k.cell == player_cell { k.taken = true; pl.pickup_key(); message = Some(("Has recogido la llave!".to_string(), 3.0)); }
                        }
                        for b in bins.iter_mut() {
                            if !b.taken && b.cell == player_cell { b.taken = true; pl.pickup_binoculars(60.0); message = Some(("Binoculares activados 60s".to_string(), 3.0)); }
                        }

                        if pl.health <= 0.0 {
                            audio.play_sfx("assets/sfx_gameover.ogg", 0.3);
                            state = AppState::GameOver;
                        }

                        let i = (pl.pos.x as usize) / block_size;
                        let j = (pl.pos.y as usize) / block_size;
                        if j < maze.len() && i < maze[j].len() && maze[j][i] == 'g' {
                            if pl.has_key || goal_unlocked {
                                audio.play_sfx("assets/sfx_victory.ogg", 0.3);
                                state = AppState::Victory;
                            } else {
                                message = Some(("Necesitas una llave para entrar a la meta".to_string(), 2.5));
                            }
                        }
                    }
                }

                _ => {}
            }

            accumulator -= FIXED_DT;
        }

        match &state {
            AppState::Menu { selected } => {
                draw_menu(&mut window, &raylib_thread, *selected);
            }
            AppState::SoundMenu { volume, .. } => {
                draw_sound_menu(&mut window, &raylib_thread, *volume);
            }
            AppState::Playing => {
                if let Some(pl) = &player {
                    framebuffer.clear();
                    let wall_distances = render_world_textured(&mut framebuffer, &maze, &pl, block_size, RENDER_SCALE, &texmgr);

                    let margin = 10;
                    let stamina_h = 18;
                    let gap1 = 8;
                    let health_h = 18;
                    let mut hud_h = margin + stamina_h + gap1 + health_h;
                    if pl.shield > 0.0 { hud_h += 6 + 18; }
                    let minimap_offset_x = 10usize;
                    let minimap_offset_y = hud_h + 10usize;
                    let mm_w = 180usize; let mm_h = 140usize;
                    draw_minimap_with_fog(&mut framebuffer, &maze, &fog, &pl, &enemies, &medkits, &keys, &bins, mm_w, mm_h, minimap_offset_x, minimap_offset_y, block_size);

                    let screen_w_px = framebuffer.width() as usize;
                    let num_cols = (screen_w_px / RENDER_SCALE).max(1);
                    let proj_plane_dist = (num_cols as f32 / 2.0) / (pl.fov / 2.0).tan();
                    let hh = framebuffer.height() as f32 / 2.0;
                    let mut entries: Vec<(&Texture2D, f32, i32, i32, f32)> = Vec::new();

                    for e in enemies.iter() {
                        let dx = e.pos.x - pl.pos.x; let dy = e.pos.y - pl.pos.y;
                        let dist = (dx*dx + dy*dy).sqrt();
                        let angle_to = dy.atan2(dx);
                        let mut rel = angle_to - pl.a;
                        while rel > std::f32::consts::PI { rel -= 2.0*std::f32::consts::PI; }
                        while rel < -std::f32::consts::PI { rel += 2.0*std::f32::consts::PI; }
                        if rel.abs() > pl.fov/2.0 { continue; }
                        let corrected = dist * rel.cos().abs().max(1e-6);
                        let screen_col = (rel / (pl.fov/2.0)) * (num_cols as f32 / 2.0) + (num_cols as f32 / 2.0);
                        if !screen_col.is_finite() { continue; }
                        let col_idx = screen_col.round() as isize;
                        if col_idx < 0 || col_idx >= wall_distances.len() as isize { continue; }
                        let wall_d = wall_distances[col_idx as usize];
                        if corrected > wall_d { continue; }
                        let sprite_h_px = (block_size as f32 / corrected) * proj_plane_dist;
                        if !sprite_h_px.is_finite() || sprite_h_px <= 0.0 { continue; }
                        let tex_h = mimikyu_tex.height() as f32;
                        if tex_h <= 0.0 { continue; }
                        let scale = sprite_h_px / tex_h;
                        let screen_x_px = screen_col * (RENDER_SCALE as f32);
                        let draw_x = (screen_x_px - (mimikyu_tex.width() as f32 * scale)/2.0).round() as i32;
                        let draw_y = (hh - sprite_h_px/2.0).round() as i32;
                        entries.push((&mimikyu_tex, scale, draw_x, draw_y, corrected));
                    }

                    for m in medkits.iter() {
                        if m.taken { continue; }
                        let mx = (m.cell.0 * block_size) as f32 + (block_size as f32)/2.0;
                        let my = (m.cell.1 * block_size) as f32 + (block_size as f32)/2.0;
                        let dx = mx - pl.pos.x; let dy = my - pl.pos.y;
                        let dist = (dx*dx + dy*dy).sqrt();
                        let angle_to = dy.atan2(dx);
                        let mut rel = angle_to - pl.a;
                        while rel > std::f32::consts::PI { rel -= 2.0*std::f32::consts::PI; }
                        while rel < -std::f32::consts::PI { rel += 2.0*std::f32::consts::PI; }
                        if rel.abs() > pl.fov/2.0 { continue; }
                        let corrected = dist * rel.cos().abs().max(1e-6);
                        let screen_col = (rel / (pl.fov/2.0)) * (num_cols as f32 / 2.0) + (num_cols as f32 / 2.0);
                        let col_idx = screen_col.round() as isize;
                        if col_idx < 0 || col_idx >= wall_distances.len() as isize { continue; }
                        let wall_d = wall_distances[col_idx as usize];
                        if corrected > wall_d { continue; }
                        let sprite_h_px = (block_size as f32 / corrected) * proj_plane_dist * 0.6;
                        let tex_h = medkit_tex.height() as f32;
                        if tex_h <= 0.0 { continue; }
                        let scale = sprite_h_px / tex_h;
                        let screen_x_px = screen_col * (RENDER_SCALE as f32);
                        let draw_x = (screen_x_px - (medkit_tex.width() as f32 * scale)/2.0).round() as i32;
                        let draw_y = (hh - sprite_h_px/2.0).round() as i32;
                        entries.push((&medkit_tex, scale, draw_x, draw_y, corrected));
                    }

                    for k in keys.iter() {
                        if k.taken { continue; }
                        let kx = (k.cell.0 * block_size) as f32 + (block_size as f32)/2.0;
                        let ky = (k.cell.1 * block_size) as f32 + (block_size as f32)/2.0;
                        let dx = kx - pl.pos.x; let dy = ky - pl.pos.y;
                        let dist = (dx*dx + dy*dy).sqrt();
                        let angle_to = dy.atan2(dx);
                        let mut rel = angle_to - pl.a;
                        while rel > std::f32::consts::PI { rel -= 2.0*std::f32::consts::PI; }
                        while rel < -std::f32::consts::PI { rel += 2.0*std::f32::consts::PI; }
                        if rel.abs() > pl.fov/2.0 { continue; }
                        let corrected = dist * rel.cos().abs().max(1e-6);
                        let screen_col = (rel / (pl.fov/2.0)) * (num_cols as f32 / 2.0) + (num_cols as f32 / 2.0);
                        let col_idx = screen_col.round() as isize;
                        if col_idx < 0 || col_idx >= wall_distances.len() as isize { continue; }
                        let wall_d = wall_distances[col_idx as usize];
                        if corrected > wall_d { continue; }
                        let sprite_h_px = (block_size as f32 / corrected) * proj_plane_dist * 0.5;
                        let tex_h = key_tex.height() as f32;
                        if tex_h <= 0.0 { continue; }
                        let scale = sprite_h_px / tex_h;
                        let screen_x_px = screen_col * (RENDER_SCALE as f32);
                        let draw_x = (screen_x_px - (key_tex.width() as f32 * scale)/2.0).round() as i32;
                        let draw_y = (hh - sprite_h_px/2.0).round() as i32;
                        entries.push((&key_tex, scale, draw_x, draw_y, corrected));
                    }

                    for b in bins.iter() {
                        if b.taken { continue; }
                        let bx = (b.cell.0 * block_size) as f32 + (block_size as f32)/2.0;
                        let by = (b.cell.1 * block_size) as f32 + (block_size as f32)/2.0;
                        let dx = bx - pl.pos.x; let dy = by - pl.pos.y;
                        let dist = (dx*dx + dy*dy).sqrt();
                        let angle_to = dy.atan2(dx);
                        let mut rel = angle_to - pl.a;
                        while rel > std::f32::consts::PI { rel -= 2.0*std::f32::consts::PI; }
                        while rel < -std::f32::consts::PI { rel += 2.0*std::f32::consts::PI; }
                        if rel.abs() > pl.fov/2.0 { continue; }
                        let corrected = dist * rel.cos().abs().max(1e-6);
                        let screen_col = (rel / (pl.fov/2.0)) * (num_cols as f32 / 2.0) + (num_cols as f32 / 2.0);
                        let col_idx = screen_col.round() as isize;
                        if col_idx < 0 || col_idx >= wall_distances.len() as isize { continue; }
                        let wall_d = wall_distances[col_idx as usize];
                        if corrected > wall_d { continue; }
                        let sprite_h_px = (block_size as f32 / corrected) * proj_plane_dist * 0.5;
                        let tex_h = binocular_tex.height() as f32;
                        if tex_h <= 0.0 { continue; }
                        let scale = sprite_h_px / tex_h;
                        let screen_x_px = screen_col * (RENDER_SCALE as f32);
                        let draw_x = (screen_x_px - (binocular_tex.width() as f32 * scale)/2.0).round() as i32;
                        let draw_y = (hh - sprite_h_px/2.0).round() as i32;
                        entries.push((&binocular_tex, scale, draw_x, draw_y, corrected));
                    }

                    let rows_m = maze.len(); let cols_m = maze[0].len();
                    for yy in 0..rows_m {
                        for xx in 0..cols_m {
                            if maze[yy][xx] == 'D' {
                                let dx_cell = (xx * block_size) as f32 + (block_size as f32)/2.0;
                                let dy_cell = (yy * block_size) as f32 + (block_size as f32)/2.0;
                                let dxp = dx_cell - pl.pos.x; let dyp = dy_cell - pl.pos.y;
                                let dist = (dxp*dxp + dyp*dyp).sqrt();
                                let angle_to = dyp.atan2(dxp);
                                let mut rel = angle_to - pl.a;
                                while rel > std::f32::consts::PI { rel -= 2.0*std::f32::consts::PI; }
                                while rel < -std::f32::consts::PI { rel += 2.0*std::f32::consts::PI; }
                                if rel.abs() > pl.fov/2.0 { continue; }
                                let corrected = dist * rel.cos().abs().max(1e-6);
                                let screen_col = (rel / (pl.fov/2.0)) * (num_cols as f32 / 2.0) + (num_cols as f32 / 2.0);
                                let col_idx = screen_col.round() as isize;
                                if col_idx < 0 || col_idx >= wall_distances.len() as isize { continue; }
                                let wall_d = wall_distances[col_idx as usize];
                                if corrected > wall_d { continue; }
                                let sprite_h_px = (block_size as f32 / corrected) * proj_plane_dist * 0.9;
                                let tex_h = door_tex.height() as f32;
                                if tex_h <= 0.0 { continue; }
                                let scale = sprite_h_px / tex_h;
                                let screen_x_px = screen_col * (RENDER_SCALE as f32);
                                let draw_x = (screen_x_px - (door_tex.width() as f32 * scale)/2.0).round() as i32;
                                let draw_y = (hh - sprite_h_px/2.0).round() as i32;
                                entries.push((&door_tex, scale, draw_x, draw_y, corrected));
                            }
                        }
                    }

                    entries.sort_by(|a,b| b.4.partial_cmp(&a.4).unwrap_or(std::cmp::Ordering::Equal));
                    let sprite_draws: Vec<(&Texture2D, f32, i32, i32)> = entries.iter().map(|&(t,s,x,y,_d)| (t,s,x,y)).collect();

                    let mut fps_text = format!("FPS: {:.1}", fps);
                    if pl.binocular_timer > 0.0 { fps_text.push_str(&format!("   BIN: {}s", pl.binocular_timer.round() as i32)); }
                    if let Some((ref msg, _t)) = message { fps_text.push_str(&format!("   MSG: {}", msg)); }

                    framebuffer.swap_buffers_with_fps(
                        &mut window,
                        &raylib_thread,
                        Some(&fps_text),
                        Some((pl.stamina, pl.stamina_max)),
                        Some((pl.health, pl.health_max)),
                        Some((pl.shield, pl.shield_max)),
                        Some(&sprite_draws),
                    );
                } else {
                    draw_menu(&mut window, &raylib_thread, 0);
                }
            }
            AppState::Victory => { draw_victory(&mut window, &raylib_thread, win_tex.as_ref());}
            AppState::GameOver => { draw_gameover(&mut window, &raylib_thread, game_over_tex.as_ref());}
            AppState::Exiting => { break 'main_loop; }
        }
    }
}

fn reveal_fog(fog: &mut Vec<Vec<bool>>, cx: usize, cy: usize, maze: &Maze, r: i32) {
    let rows = maze.len() as i32;
    let cols = maze[0].len() as i32;
    let cr = cy as i32;
    let cc = cx as i32;
    for dy in -r..=r {
        for dx in -r..=r {
            let ny = cr + dy;
            let nx = cc + dx;
            if nx >= 0 && ny >= 0 && nx < cols && ny < rows {
                let dist2 = dx*dx + dy*dy;
                if dist2 as f32 <= (r as f32) * (r as f32) { fog[ny as usize][nx as usize] = true; }
            }
        }
    }
}
