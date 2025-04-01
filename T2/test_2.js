import assert from "assert";
// Choose proper "import" depending on your PL.
// For example, for Rust build:
import { greedy_snake_move_barriers as greedySnakeMoveBarriers } from "./t2_rust/pkg/t2_rust.js";
// [Write your own "import" for other PLs.]

/**
 * Checker function that simulates the snake moving toward food.
 * Returns:
 *  >0 : number of turns taken to eat all food (success)
 *  -1 : snake out of range
 *  -2 : snake hit a barrier
 *  -3 : timeout (exceeded 200 turns)
 *  -4 : invalid direction returned
 *  -5 : unexpected direction in unreachable case
 *  In unreachable cases (access === 0), the function expects greedySnakeMoveBarriers to return -1 immediately.
 */
function greedy_snake_barriers_checker(initial_snake, food_num, foods, barriers, access) {
    if (initial_snake.length !== 8) throw "Invalid snake length";

    let current_snake = [...initial_snake];
    let current_foods = [...foods];
    const barriers_list = [];
    // Build barrier list from the barriers array (each barrier is 2 numbers; ignore pairs with -1)
    for (let i = 0; i < barriers.length; i += 2) {
        const x = barriers[i];
        const y = barriers[i + 1];
        if (x !== -1 && y !== -1) {
            barriers_list.push({ x, y });
        }
    }
    let turn = 1;

    while (turn <= 200) {
        const direction = greedySnakeMoveBarriers(current_snake, current_foods, barriers);

        if (access === 0) {
            // For unreachable cases, the algorithm should immediately return -1.
            if (direction !== -1) {
                return -5;
            } else {
                return 1;
            }
        }

        // Check for invalid direction
        if (direction < 0 || direction > 3) return -4;

        // Compute new snake position: snake moves by adding a new head and shifting body.
        let new_snake = [
            current_snake[0] + (direction === 3) - (direction === 1),
            current_snake[1] + (direction === 0) - (direction === 2),
            current_snake[0],
            current_snake[1],
            current_snake[2],
            current_snake[3],
            current_snake[4],
            current_snake[5],
        ];

        // Check out-of-bound condition (field is 8x8 with coordinates [1,8])
        if (new_snake[0] < 1 || new_snake[0] > 8 || new_snake[1] < 1 || new_snake[1] > 8) return -1;

        // Check if snake hits any barrier
        if (barriers_list.some(ob => ob.x === new_snake[0] && ob.y === new_snake[1])) return -2;

        // Check if snake eats food: food coordinate match with new head
        let ate_index = -1;
        for (let i = 0; i < current_foods.length; i += 2) {
            if (new_snake[0] === current_foods[i] && new_snake[1] === current_foods[i + 1]) {
                ate_index = i;
                break;
            }
        }

        if (ate_index !== -1) {
            current_foods.splice(ate_index, 2);
            food_num -= 1;
        }

        if (food_num === 0) {
            console.log("Total turn: " + turn);
            return turn;
        }

        current_snake = new_snake;
        turn++;
    }

    // Timeout: more than 200 turns without eating all food.
    return -3;
}

/* Helper functions for random test case generation */

// Returns a random integer in [min, max] (inclusive)
function randomInt(min, max) {
    return Math.floor(Math.random() * (max - min + 1)) + min;
}

/**
 * Generate a snake of 4 segments (8 numbers) in a straight line.
 * We choose a head position with enough margin and a random direction.
 */
function generateRandomSnake() {
    // To allow a straight body, choose head coordinates in [3,6]
    const head_x = randomInt(3, 6);
    const head_y = randomInt(3, 6);
    // Randomly choose one of the 4 directions for the snake body
    const directions = [
        { dx: 0, dy: -1 }, // body extends downward
        { dx: 0, dy: 1 },  // body extends upward
        { dx: -1, dy: 0 }, // body extends rightward
        { dx: 1, dy: 0 }   // body extends leftward
    ];
    const { dx, dy } = directions[randomInt(0, 3)];

    // Build the snake in a straight line (head then body segments)
    const second_x = head_x - dx;
    const second_y = head_y - dy;
    const third_x = second_x - dx;
    const third_y = second_y - dy;
    const fourth_x = third_x - dx;
    const fourth_y = third_y - dy;

    // Ensure all coordinates are within [1,8]
    const coords = [head_x, head_y, second_x, second_y, third_x, third_y, fourth_x, fourth_y];
    if (coords.some(c => c < 1 || c > 8)) {
        // Retry if out of bounds
        return generateRandomSnake();
    }
    return coords;
}

/**
 * Generate a food coordinate [x, y] that is not part of the snake.
 */
function generateRandomFood(snake) {
    let food;
    while (true) {
        const x = randomInt(1, 8);
        const y = randomInt(1, 8);
        // Check if food conflicts with any snake segment (all segments in snake)
        let conflict = false;
        for (let i = 0; i < snake.length; i += 2) {
            if (snake[i] === x && snake[i + 1] === y) {
                conflict = true;
                break;
            }
        }
        if (!conflict) {
            food = [x, y];
            break;
        }
    }
    return food;
}

/**
 * Check reachability using a simple BFS.
 * This mimics the logic of our snake function: obstacles include snake body (indices 2-5) and barriers.
 */
function isReachable(snake, food, barriers) {
    const obstacles = new Set();
    // Add snake body obstacles (exclude head and tail)
    for (let i = 2; i < snake.length - 2; i += 2) {
        obstacles.add(snake[i] + ',' + snake[i + 1]);
    }
    // Add barriers (ignoring pairs with -1)
    for (let i = 0; i < barriers.length; i += 2) {
        const bx = barriers[i], by = barriers[i + 1];
        if (bx !== -1 && by !== -1) {
            obstacles.add(bx + ',' + by);
        }
    }
    const start = { x: snake[0], y: snake[1] };
    const target = { x: food[0], y: food[1] };

    const queue = [start];
    const visited = new Set();
    visited.add(start.x + ',' + start.y);
    const moves = [ {dx: 0, dy: 1}, {dx: -1, dy: 0}, {dx: 0, dy: -1}, {dx: 1, dy: 0} ];
    while (queue.length > 0) {
        const {x, y} = queue.shift();
        if (x === target.x && y === target.y) return true;
        for (const {dx, dy} of moves) {
            const nx = x + dx, ny = y + dy;
            const key = nx + ',' + ny;
            if (nx >= 1 && nx <= 8 && ny >= 1 && ny <= 8 &&
                !obstacles.has(key) && !visited.has(key)) {
                visited.add(key);
                queue.push({x: nx, y: ny});
            }
        }
    }
    return false;
}

/**
 * Generate random barriers.
 * If reachable is true, we try to generate 12 barriers (24 numbers) that do not block the path from snake to food.
 * If reachable is false, we deliberately block all moves from the snake head.
 */
function generateRandomBarriers(snake, food, reachable) {
    const numBarriers = 12;
    let barriers = [];
    if (!reachable) {
        // Block all adjacent cells of the snake head
        const head_x = snake[0], head_y = snake[1];
        const moves = [ {x: head_x, y: head_y + 1},
                        {x: head_x - 1, y: head_y},
                        {x: head_x, y: head_y - 1},
                        {x: head_x + 1, y: head_y} ];
        for (const m of moves) {
            barriers.push(m.x, m.y);
        }
        // Fill the remaining barriers with -1 (indicating no barrier)
        while (barriers.length < numBarriers * 2) {
            barriers.push(-1, -1);
        }
        return barriers;
    } else {
        // Try generating random barriers until the food is reachable
        let attempts = 0;
        while (attempts < 1000) {
            barriers = [];
            const used = new Set();
            // Reserve snake and food positions so that barriers don't overlap them.
            for (let i = 0; i < snake.length; i += 2) {
                used.add(snake[i] + ',' + snake[i+1]);
            }
            used.add(food[0] + ',' + food[1]);

            for (let i = 0; i < numBarriers; i++) {
                let bx, by;
                let key;
                let trial = 0;
                do {
                    bx = randomInt(1, 8);
                    by = randomInt(1, 8);
                    key = bx + ',' + by;
                    trial++;
                    if (trial > 20) break;
                } while (used.has(key));
                // Mark barrier position as used to avoid duplicates.
                used.add(key);
                barriers.push(bx, by);
            }
            // If fewer than 12 barriers generated (shouldn't happen), fill the rest with -1.
            while (barriers.length < numBarriers * 2) {
                barriers.push(-1, -1);
            }
            if (isReachable(snake, food, barriers)) {
                return barriers;
            }
            attempts++;
        }
        // If no valid barrier configuration found after many attempts, fallback to no barriers.
        return Array(numBarriers * 2).fill(-1);
    }
}

/* ---------- Fixed test cases ---------- */

// Test case 1: reachable scenario with fixed parameters
assert.strictEqual(
    greedy_snake_barriers_checker(
        [4, 4, 4, 3, 4, 2, 4, 1],
        1,
        [4, 5],
        [5, 4, 8, 8, 8, 7, 8, 6, 8, 5, 8, 4, 8, 3, 8, 2, 8, 1, 7, 8, 7, 7, 7, 6],
        1
    ) > 0,
    true
);

// Test case 2: another reachable scenario with fixed parameters
assert.strictEqual(
    greedy_snake_barriers_checker(
        [1, 4, 1, 3, 1, 2, 1, 1],
        1,
        [5, 5],
        [2, 7, 2, 6, 3, 7, 3, 6, 4, 6, 5, 6, 6, 6, 7, 6, 4, 5, 4, 4, 4, 3, 5, 4],
        1
    ) > 0,
    true
);

// Test case 3: unreachable scenario with fixed parameters
assert.strictEqual(
    greedy_snake_barriers_checker(
        [1, 4, 1, 3, 1, 2, 1, 1],
        1,
        [1, 7],
        [2, 7, 2, 6, 3, 7, 3, 6, 4, 7, 4, 6, 5, 7, 5, 6, 1, 6, 6, 6, 7, 6, 8, 6],
        0
    ),
    1
);

/* ---------- Random test cases ---------- */

// Generate and test 5 random reachable cases.
for (let i = 0; i < 20000; i++) {
    const snake = generateRandomSnake();
    const food = generateRandomFood(snake);
    const barriers = generateRandomBarriers(snake, food, true);
    const result = greedy_snake_barriers_checker(snake, 1, food, barriers, 1);
    console.log(`Random Reachable Test ${i+1}:`);
    console.log("  Snake:    ", snake);
    console.log("  Food:     ", food);
    console.log("  Barriers: ", barriers);
    console.log("  Result:   ", result);
    // For reachable cases, we expect a positive number (number of turns taken)
    assert.strictEqual(result > 0, true);
}

// Generate and test 5 random unreachable cases.
for (let i = 0; i < 20000; i++) {
    const snake = generateRandomSnake();
    // Even if food is placed arbitrarily, we force unreachable by blocking adjacent cells.
    const food = generateRandomFood(snake);
    const barriers = generateRandomBarriers(snake, food, false);
    const result = greedy_snake_barriers_checker(snake, 1, food, barriers, 0);
    console.log(`Random Unreachable Test ${i+1}:`);
    console.log("  Snake:    ", snake);
    console.log("  Food:     ", food);
    console.log("  Barriers: ", barriers);
    console.log("  Result:   ", result);
    // For unreachable cases, our checker expects 1 (as per our unreachable convention)
    assert.strictEqual(result, 1);
}

console.log("🎉 You have passed all the tests provided.");
