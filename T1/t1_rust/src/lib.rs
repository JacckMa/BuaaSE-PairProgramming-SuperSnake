use wasm_bindgen::prelude::*;
use std::collections::{VecDeque, HashSet};

#[wasm_bindgen]
pub fn greedy_snake_move(snake: &[i32], food: &[i32]) -> i32 {
    // 解析蛇头和食物坐标
    let (head_x, head_y) = (snake[0], snake[1]);
    let (food_x, food_y) = (food[0], food[1]);

    // 构建障碍物集合：排除蛇尾（最后一节），因为移动时尾部会被移除
    let mut obstacles = HashSet::new();
    // 蛇的数组长度至少为 8（4 节），障碍物使用索引范围 [2, snake.len() - 2)
    for i in (2..snake.len()-2).step_by(2) {
        obstacles.insert((snake[i], snake[i+1]));
    }

    // BFS 初始化
    let mut queue = VecDeque::new();
    let mut visited = HashSet::new();
    queue.push_back((head_x, head_y, vec![]));
    visited.insert((head_x, head_y));

    // 四个方向：上、左、下、右
    let directions = [(0, 1), (-1, 0), (0, -1), (1, 0)];

    while let Some((x, y, path)) = queue.pop_front() {
        // 找到食物时返回首个移动方向
        if x == food_x && y == food_y {
            return path.first().copied().unwrap_or(0);
        }
        // 遍历四个方向
        for (dir, (dx, dy)) in directions.iter().enumerate() {
            let (nx, ny) = (x + dx, y + dy);
            let mut new_path = path.clone();
            new_path.push(dir as i32);

            // 检查边界条件和是否碰撞（以及是否已访问）
            if nx >= 1 && nx <= 8 &&
               ny >= 1 && ny <= 8 &&
               !obstacles.contains(&(nx, ny)) &&
               !visited.contains(&(nx, ny))
            {
                visited.insert((nx, ny));
                queue.push_back((nx, ny, new_path));
            }
        }
    }

    // 如果 BFS 找不到路径，选择第一个不碰撞的方向（仍然排除尾部）
    for dir in 0..4 {
        let (dx, dy) = directions[dir];
        let (nx, ny) = (head_x + dx, head_y + dy);
        if nx >= 1 && nx <= 8 && ny >= 1 && ny <= 8 && !obstacles.contains(&(nx, ny)) {
            return dir as i32;
        }
    }
    0 // 默认返回上方向
}