const timestamp = Date.now();

const testaccount1 = `test-ac-${timestamp}-1.testnet`;

module.exports = [
  // Creating a pre-funded account
  {
    jsCmd: `create-account ${testaccount1} --useFaucet`,
    expectedResult: `New account <${testaccount1}> created successfully.`,
  },
  // Getting a list of keys
  {
    jsCmd: `list-keys ${testaccount1}`,
    expectedResult: 'ed25519\:.* \| full access',
    isNeedToWaitForNextBlock: true,
  },
  // Adding public key to account
  {
    jsCmd: `add-key ${testaccount1} "78MziB9aTNsu19MHHVrfWy762S5mAqXgCB6Vgvrv9uGV"`,
    expectedResult: `Added access key = ed25519\:78MziB9aTNsu19MHHVrfWy762S5mAqXgCB6Vgvrv9uGV to ${testaccount1}.`,
  },
  // Adding function call key
  {
    jsCmd: `add-key ${testaccount1} "ed25519:DReZmNmnGhpsYcCFFeYgPsJ9YCm9xH16GGujCPe3KQEq" --contractId multichain-testnet-2.testnet --allowance 1`,
    expectedResult: `Added access key = ed25519\:DReZmNmnGhpsYcCFFeYgPsJ9YCm9xH16GGujCPe3KQEq to ${testaccount1}.`,
  },
  // Failing while adding a key that already exists
  {
    jsCmd: `add-key ${testaccount1} "78MziB9aTNsu19MHHVrfWy762S5mAqXgCB6Vgvrv9uGV"`,
    expectedResult: `Public key <ed25519\:78MziB9aTNsu19MHHVrfWy762S5mAqXgCB6Vgvrv9uGV> is already used for an existing account ID <${testaccount1}>.`,
  },
  // Deleting a key
  {
    jsCmd: `delete-key ${testaccount1} "78MziB9aTNsu19MHHVrfWy762S5mAqXgCB6Vgvrv9uGV"`,
    expectedResult: `Access key <ed25519\:78MziB9aTNsu19MHHVrfWy762S5mAqXgCB6Vgvrv9uGV> for account <${testaccount1}> has been successfully deleted.`,
  },
];