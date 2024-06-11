const timestamp = Date.now();

const testaccount1 = `test-ac-${timestamp}-1.testnet`;
const testaccount2 = `test-ac-${timestamp}-2.testnet`;
const testaccount3 = `test-ac-${timestamp}-3.testnet`;
const testaccount4 = `test-ac-${timestamp}-4.testnet`;
const testaccount5 = `test-ac-${timestamp}-5.testnet`;
const testaccount6 = `test-ac-${timestamp}-6.testnet`;

const tooLongAccountName = 'x'.repeat(65);

module.exports = [
  // Creating a pre-funded account
  {
    jsCmd: `create-account ${testaccount1} --useFaucet`,
    expectedResult: `New account <${testaccount1}> created successfully.`,
  },
  // Creating a pre-funded account with a seedPhrase
  {
    jsCmd: `create-account ${testaccount2} --seedPhrase \"crisp clump stay mean dynamic become fashion mail bike disorder chronic sight\" --useFaucet`,
    expectedResult: `New account <${testaccount2}> created successfully.`,
  },
  // Creating a subaccount with a given public key & balance
  {
    jsCmd: `create-account sub.${testaccount1} --accountId ${testaccount1} --publicKey "78MziB9aTNsu19MHHVrfWy762S5mAqXgCB6Vgvrv9uGV" --initialBalance 0.1`,
    expectedResult: `New account <sub.${testaccount1}> has been successfully created.`,
  },
  // Creating an account funded by another account
  {
    jsCmd: `create-account ${testaccount3} --accountId ${testaccount1}`,
    expectedResult: `The "create_account" call to <testnet> on behalf of <${testaccount1}> succeeded.`,
  },
  // Creating zero-balance accounts
  {
    jsCmd: `create-account ${testaccount4} --accountId ${testaccount1} --initialBalance 0`,
    expectedResult: `The "create_account" call to <testnet> on behalf of <${testaccount1}> succeeded.`,
  },
  // Failing creating sub-account for another account
  {
    jsCmd: `create-account sub.${testaccount2} --accountId ${testaccount1}`,
    expectedResult: `Signer account <${testaccount1}> does not have permission to create account <sub.${testaccount2}>.`,
  },
  // Failing using a non-existing account to fund another
  {
    jsCmd: `create-account ${testaccount5} --accountId ${testaccount6}`,
    expectedResult: `Access key file for account <${testaccount6}> on network <testnet> not found!`,
  },
  // Failing creating a top level account with a short name
  {
    jsCmd: `create-account tooshortfortla --accountId ${testaccount1}`,
    expectedResult: 'Account <tooshortfortla> has <14> character count.',
  },
  // Failing creating a top level account with a tool long name
  {
    jsCmd: `create-account ${tooLongAccountName} --accountId ${testaccount1}`,
    expectedResult: 'the Account ID is too long',
  },
  // Failing creating an account in mainnet using Faucet
  {
    jsCmd: `create-account ${testaccount5} --useFaucet --networkId mainnet`,
    expectedResult: 'The <mainnet> network does not have a faucet',
  },
  // Failing creating an account when master account does not have enough balance
  {
    jsCmd: `create-account ${testaccount5} --accountId ${testaccount1} --initialBalance 100`,
    expectedResult: `Account <${testaccount1}> does not have enough balance`,
  },
  // Deleting an account
  {
    jsCmd: `delete-account ${testaccount3} ${testaccount1}`,
    expectedResult: `Account <${testaccount3}> has been successfully deleted.`,
  },
  // Deleting an account on mainnet
  {
    jsCmd: `delete-account ${testaccount3} ${testaccount1} --networkId mainnet`,
    expectedResult: `Access key file for account <${testaccount3}> on network <mainnet> not found!`,
  },
  // check that the key is the right one - do at the end, so the account info is updated
  {
    jsCmd: `keys ${testaccount2}`,
    expectedResult: `.*ed25519:GPnL8k4MV1hLccB5rkNiihVAEEmQX3BTDJnmW1T7ZDXG.*`,
  },
];