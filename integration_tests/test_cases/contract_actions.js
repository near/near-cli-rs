const timestamp = Date.now();

const guestBookContract = `test-ac-${timestamp}-1.testnet`;
const fungibleTokenContract1 = `test-ac-${timestamp}-2.testnet`;
const fungibleTokenContract2 = `test-ac-${timestamp}-3.testnet`;
const guestBookContractPath = './integration_tests/res/guest_book.wasm';
const fungibleTokenContractPath = './integration_tests/res/fungible_token.wasm';

module.exports = [
  // Creating a pre-funded account
  {
    jsCmd: `create-account ${guestBookContract} --useFaucet`,
    expectedResult: `New account <${guestBookContract}> created successfully.`,
  },
  // Deploying a contract without init method
  {
    jsCmd: `deploy ${guestBookContract} ${guestBookContractPath}`,
    expectedResult: 'Contract code has been successfully deployed.',
  },
  // Calling a function with base64 arguments
  {
    jsCmd: `call ${guestBookContract} addMessage --base64 'eyJ0ZXh0IjoiSlNPTiJ9' --accountId ${guestBookContract}`,
    expectedResult: `The \"addMessage\" call to <${guestBookContract}> on behalf of <${guestBookContract}> succeeded.`,
  },
  // Viewing a result of calling
  {
    jsCmd: `view ${guestBookContract} getMessages '{}'`,
    expectedResult: `\"sender\": \"${guestBookContract}\",\n    \"text\": \"JSON\"`,
    isNeedToWaitForNextBlock: true,
  },
  // Calling a function with json arguments
  {
    jsCmd: `call ${guestBookContract} addMessage '{"text":"BASE64"}' --accountId ${guestBookContract}`,
    expectedResult: `The \"addMessage\" call to <${guestBookContract}> on behalf of <${guestBookContract}> succeeded.`,
  },
  // Viewing a result of calling
  {
    jsCmd: `view ${guestBookContract} getMessages '{}'`,
    expectedResult: `\"sender\": \"${guestBookContract}\",\n    \"text\": \"BASE64\"`,
    isNeedToWaitForNextBlock: true,
  },
  // Creating a pre-funded account
  {
    jsCmd: `create-account ${fungibleTokenContract1} --useFaucet`,
    expectedResult: `New account <${fungibleTokenContract1}> created successfully.`,
  },
  // Deploying a contract which has to be initialized without init method
  {
    jsCmd: `deploy ${fungibleTokenContract1} ${fungibleTokenContractPath}`,
    expectedResult: 'Contract code has been successfully deployed.',
  },
  // Trying to get balance (should fail)
  {
    jsCmd: `view ${fungibleTokenContract1} get_balance '{"owner_id":"something.testnet"}'`,
    expectedResult: 'Fun token should be initialized before usage',
    isNeedToWaitForNextBlock: true,
  },
  // Creating a pre-funded account for fungible token contract
  {
    jsCmd: `create-account ${fungibleTokenContract2} --useFaucet`,
    expectedResult: `New account <${fungibleTokenContract2}> created successfully.`,
  },
  // Deploying a contract which has to be initialized without init method
  {
    jsCmd: `deploy ${fungibleTokenContract2} ${fungibleTokenContractPath} --initFunction new --initArgs '{"owner_id":"${fungibleTokenContract2}","total_supply":"1000000"}'`,
    expectedResult: 'Contract code has been successfully deployed.',
    isNeedToWaitForNextBlock: true,
  },
  // Trying to get balance (should work)
  {
    jsCmd: `view ${fungibleTokenContract2} get_balance '{"owner_id":"${fungibleTokenContract2}"}'`,
    expectedResult: '\"1000000\"',
    isNeedToWaitForNextBlock: true,
  },
];