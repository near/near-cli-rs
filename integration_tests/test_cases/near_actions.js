const timestamp = Date.now();

const testaccount1 = `test-ac-${timestamp}-1.testnet`;
const testaccount2 = `test-ac-${timestamp}-2.testnet`;

module.exports = [
  // Creating a pre-funded account
  {
    jsCmd: `create-account ${testaccount1} --useFaucet`,
    expectedResult: `New account <${testaccount1}> created successfully.`,
  },
  // Creating a pre-funded account
  {
    jsCmd: `create-account ${testaccount2} --useFaucet`,
    expectedResult: `New account <${testaccount2}> created successfully.`,
  },
  // Sending near
  {
    jsCmd: `send-near ${testaccount1} ${testaccount2} 1`,
    expectedResult: `<${testaccount1}> has transferred 1 NEAR to <${testaccount2}> successfully`,
    isNeedToWaitForNextBlock: true,
  },
];