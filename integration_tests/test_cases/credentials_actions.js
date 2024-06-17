const timestamp = Date.now();

const testaccount1 = `test-ac-${timestamp}-1.testnet`;

module.exports = [
  // Testing generating-key: new key
  {
    jsCmd: `generate-key new-account-name --networkId testnet`,
    expectedResult: `The file ".*\/[a-f0-9]{64}\.json" was saved successfully`,
  },
  // Testing generating-key: key for account already exists
  {
    jsCmd: `generate-key new-account-name --networkId testnet`,
    expectedResult: `The file ".*\/[a-f0-9]{64}\.json" was saved successfully`,
  }
];