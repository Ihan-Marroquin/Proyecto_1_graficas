use rand::{thread_rng, seq::SliceRandom};
use crate::maze::Maze;


pub fn generate_maze_text(width_cells: usize, height_cells: usize) -> Maze {
    let mut visited = vec![vec![false; width_cells]; height_cells];
    let mut stack = Vec::new();
    let mut rng = thread_rng();

    let dirs = [(1isize,0isize), (-1,0), (0,1), (0,-1)];

    visited[0][0] = true;
    stack.push((0usize, 0usize));

    while let Some((cx, cy)) = stack.pop() {
        let mut neighbors = Vec::new();
        for &(dx, dy) in dirs.iter() {
            let nx = cx as isize + dx;
            let ny = cy as isize + dy;
            if nx >= 0 && ny >= 0 && (nx as usize) < width_cells && (ny as usize) < height_cells {
                if !visited[ny as usize][nx as usize] {
                    neighbors.push((nx as usize, ny as usize));
                }
            }
        }

        if !neighbors.is_empty() {
            stack.push((cx, cy));
            let &(nx, ny) = neighbors.choose(&mut rng).unwrap();
            visited[ny][nx] = true;
            stack.push((nx, ny));
        }
    }

    let out_h = height_cells * 2 + 1;
    let out_w = width_cells * 2 + 1;
    let mut grid = vec![vec!['+'; out_w]; out_h];

    for y in 0..out_h {
        for x in 0..out_w {
            if y % 2 == 1 && x % 2 == 1 {
                grid[y][x] = ' '; 
            } else if y % 2 == 0 && x % 2 == 0 {
                grid[y][x] = '+';
            } else if y % 2 == 0 {
                grid[y][x] = '-';
            } else {
                grid[y][x] = '|';
            }
        }
    }

    let mut visited2 = vec![vec![false; width_cells]; height_cells];
    let mut stack2 = Vec::new();
    visited2[0][0] = true;
    stack2.push((0usize, 0usize));

    while let Some((cx, cy)) = stack2.pop() {
        let mut neighbors = Vec::new();
        for &(dx, dy) in dirs.iter() {
            let nx = cx as isize + dx;
            let ny = cy as isize + dy;
            if nx >= 0 && ny >= 0 && (nx as usize) < width_cells && (ny as usize) < height_cells {
                if !visited2[ny as usize][nx as usize] {
                    neighbors.push((nx as usize, ny as usize));
                }
            }
        }

        if !neighbors.is_empty() {
            stack2.push((cx, cy));
            let &(nx, ny) = neighbors.choose(&mut rng).unwrap();

            let gx = cx * 2 + 1;
            let gy = cy * 2 + 1;
            let nxg = nx * 2 + 1;
            let nyg = ny * 2 + 1;
            let wall_x = (gx + nxg) / 2;
            let wall_y = (gy + nyg) / 2;
            grid[wall_y][wall_x] = ' ';
            visited2[ny][nx] = true;
            stack2.push((nx, ny));
        }
    }
    grid[1][1] = 'p';
    grid[out_h - 2][out_w - 2] = 'g';

    grid
}
