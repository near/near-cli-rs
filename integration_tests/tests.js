const util = require('node:util');
const exec = util.promisify(require('node:child_process').exec);

const creationAccountCommands = require('./test_cases/account_actions');

const script_path = "./target/release/near";

const testResults = {
  successful: 0,
  failed: 0,
};

async function runCliCommand(command, regex) {
  try {
    const { stdout, stderr } = await exec(command);

    const match = stderr.trim().match(regex);
    const result = match ? match[0] : null;

    return result;
  } catch (error) {
    const match = error.message.match(regex);
    const suggestedCommand = match ? match[0] : null;

    if (suggestedCommand) {
      return suggestedCommand;
    }

    return error;
  }
}

async function start() {
  for (let i = 0; i < creationAccountCommands.length; i++) {
    const jsCmd = creationAccountCommands[i].jsCmd;
    const suggestedCommandRegexPattern = creationAccountCommands[i].suggestedCommandRegexPattern;
    const expectedResult = creationAccountCommands[i].expectedResult;

    console.log(`▶️ Running the command: \n\t${jsCmd}`);
    const suggestedCommand = await runCliCommand(`${script_path} ${jsCmd}`, new RegExp(`${script_path} ${suggestedCommandRegexPattern}`));
    console.log(`\nSuggested command: \n\t${suggestedCommand}`);
  
    console.log("\nRunning the suggested command...");
    const result = (await runCliCommand(suggestedCommand, new RegExp(expectedResult)));
    console.log(`\t${result}`);
  
    if (result === expectedResult) {
      console.log("\n✅ Test passed");
      testResults.successful += 1;
    } else {
      console.error("❌ Test failed");
      testResults.failed += 1;
    }
    console.log("\n---\n");
  }

  console.log('Test stats:\n');
  console.log('✅ Successful: ', testResults.successful);
  console.log('❌ Failed: ', testResults.failed);
}

start()
