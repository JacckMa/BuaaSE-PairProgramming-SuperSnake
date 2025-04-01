use wasm_bindgen::prelude::*;
use std::collections::{VecDeque, HashSet};

#[wasm_bindgen]
pub fn greedy_snake_move_barriers(snake: &[i32], food: &[i32], barriers: &[i32]) -> i32 {
    // 解析蛇头和食物坐标
    let (head_x, head_y) = (snake[0], snake[1]);
    let (food_x, food_y) = (food[0], food[1]);

    // 构建障碍物集合：包括蛇身（排除蛇尾）以及额外障碍物
    let mut obstacles = HashSet::new();
    // 将蛇的障碍物加入集合，蛇身数组至少有 8 个元素，障碍物取索引范围 [2, snake.len()-2)，步长 2
    for i in (2..snake.len()-2).step_by(2) {
        obstacles.insert((snake[i], snake[i+1]));
    }
    // 将障碍物数组中的 12 个障碍物（24 个数字）加入集合
    for i in (0..barriers.len()).step_by(2) {
        obstacles.insert((barriers[i], barriers[i+1]));
    }

    // BFS 初始化
    let mut queue = VecDeque::new();
    let mut visited = HashSet::new();
    // 在队列中保存当前坐标以及从蛇头出发的移动路径
    queue.push_back((head_x, head_y, vec![]));
    visited.insert((head_x, head_y));

    // 定义四个移动方向：上、左、下、右
    let directions = [(0, 1), (-1, 0), (0, -1), (1, 0)];

    // 执行 BFS 搜索到达果子的位置
    while let Some((x, y, path)) = queue.pop_front() {
        if x == food_x && y == food_y {
            // 找到果子时返回路径中的第一个移动方向，如果路径为空，则默认返回 0
            return path.first().copied().unwrap_or(0);
        }
        // 遍历所有可能的方向
        for (dir, (dx, dy)) in directions.iter().enumerate() {
            let (nx, ny) = (x + dx, y + dy);
            let mut new_path = path.clone();
            new_path.push(dir as i32);

            // 检查是否在场地边界内、没有碰到障碍物，且该位置未被访问
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

    // 如果 BFS 搜索不到果子，按照题目要求直接返回 -1 表示不可达
    -1
}
