import { Gpio } from 'onoff';
import { Client } from '../../index.js';

const GPIO04 = 516;
const DESIRED_STATE_KEY =
  'cns/network/nodes/sri4FZUq2V7S4ik2PrG4pj/contexts/kMqdHs8ZcskdkCvf1VpfSZ/provider/padi.button/connections/geizaJngWyA1AL3Nhn5dzD/properties/sState';

const NODE_ID = 'ozr9fZbU8i7hMdjEjuTS2o';
const NODE_NAME = 'arete-sdk-01-switch';

const CONTEXT_ID = 'uRLoYsXEY7nsbs9fRdjM8A';
const CONTEXT_NAME = 'Building 23, Office 41-B';

const PADI_LIGHT_PROFILE = 'padi.light';

// Configure pin for input
let pin = new Gpio(GPIO04, 'in', 'both', { debounceTimeout: 10 });

// Connect to Arete control plane
let client = new Client({
  protocol: 'wss:',
  host: 'dashboard.test.cns.dev',
  port: 443,
});
await client.waitForOpen(5000);
console.log('Connected to Arete control plane');

// Register this node and its context with the control plane
await client.addSystem();
await client.addNode(NODE_ID, NODE_NAME, false);
console.log(`Registered as node ${NODE_ID} on Arete control plane`);
await client.addContext(NODE_ID, CONTEXT_ID, CONTEXT_NAME);
console.log(
  `Registered context ${CONTEXT_ID} for node ${NODE_ID} on Arete control plane`,
);

// Register the "padi.light" profile with the context
await client.addProfile(NODE_ID, CONTEXT_ID, PADI_LIGHT_PROFILE)
console.log(
    `Registered profile ${PADI_LIGHT_PROFILE} for context ${CONTEXT_ID} on Arete control plane`,
);

// Read initial switch state, and sync it with Arete
let state = pin.readSync();
client.put(DESIRED_STATE_KEY, state ? '1' : '0');
console.log('Switch is initially', state ? 'ON' : 'OFF');

// Detect future changes in switch state, and sync it with Arete
pin.watch((err, state) => {
  if (err) {
    throw err;
  }
  client.put(DESIRED_STATE_KEY, state ? '1' : '0');
  console.log('Switch is now', state ? 'ON' : 'OFF');
});

// Register shutdown handling
process.on('SIGINT', (_) => {
  console.log();
  console.log('Switch service terminating');
  pin.unexport();
});

// Startup complete
console.log('Switch service started');
