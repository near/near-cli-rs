const util = require('node:util');
const exec = util.promisify(require('node:child_process').exec);

const accountActions = require('./test_cases/account_actions');
const contractActions = require('./test_cases/contract_actions');
const keyActions = require('./test_cases/key_actions');
const credentialsActions = require('./test_cases/credentials_actions');
const nearActions = require('./test_cases/near_actions');

const testCases = [
  accountActions,
  contractActions, 
  keyActions,
  credentialsActions,
  nearActions
];

const script_path = "./target/release/near";

const testResults = {
  successful: 0,
  failed: 0,
};

function sleep(ms) {
  return new Promise(resolve => setTimeout(resolve, ms));
}

async function getSuggestedCommand(command) {
  try {
    await exec(command);
    throw new Error(`Command ${command} should have failed`);
  } catch (error) {
    // we actually expect this to run into an error
    const regex = new RegExp(`(    )(${script_path} .*)`);

    // replace here removes styling from the error message
    const match = error.message.replace(/\u001b\[.*?m/g, '').match(regex);
    
    const suggestedCommand = match ? match[2] : null;

    if (suggestedCommand) {
      return suggestedCommand;
    }

    return error;
  }
}

async function runSuggestedCommand(command, expectedResult, isNeedToWaitForNextBlock = false) {
  try {
    if (isNeedToWaitForNextBlock) {
      await sleep(2000);
    }
    const { stdout, stderr } = await exec(command);
    // console.log(stdout);
    // console.log(stderr);
    // console.log(expectedResult);
    const match = (stdout + stderr).trim().match(expectedResult);
    return match ? match[0] : result;
  } catch (error) {
    const match = error.message.match(expectedResult);
    return match ? match[0] : null;
  }
}

async function start() {
  for (j = 0; j < testCases.length; j++) {
    for (let i = 0; i < testCases[j].length; i++) {
      const { jsCmd, expectedResult, isNeedToWaitForNextBlock } = testCases[j][i];
  
      console.log(`▶️ Running the command: \n\t${jsCmd}`);
      const suggestedCommand = await getSuggestedCommand(`${script_path} ${jsCmd}`);
      console.log(`\nSuggested command: \n\t${suggestedCommand}`);
  
      console.log("\nRunning the suggested command...");
      const result = await runSuggestedCommand(suggestedCommand, expectedResult, isNeedToWaitForNextBlock);
      console.log(`\t${result}`);
  
      if (result) {
        console.log("\n✅ Test passed");
        testResults.successful += 1;
      } else {
        console.error("❌ Test failed");
        testResults.failed += 1;
      }
      console.log("\n---\n");
    }
  }

  console.log('Test stats:\n');
  console.log('✅ Successful: ', testResults.successful);
  console.log('❌ Failed: ', testResults.failed);
}

start();
