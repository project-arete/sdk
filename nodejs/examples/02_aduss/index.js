//
// Counter Arete Application
//
// Connects to an aretehosting instance and increments a counter 
//


import { Client } from '../../index.js';

const NODE_ID = "node-id"
const NODE_NAME = "02-javascript-node"

const CONTEXT_ID = "context-id"
const CONTEXT_NAME = "02-javascript-context"

const CNS_PROFILE = "arete.sdk.example.javascript"

let client = new Client({
  protocol: 'wss:',
  host: 'fairfax.aretehosting.com',
  port: 443,
  // Paste Token from aretehosting Realm page
  token: 'TOKEN'
});
await client.waitForOpen(5000);

// Register with the control plane
let system = await client.system();
let node = await system.node(NODE_ID, NODE_NAME, false);
let context = await node.context(CONTEXT_ID, CONTEXT_NAME);


const sleep = (ms) => new Promise(resolve => setTimeout(resolve, ms));

const update = async (count) => {
  let consumer = await context.consumer(CNS_PROFILE);
  let date = Date.now()
  consumer.put('success', count)
  consumer.put('updated', date)
  console.log(`Success - ${count} at ${date}`);
}


// Do something
let counter = 0;
while (true) {
  await update(counter)
  await sleep(5000)
  counter++
}

