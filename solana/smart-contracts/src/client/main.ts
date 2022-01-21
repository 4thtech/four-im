/**
 * Documents
 */

import {
  establishConnection,
  establishPayer,
  checkProgram,
  sendMessages,
  readConversations,
} from './instantMessaging';

async function main() {
  console.log("Let's send some instant messages to a Solana account...");

  // Establish connection to the cluster
  await establishConnection();

  // Determine who pays for the fees
  await establishPayer();

  // Check if the program has been deployed
  await checkProgram();

  // Send messages to an account
  await sendMessages();

  // Read conversations from an account
  await readConversations();

  console.log('Success');
}

main().then(
  () => process.exit(),
  err => {
    console.error(err);
    process.exit(-1);
  },
);
