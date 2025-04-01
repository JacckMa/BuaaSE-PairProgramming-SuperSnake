use wasm_bindgen::prelude::*;
use web_sys::console;
use std::collections::HashMap;
use std::cell::RefCell;

// 是否输出调试日志
const LOG_ENABLED: bool = false;

// 全局变量：保存其他蛇的历史轨迹（key：固定索引）
thread_local! {
    static SNAKE_TRAJECTORIES: RefCell<HashMap<usize, Vec<(i32, i32)>>> = RefCell::new(HashMap::new());
}
// 全局变量：保存上一回合果子坐标，每两个数字构成一个 (x, y)
thread_local! {
    static LAST_FOODS: RefCell<Vec<(i32, i32)>> = RefCell::new(Vec::new());
}
// 全局变量：保存其他蛇的累计目标得分（key：固定索引）
thread_local! {
    static SNAKE_SCORES: RefCell<HashMap<usize, f32>> = RefCell::new(HashMap::new());
}
// 全局变量：保存我方的累计目标得分
thread_local! {
    static MY_SCORE: RefCell<f32> = RefCell::new(0.0);
}
// 全局变量：保存上一回合敌蛇坐标（固定索引对应的蛇体坐标）
thread_local! {
    static LAST_ENEMY_SNAKES: RefCell<HashMap<usize, Vec<(i32, i32)>>> = RefCell::new(HashMap::new());
}

// 全局变量：保存游戏模式（1v1 或 4 蛇对战）
// 1 表示 1v1 模式，3 表示 4 蛇对战模式
thread_local! {
    static GAME_MODE: RefCell<Option<i32>> = RefCell::new(None);
}

// 辅助函数：将 (x,y) 转换为数组索引（棋盘坐标从 1 开始）
fn pos_to_index(x: i32, y: i32, board_size: usize) -> usize {
    ((y - 1) as usize) * board_size + ((x - 1) as usize)
}

/// 解析单条蛇的坐标（坐标从 1 开始），遇到 -1 则停止
fn parse_snake_coords(snake: &[i32]) -> Vec<(i32, i32)> {
    let mut coords = Vec::with_capacity(4);
    for i in 0..4 {
        let x = snake[2 * i];
        let y = snake[2 * i + 1];
        if x >= 1 && y >= 1 {
            coords.push((x, y));
        } else {
            break;
        }
    }
    coords
}

/// 解析其他蛇，返回 (Vec<每条蛇坐标>, Vec<(蛇头x, 蛇头y, 蛇体长度)>)
fn parse_other_snakes(other_snakes: &[i32]) -> (Vec<Vec<(i32, i32)>>, Vec<(i32, i32, usize)>) {
    let snake_num = other_snakes.len() / 8;
    let mut all_coords = Vec::with_capacity(snake_num);
    let mut heads = Vec::with_capacity(snake_num);
    for i in 0..snake_num {
        let start = i * 8;
        let end = start + 8;
        let snake_data = &other_snakes[start..end];
        let coords = parse_snake_coords(snake_data);
        if !coords.is_empty() {
            heads.push((coords[0].0, coords[0].1, coords.len()));
        }
        all_coords.push(coords);
    }
    (all_coords, heads)
}

/// 模拟移动，生成新的蛇体。若新头在果子上，则不移除尾部（即蛇体增长），否则删除尾部
fn simulate_move(my_snake_coords: &Vec<(i32, i32)>, new_head: (i32, i32), food_coords: &Vec<(i32, i32)>) -> Vec<(i32, i32)> {
    let mut new_body = Vec::with_capacity(my_snake_coords.len());
    new_body.push(new_head);
    for idx in 0..(my_snake_coords.len() - 1) {
        new_body.push(my_snake_coords[idx]);
    }
    if !food_coords.contains(&new_head) {
        new_body.pop();
    }
    new_body
}

/// 构建危险地图，标记所有蛇（包括我方和其他蛇）所在的格子为危险
fn build_danger_map(my_snake_coords: &Vec<(i32, i32)>, other_snakes_coords: &Vec<Vec<(i32, i32)>>, n: i32, board_size: usize) -> Vec<bool> {
    let mut dangerous = vec![false; board_size * board_size];
    for snake_body in other_snakes_coords {
        for &(bx, by) in snake_body {
            if bx >= 1 && by >= 1 && bx <= n && by <= n {
                let idx = pos_to_index(bx, by, board_size);
                dangerous[idx] = true;
            }
        }
    }
    for &(bx, by) in my_snake_coords {
        if bx >= 1 && by >= 1 && bx <= n && by <= n {
            let idx = pos_to_index(bx, by, board_size);
            dangerous[idx] = true;
        }
    }
    dangerous
}

/// 更新其他蛇的历史轨迹，对于已死亡的蛇删除记录
fn update_trajectories(other_snakes_coords: &Vec<Vec<(i32, i32)>>) {
    SNAKE_TRAJECTORIES.with(|traj| {
        let mut traj = traj.borrow_mut();
        for (i, snake_body) in other_snakes_coords.iter().enumerate() {
            if snake_body.is_empty() {
                traj.remove(&i);
            } else {
                let head = snake_body[0];
                let entry = traj.entry(i).or_insert(Vec::new());
                entry.push(head);
                if entry.len() > 5 {
                    entry.remove(0);
                }
            }
        }
        if LOG_ENABLED {
            console::log_1(&format!("[TRAJECTORY] {:?}", *traj).into());
        }
    });
}

/// 解析果子坐标（坐标从 1 开始）
fn parse_food_coords(foods: &[i32]) -> Vec<(i32, i32)> {
    let food_num = foods.len() / 2;
    let mut coords = Vec::with_capacity(food_num);
    for i in 0..food_num {
        let x = foods[2 * i];
        let y = foods[2 * i + 1];
        if x >= 1 && y >= 1 {
            coords.push((x, y));
        }
    }
    coords
}

/// 根据其他蛇历史轨迹预测敌蛇争夺果子的情况，返回 (contested_food, enemy_dist)
fn predict_contested_food(food_coords: &Vec<(i32, i32)>, other_heads: &Vec<(i32, i32, usize)>) -> (Vec<bool>, Vec<i32>) {
    let mut contested = vec![false; food_coords.len()];
    let mut enemy_dist = vec![i32::MAX; food_coords.len()];
    SNAKE_TRAJECTORIES.with(|traj| {
        let traj = traj.borrow();
        for (&snake_id, positions) in traj.iter() {
            if snake_id >= other_heads.len() {
                continue;
            }
            if positions.len() >= 2 {
                let first = positions.first().unwrap();
                let last = positions.last().unwrap();
                let dx = last.0 - first.0;
                let dy = last.1 - first.1;
                let predicted = (other_heads[snake_id].0 + dx, other_heads[snake_id].1 + dy);
                for (i, &(fx, fy)) in food_coords.iter().enumerate() {
                    let dist = (predicted.0 - fx).abs() + (predicted.1 - fy).abs();
                    if dist < enemy_dist[i] {
                        enemy_dist[i] = dist;
                    }
                    if dist <= 2 {
                        contested[i] = true;
                    }
                }
            }
        }
        if LOG_ENABLED {
            console::log_1(&format!("[CONTESTED] {:?}", contested).into());
            console::log_1(&format!("[ENEMY_DIST] {:?}", enemy_dist).into());
        }
    });
    (contested, enemy_dist)
}

/// 匹配当前敌蛇与上一回合敌蛇记录，返回 mapping 向量，mapping[i] 为当前敌蛇 i 的固定索引
fn match_enemy_snakes(current: &Vec<Vec<(i32, i32)>>) -> Vec<usize> {
    let last = LAST_ENEMY_SNAKES.with(|les| les.borrow().clone());
    let mut mapping = Vec::with_capacity(current.len());
    let mut used: HashMap<usize, bool> = HashMap::new();
    let mut next_id = if let Some(max_id) = last.keys().max() {
        max_id + 1
    } else {
        0
    };
    for curr in current.iter() {
        let mut best_match: Option<usize> = None;
        let mut best_count = 0;
        for (&id, prev) in last.iter() {
            let count = curr.iter().filter(|&&p| prev.contains(&p)).count();
            if count >= 3 && count > best_count && !used.get(&id).copied().unwrap_or(false) {
                best_match = Some(id);
                best_count = count;
            }
        }
        if let Some(id) = best_match {
            mapping.push(id);
            used.insert(id, true);
        } else {
            mapping.push(next_id);
            next_id += 1;
        }
    }
    mapping
}

/// 更新全局 LAST_ENEMY_SNAKES，一次性更新，不在方向循环中调用
fn update_last_enemy_snakes(current: &Vec<Vec<(i32, i32)>>, mapping: &Vec<usize>) {
    let mut new_last = HashMap::new();
    for (i, &assigned) in mapping.iter().enumerate() {
        new_last.insert(assigned, current[i].clone());
    }
    LAST_ENEMY_SNAKES.with(|les| {
        *les.borrow_mut() = new_last;
    });
}

/// 计算单个蛇的目标得分：
/// 每次吃到一个果子加 1 分；如果其蛇头出现在上一回合的果子列表中，则返回 1，否则返回 0。
fn compute_individual_score(head: (i32, i32), last_food_coords: &Vec<(i32, i32)>) -> f32 {
    if last_food_coords.contains(&head) {
        1.0
    } else {
        0.0
    }
}

/// 更新并返回敌蛇累计目标得分，基于上一回合果子坐标；使用 mapping 确定固定索引
fn update_and_get_enemy_scores_with_mapping(
    other_heads: &Vec<(i32, i32, usize)>,
    last_food_coords: &Vec<(i32, i32)>,
    mapping: &Vec<usize>
) -> Vec<f32> {
    let mut scores = Vec::with_capacity(other_heads.len());
    SNAKE_SCORES.with(|scores_map| {
        let mut scores_map = scores_map.borrow_mut();
        for (i, &(hx, hy, _)) in other_heads.iter().enumerate() {
            let assigned = mapping[i];
            let score = compute_individual_score((hx, hy), last_food_coords);
            let cumulative = scores_map.get(&assigned).copied().unwrap_or(0.0) + score;
            scores_map.insert(assigned, cumulative);
            scores.push(cumulative);
        }
    });
    scores
}
/// 返回敌蛇累计目标得分，基于上一回合果子坐标；使用 mapping 确定固定索引
fn get_enemy_scores_with_mapping(
    other_heads: &Vec<(i32, i32, usize)>,
    mapping: &Vec<usize>
) -> Vec<f32> {
    let mut scores = Vec::with_capacity(other_heads.len());
    SNAKE_SCORES.with(|scores_map| {
        let scores_map = scores_map.borrow();
        for (i, &(_, _, _)) in other_heads.iter().enumerate() {
            let assigned = mapping[i];
            let cumulative = scores_map.get(&assigned).copied().unwrap_or(0.0);
            scores.push(cumulative);
        }
    });
    scores
}

/// 计算果子得分：若吃到果子则 +100，否则按曼哈顿距离扣分；对争夺果子和敌蛇预测优势情况加大扣分
fn compute_food_score(
    new_head: (i32, i32),
    food_coords: &Vec<(i32, i32)>,
    contested_food: &Vec<bool>,
    enemy_dist: &Vec<i32>,
    eat: bool
) -> f32 {
    let mut score = 0.0;
    // 根据游戏模式确定中心位置
    let center = GAME_MODE.with(|m| {
        if let Some(mode) = *m.borrow() {
            if mode == 1 {
                (2.5, 2.5)
            } else if mode == 3 {
                (4.5, 4.5)
            } else {
                (2.5, 2.5)
            }
        } else {
            (2.5, 2.5)
        }
    });
    
    if eat {
        score += 100.0;
    } else if !food_coords.is_empty() {
        let mut min_dist = i32::MAX;
        for (i, &(fx, fy)) in food_coords.iter().enumerate() {
            let dist = (new_head.0 - fx).abs() + (new_head.1 - fy).abs();
            // 计算果子与中心的曼哈顿距离（以浮点数计算）
            let center_dist = ((fx as f64) - center.0).abs() + ((fy as f64) - center.1).abs();
            // 如果果子靠近中心（距离小于1.5），则增加额外权重 bonus
            let bonus = if center_dist < 1.5 { 10.0 } else { 0.0 };
            
            if contested_food.get(i).copied().unwrap_or(false) {
                score += -3.0 * dist as f32 + bonus;
            } else {
                if dist < min_dist {
                    min_dist = dist;
                }
                if enemy_dist.get(i).copied().unwrap_or(i32::MAX) < dist {
                    score += -dist as f32 + bonus;
                }
            }
        }
        if min_dist != i32::MAX {
            score += -min_dist as f32;
        }
    }
    score
}

/// 使用洪水填充计算从 start 出发的可活动区域面积
fn compute_free_space(start: (i32, i32), obstacles: &Vec<bool>, n: i32, board_size: usize) -> i32 {
    let mut visited = vec![false; board_size * board_size];
    let mut queue = Vec::new();
    let mut area = 0;
    if start.0 < 1 || start.0 > n || start.1 < 1 || start.1 > n {
        return 0;
    }
    let idx = pos_to_index(start.0, start.1, board_size);
    if obstacles[idx] {
        return 0;
    }
    queue.push(start);
    visited[idx] = true;
    while let Some((cx, cy)) = queue.pop() {
        area += 1;
        let neighbors = [(cx, cy+1), (cx-1, cy), (cx, cy-1), (cx+1, cy)];
        for &(nx, ny) in neighbors.iter() {
            if nx < 1 || nx > n || ny < 1 || ny > n {
                continue;
            }
            let ni = pos_to_index(nx, ny, board_size);
            if !visited[ni] && !obstacles[ni] {
                visited[ni] = true;
                queue.push((nx, ny));
            }
        }
    }
    area
}

/// 计算生存得分：使用洪水填充计算新头的可活动区域，若区域小于蛇体长度则扣分；对邻近敌蛇头额外扣分
fn compute_survival_score(new_head: (i32, i32), new_body: &Vec<(i32, i32)>, other_snakes_coords: &Vec<Vec<(i32, i32)>>, other_heads: &Vec<(i32, i32, usize)>, n: i32, board_size: usize, my_length: usize) -> f32 {
    let mut survival_score = 0.0;
    let mut space = 0;
    let mut visited = vec![false; board_size * board_size];
    let mut queue = Vec::with_capacity(board_size * board_size);
    let mut static_block = vec![false; board_size * board_size];
    for snake_body in other_snakes_coords {
        for &(bx, by) in snake_body {
            if bx >= 1 && by >= 1 && bx <= n && by <= n {
                let idx = pos_to_index(bx, by, board_size);
                static_block[idx] = true;
            }
        }
    }
    for &(bx, by) in new_body {
        if bx >= 1 && by >= 1 && bx <= n && by <= n {
            let idx = pos_to_index(bx, by, board_size);
            static_block[idx] = true;
        }
    }
    queue.push(new_head);
    let start_idx = pos_to_index(new_head.0, new_head.1, board_size);
    visited[start_idx] = true;
    static_block[start_idx] = true;
    while let Some((cx, cy)) = queue.pop() {
        space += 1;
        let neighbors = [(cx, cy+1), (cx-1, cy), (cx, cy-1), (cx+1, cy)];
        for &(nx, ny) in &neighbors {
            if nx < 1 || nx > n || ny < 1 || ny > n {
                continue;
            }
            let ni = pos_to_index(nx, ny, board_size);
            if visited[ni] || static_block[ni] {
                continue;
            }
            visited[ni] = true;
            queue.push((nx, ny));
        }
    }
    // 修复两个编译错误：
    if space < my_length as i32 {
        survival_score -= 100.0;
    } else {
        survival_score += 50.0 * (space as f32).sqrt(); // 1. 使用f32的sqrt方法 2. 改为浮点运算
    }

    survival_score
}

/// 重新设计的进攻得分函数：
/// 场景1：对于每个敌蛇，根据全局累计目标得分（SNAKE_SCORES）与我方累计目标得分（MY_SCORE）比较；
// 如果我方累计得分高，并且我方新头与该敌蛇头相邻，则奖励额外分（同归于尽奖励）；
/// 场景2：对于每个敌蛇，如果我方新头靠近（距离≤2），模拟阻断后计算敌蛇自由空间，
// 若自由空间低于阈值，则奖励 (阈值 - 自由空间)/距离 得分。
fn compute_aggression_score(
    new_head: (i32, i32),
    other_heads: &Vec<(i32, i32, usize)>,
    n: i32,
    board_size: usize,
    dangerous: &Vec<bool>,
    last_food_coords: &Vec<(i32, i32)>,
    mapping: &Vec<usize>
) -> f32 {
    let mut aggression_score = 0.0;
    let free_space_threshold = 3;
    let my_cumulative = MY_SCORE.with(|ms| *ms.borrow());
    let enemy_scores = get_enemy_scores_with_mapping(other_heads, mapping);
    // 场景1：同归于尽机会
    for (&enemy_score, &(hx, hy, _)) in enemy_scores.iter().zip(other_heads.iter()) {
        if LOG_ENABLED {
            console::log_1(&format!("[AGGRESSION] My score: {}, Enemy score: {}", my_cumulative, enemy_score).into());
        }
        // TODO：这里的阈值需要调整 >= ??
        if my_cumulative > enemy_score {
            let dist = (new_head.0 - hx).abs() + (new_head.1 - hy).abs();
            if dist <= 2 && !dangerous[pos_to_index(new_head.0, new_head.1, board_size)] {
                // 根据游戏模式调整奖励值
                GAME_MODE.with(|mode| {
                    if let Some(mode) = *mode.borrow() {
                        if mode == 3 {
                            aggression_score += 100.0; // 4蛇模式奖励100
                        } else {
                            aggression_score += 1000.0; // 其他模式奖励1000
                        }
                    }
                });
            }
        }
    }
    // 场景2：逼死敌蛇
    for &(hx, hy, _) in other_heads {
        let dist = (new_head.0 - hx).abs() + (new_head.1 - hy).abs();
        if dist > 2 {
            continue;
        }
        let mut obstacles = dangerous.clone();
        let our_idx = pos_to_index(new_head.0, new_head.1, board_size);
        obstacles[our_idx] = true;
        let enemy_space = compute_free_space((hx, hy), &obstacles, n, board_size);
        if enemy_space < free_space_threshold {
            aggression_score += (free_space_threshold - enemy_space) as f32 / dist as f32;
        }
    }
    aggression_score
}

/// 主策略函数，根据当前棋盘信息返回最佳移动方向（0:上, 1:左, 2:下, 3:右）。
/// 本函数内部使用全局变量保存上一回合果子和敌蛇数据，保证敌蛇索引固定并累计目标得分；
/// 同时，更新我方累计目标得分（每吃到一个果子加 1 分）。
#[wasm_bindgen]
pub fn greedy_snake_step(
    n: i32,
    my_snake: Vec<i32>,
    _snake_num: i32, // 不再依赖传入的
    other_snakes: Vec<i32>,
    _food_num: i32,  // 不再依赖传入的
    foods: Vec<i32>,
    round: i32
) -> i32 {
     // 初始化游戏模式

    GAME_MODE.with(|mode| {
        let mut mode = mode.borrow_mut();
        if mode.is_none() {
            *mode = Some(_snake_num);
        }
    });
    

    let board_size = n as usize;
    if LOG_ENABLED {
        console::log_1(&format!("[INPUT] Board size: {}", n).into());
        console::log_1(&format!("[INPUT] Round: {}", round).into());
        console::log_1(&format!("[MY_SNAKE] raw_data: {:?}", my_snake).into());
        console::log_1(&format!("[OTHER_SNAKES] raw_data: {:?}", other_snakes).into());
        console::log_1(&format!("[FOODS] raw_data: {:?}", foods).into());
    }
    // 解析我方蛇
    let my_snake_coords = parse_snake_coords(&my_snake);
    if my_snake_coords.is_empty() {
        if LOG_ENABLED {
            console::log_1(&"[MY_SNAKE] Snake is dead, returning 0.".into());
        }
        return 0;
    }
    let my_length = my_snake_coords.len();
    if LOG_ENABLED {
        console::log_1(&format!("[MY_SNAKE] Parsed coordinates: {:?}", my_snake_coords).into());
    }
    // 解析其他蛇
    let (mut other_snakes_coords, mut other_heads) = parse_other_snakes(&other_snakes);
    if LOG_ENABLED {
        console::log_1(&format!("[OTHER_SNAKES] Parsed heads: {:?}", other_heads).into());
    }
    // 更新其他蛇历史轨迹（删除已死亡记录）
    update_trajectories(&other_snakes_coords);
    // 匹配当前敌蛇与上一回合敌蛇数据，获得 mapping 数组（一次性调用）
    let mapping = match_enemy_snakes(&other_snakes_coords);
    // 更新全局 LAST_ENEMY_SNAKES（只调用一次）
    update_last_enemy_snakes(&other_snakes_coords, &mapping);
    // 解析当前果子坐标
    let food_coords = parse_food_coords(&foods);
    if LOG_ENABLED {
        console::log_1(&format!("[FOODS] Parsed: {:?}", food_coords).into());
    }
    // 获取上一回合果子坐标，如果为空则用当前果子代替
    let mut last_food_coords = LAST_FOODS.with(|lf| lf.borrow().clone());
    if last_food_coords.is_empty() {
        last_food_coords = food_coords.clone();
    }
    if LOG_ENABLED {
        console::log_1(&format!("[LAST FOODS] {:?}", last_food_coords).into());
    }
    // 更新我方累计目标得分：如果我方蛇头出现在上一回合果子中，则加 1
    let my_round_score = compute_individual_score(my_snake_coords[0], &last_food_coords);
    MY_SCORE.with(|ms| {
        let mut ms = ms.borrow_mut();
        *ms += my_round_score;
    });
    let my_cumulative_score = MY_SCORE.with(|ms| *ms.borrow());
    if LOG_ENABLED {
        console::log_1(&format!("[MY SCORE] Cumulative: {}", my_cumulative_score).into());
    }
    // 更新并获得敌蛇累计目标得分，使用 mapping 保持固定索引
    let enemy_scores = update_and_get_enemy_scores_with_mapping(&other_heads, &last_food_coords, &mapping);
    if LOG_ENABLED {
        console::log_1(&format!("[ENEMY SCORES] {:?}", enemy_scores).into());
    }
    // 预测果子争夺情况
    let (contested_food, enemy_dist) = predict_contested_food(&food_coords, &other_heads);
    // 构建危险地图
    let dangerous = build_danger_map(&my_snake_coords, &other_snakes_coords, n, board_size);
    // 定义方向向量：0:上, 1:左, 2:下, 3:右
    let dir_vecs = [(0, 1), (-1, 0), (0, -1), (1, 0)];
    // 权重设置
     // 权重设置
     let score_weight: f32 = 1.0;
     let mut survival_weight: f32 = 3.0;
     let mut aggression_weight: f32 = 1.0;
 
     // 根据游戏模式调整权重
     GAME_MODE.with(|mode| {
         if let Some(mode) = *mode.borrow() {
             if mode == 3 {
                 survival_weight = 10.0; // 4蛇模式加大生存权重
             } 
             if _snake_num == 2 {
                 aggression_weight = 3.0; // 1v1模式且_snake_num为2时加大攻击权重
             }
         }
     });
 
    let mut best_dir: i32 = 0;
    let mut best_score: f32 = -1e9;
    for (dir_idx, (dx, dy)) in dir_vecs.iter().enumerate() {
        let head = my_snake_coords[0];
        if LOG_ENABLED {
            console::log_1(&format!("[DIRECTION {}] Current head: {:?}", dir_idx, head).into());
        }
        let new_head = (head.0 + dx, head.1 + dy);
        if new_head.0 < 1 || new_head.0 > n || new_head.1 < 1 || new_head.1 > n {
            if LOG_ENABLED {
                console::log_1(&format!("[DIRECTION {}] Skipped: wall collision at {:?}", dir_idx, new_head).into());
            }
            continue;
        }
        let new_idx = pos_to_index(new_head.0, new_head.1, board_size);
        if dangerous[new_idx] {
            let tail = *my_snake_coords.last().unwrap();
            let tail_idx = pos_to_index(tail.0, tail.1, board_size);
            let is_own_tail = new_idx == tail_idx;
            let fruit_at_new_head = food_coords.iter().any(|&(fx, fy)| fx == new_head.0 && fy == new_head.1);
            if !(is_own_tail && !fruit_at_new_head && my_length > 1) {
                if LOG_ENABLED {
                    console::log_1(&format!("[DIRECTION {}] Skipped: collision at {:?}", dir_idx, new_head).into());
                }
                continue;
            }
        }
        let new_body = simulate_move(&my_snake_coords, new_head, &food_coords);
        if LOG_ENABLED {
            console::log_1(&format!("[DIRECTION {}] Simulated body: {:?}", dir_idx, new_body).into());
        }
        let eat = food_coords.contains(&new_head);
        let food_score = compute_food_score(new_head, &food_coords, &contested_food, &enemy_dist, eat);
        let survival_score = compute_survival_score(new_head, &new_body, &other_snakes_coords, &other_heads, n, board_size, my_length);
        let aggression_score = compute_aggression_score(new_head, &other_heads, n, board_size, &dangerous, &last_food_coords, &mapping);
        if LOG_ENABLED {
            console::log_1(&format!("[DIRECTION {}] Food score: {}", dir_idx, food_score).into());
            console::log_1(&format!("[DIRECTION {}] Survival score: {}", dir_idx, survival_score).into());
            console::log_1(&format!("[DIRECTION {}] Aggression score: {}", dir_idx, aggression_score).into());
        }
        let total_score = food_score * score_weight + survival_score * survival_weight + aggression_score * aggression_weight;
        if LOG_ENABLED {
            console::log_1(&format!("[DIRECTION {}] Total score: {}", dir_idx, total_score).into());
        }
        if total_score > best_score {
            best_score = total_score;
            best_dir = dir_idx as i32;
        }
    }
    if LOG_ENABLED {
        console::log_1(&format!("[RESULT] Chosen direction: {}", best_dir).into());
    }
    // 更新 LAST_FOODS 为当前果子坐标，供下一回合使用
    let current_food_coords = parse_food_coords(&foods);
    LAST_FOODS.with(|lf| {
        *lf.borrow_mut() = current_food_coords;
    });
    best_dir
}
