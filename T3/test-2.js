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
  
  // 初始化统计结果
  const totalResults = {
      wins: Array(gameState.snake_num).fill(0),
      totalScores: Array(gameState.snake_num).fill(0),
      totalTime: Array(gameState.snake_num).fill(0)
  };
  
  const totalGames = 10000;
  
  for (let gameCount = 0; gameCount < totalGames; gameCount++) {
      // 确保seed是有效的数字
      const currentSeed = Number(CUSTOM_SEED) + gameCount;
      if (isNaN(currentSeed)) {
          console.error(`Invalid seed value: ${CUSTOM_SEED}`);
          process.exit(1);
      }
      
      // 初始化游戏状态
      gameState = initializeGameState(TEST_MODE, snakeModules, currentSeed);
      
      while (!isGameOver(gameState)) {
        
        try {
          // Process one turn and get any messages
          const { gameState: newGameState, messages } = processGameTurn(gameState);
          gameState = newGameState;
          
          // Display any warnings or errors
          if (messages && messages.warnings.length > 0) {
            console.warn("Warnings in this turn:");
            messages.warnings.forEach(warning => console.warn(`- ${warning}`));
          }
          
          if (messages && messages.errors.length > 0) {
            console.error("Errors in this turn:");
            messages.errors.forEach(error => console.error(`- ${error}`));
          }
    
          // If necessary, output the intermediate state through the following lines
          // console.log(`\nRound ${gameState.round + 1}/${gameState.max_rounds}:`);
          // console.log("Current board state:");
          // printBoardState(gameState);
        } catch (error) {
          console.error("Game error:", error);
          process.exit(1);
        }
      }
      
      // Display functions
      function printBoardState(gameState) {
        const { alive, snake_num, scores } = gameState;
        console.log(`Living snakes ${alive.filter(Boolean).length}/${snake_num}: ${alive.join(', ')}`);
        console.log(`Scores: ${scores.join(', ')}`);
      }
      
      // Get the final result
      const finalResults = getFinalResults(gameState);
      
      // 创建排序索引数组 [0,1,2,...]，按分数降序、时间升序排序
      const sortedIndices = Array.from({length: gameState.snake_num}, (_, i) => i)
        .sort((a, b) => {
          if (finalResults.scores[b] !== finalResults.scores[a]) {
            return finalResults.scores[b] - finalResults.scores[a]; // 分数高的在前
          }
          return finalResults.time[a] - finalResults.time[b]; // 同分则耗时少的在前
        });
  
      // 更新统计结果
      const winnerIndex = sortedIndices[0];
      totalResults.wins[winnerIndex]++;
      finalResults.scores.forEach((score, index) => totalResults.totalScores[index] += score);
      finalResults.time.forEach((time, index) => totalResults.totalTime[index] += time);
  }
  
  // 输出最终统计结果
  console.log("\n=== 10000 GAMES SUMMARY ===");
  console.log("Total wins:");
  
  // 按胜场数排序
  const sortedWinners = Array.from({length: gameState.snake_num}, (_, i) => i)
      .sort((a, b) => totalResults.wins[b] - totalResults.wins[a]);
  
  for (const snakeIndex of sortedWinners) {
      console.log(`Snake ${snakeIndex + 1}: ${totalResults.wins[snakeIndex]} wins` +
          ` (avg score: ${(totalResults.totalScores[snakeIndex] / totalGames).toFixed(2)},` +
          ` avg time: ${(totalResults.totalTime[snakeIndex] / totalGames).toFixed(3)}ms)`);
  }