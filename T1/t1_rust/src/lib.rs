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

#[cfg(test)]
mod tests {
    use super::*;

    // 测试1：食物在蛇头正上方，且路径畅通
    #[test]
    fn test_food_above() {
        // 蛇的表示（长度8）：头部 (4,4)，障碍物为 (1,1) 和 (1,2)，尾部 (1,3) 不计为障碍物
        // 食物位置在 (4,8)
        // 预期：最短路径的首步为上方，即返回 0
        assert_eq!(greedy_snake_move(&[4,4, 1,1, 1,2, 1,3], &[4,8]), 0);
    }

    // 测试2：食物在蛇头正左方
    #[test]
    fn test_food_left() {
        // 蛇：头部 (4,4)，障碍物为 (8,8) 和 (8,7)，尾部 (8,6)
        // 食物位置在 (1,4)
        // 预期：最短路径首步为左，即返回 1
        assert_eq!(greedy_snake_move(&[4,4, 8,8, 8,7, 8,6], &[1,4]), 1);
    }

    // 测试3：食物在蛇头正下方
    #[test]
    fn test_food_down() {
        // 蛇：头部 (4,4)，障碍物为 (1,1) 和 (1,2)，尾部 (1,3)
        // 食物位置在 (4,1)
        // 预期：返回下方方向，方向值为 2
        assert_eq!(greedy_snake_move(&[4,4, 1,1, 1,2, 1,3], &[4,1]), 2);
    }

    // 测试4：食物在蛇头正右方
    #[test]
    fn test_food_right() {
        // 蛇：头部 (4,4)，障碍物为 (1,1) 和 (1,2)，尾部 (1,3)
        // 食物位置在 (8,4)
        // 预期：返回右方方向，方向值为 3
        assert_eq!(greedy_snake_move(&[4,4, 1,1, 1,2, 1,3], &[8,4]), 3);
    }

    // 测试5：食物位置与蛇头重合
    #[test]
    fn test_food_at_head() {
        // 当蛇头已经位于食物所在位置时，
        // BFS 在初始状态就满足条件，返回 path.first().unwrap_or(0)，即默认返回 0
        assert_eq!(greedy_snake_move(&[4,4, 1,1, 1,2, 1,3], &[4,4]), 0);
    }

    // 测试6：所有相邻位置均被阻挡（构造较长的蛇使得蛇头四周均为障碍）
    #[test]
    fn test_all_directions_blocked() {
        // 构造一个长度为 12 的蛇，使得障碍物（索引 2,4,6,8）正好覆盖蛇头 (4,4) 周围的四个方向
        // 具体：
        //   头部：(4,4)
        //   障碍物： (4,5)（上）、(3,4)（左）、(4,3)（下）、(5,4)（右）
        //   尾部：(6,6)（不计障碍物）
        let snake = [4,4, 4,5, 3,4, 4,3, 5,4, 6,6];
        // 食物放在任意无法到达的位置，例如 (8,8)
        // 此时 BFS 无法找到路径，后续 fallback 检查发现四个方向都不安全，最终返回默认的 0
        assert_eq!(greedy_snake_move(&snake, &[8,8]), 0);
    }

    // 测试7：复杂寻路 —— 需要绕行的情况
    #[test]
    fn test_complex_detour() {
        // 蛇：头部 (4,4)，障碍物为 (1,1) 和 (1,2)，尾部 (1,3)
        // 食物位置在 (3,6)
        // 分析：虽然直接向左也可行，但 BFS 按照顺序（上、左、下、右）探索，最先扩展的方向为上，
        // 并经过上方和左上角到达食物，因此预期首步为上（0）
        assert_eq!(greedy_snake_move(&[4,4, 1,1, 1,2, 1,3], &[3,6]), 0);
    }

    // 测试8：蛇头位于边界附近
    #[test]
    fn test_near_border() {
        // 蛇头位于左下角 (1,1)
        // 蛇： [1,1, 2,2, 2,3, 2,4]，障碍物为 (2,2) 和 (2,3)，尾部 (2,4)
        // 食物位置设为 (1,8)（上方）
        // 预期：返回上方方向 0
        assert_eq!(greedy_snake_move(&[1,1, 2,2, 2,3, 2,4], &[1,8]), 0);
    }

    // 测试9：较长蛇的情况，验证障碍物计算正确
    #[test]
    fn test_longer_snake() {
        // 构造一个长度为 12 的蛇：
        //   头部 (4,4)
        //   障碍物：依次为 (8,8)、(8,7)、(8,6)、(8,5)
        //   尾部：(8,4) 不计为障碍物
        let snake = [4,4, 8,8, 8,7, 8,6, 8,5, 8,4];
        // 食物在 (4,8) ，预期：最佳路径是向上，因此返回 0
        assert_eq!(greedy_snake_move(&snake, &[4,8]), 0);
    }
}