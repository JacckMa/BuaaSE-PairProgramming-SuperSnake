// Import game engine
import { 
  initializeGameState, 
  processGameTurn, 
  isGameOver, 
  getFinalResults 
} from './snake-engine.js';

// Import configuration
import {
  GAME_MODE,
  snakeModules,
  CUSTOM_SEED
} from './game-config.js';

// Test parameters
const TEST_MODE = GAME_MODE;

// Initialize game state
let gameState = initializeGameState(TEST_MODE, snakeModules, CUSTOM_SEED);

// Main simulation loop
console.log(`Starting ${TEST_MODE} mode with ${gameState.snake_num} snakes, board size ${gameState.n}x${gameState.n}, ${gameState.food_num} foods, ${gameState.max_rounds} rounds`);
console.log(`Game seed: 0x${gameState.seed.toString(16).padStart(16, '0')}`)

// 初始化统计数组
const firstPlaceCounts = new Map();
const secondPlaceCounts = new Map();
const totalWins = new Map();
let battleCount = 10000;

// 进行10000次对战
for (let i = 0; i < battleCount; i++) {
  try {
    // 使用显式类型转换确保类型一致
    const dynamicSeed = BigInt(CUSTOM_SEED) + BigInt(i);
    let gameState = initializeGameState(TEST_MODE, snakeModules, dynamicSeed);

    // 运行完整游戏
    while (!isGameOver(gameState)) {
      const { gameState: newGameState } = processGameTurn(gameState);
      gameState = newGameState;
    }

    // 获取最终结果并排序
    const finalResults = getFinalResults(gameState);
    const sorted = Array.from({length: gameState.snake_num}, (_, i) => i)
      .sort((a, b) => {
        if (finalResults.scores[b] !== finalResults.scores[a]) {
          return finalResults.scores[b] - finalResults.scores[a];
        }
        return finalResults.time[a] - finalResults.time[b];
      });

    // 记录排名
    if (sorted.length > 0) {
      const firstPlace = sorted[0] + 1;
      firstPlaceCounts.set(firstPlace, (firstPlaceCounts.get(firstPlace) || 0) + 1);
    }
    if (sorted.length > 1) {
      const secondPlace = sorted[1] + 1;
      secondPlaceCounts.set(secondPlace, (secondPlaceCounts.get(secondPlace) || 0) + 1);
    }

    // 记录总积分
    sorted.forEach((snakeIndex, rank) => {
      const key = snakeIndex + 1;
      totalWins.set(key, (totalWins.get(key) || 0) + (battleCount - rank));
    });

  } catch (error) {
    console.error(`Battle ${i + 1} failed:`, error);
  }
}

// 生成最终排名
const sortedResults = Array.from(totalWins.entries())
  .sort((a, b) => b[1] - a[1]);

// 输出最终统计
console.log("\n=== 10000 BATTLES FINAL RANKING ===");
console.log("First Place Counts:");
Array.from({length: gameState.snake_num}, (_, i) => i + 1).forEach(snake => {
  console.log(`Snake ${snake}: ${firstPlaceCounts.get(snake) || 0} times`);
});

console.log("\nSecond Place Counts:");
Array.from({length: gameState.snake_num}, (_, i) => i + 1).forEach(snake => {
  console.log(`Snake ${snake}: ${secondPlaceCounts.get(snake) || 0} times`);
});

console.log("\nTotal Points Ranking:");
sortedResults.forEach(([snake, score], index) => {
  console.log(`#${index + 1} Snake ${snake}: ${score} points`);
});

// Display functions
function printBoardState(gameState) {
  const { alive, snake_num, scores } = gameState;
  console.log(`Living snakes ${alive.filter(Boolean).length}/${snake_num}: ${alive.join(', ')}`);
  console.log(`Scores: ${scores.join(', ')}`);
}

// Get the final result
const finalResults = getFinalResults(gameState);

// Final results
console.log("\n=== FINAL RESULTS ===");
console.log(`Snake scores:`);
for (let i = 0; i < gameState.snake_num; i++) {
  console.log(`Snake ${i + 1}: ${finalResults.scores[i]} points${finalResults.alive[i] ? " (survived)" : " (died in round " + (finalResults.dead_round[i]) + ")"} spent ${finalResults.time[i].toFixed(3)}ms`);
}
