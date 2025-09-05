import { Gpio } from 'onoff';
import { Client } from '../../index.js';

const APPNAME = 'arete-sdk-01-switch';
const GPIO04 = 516;
const NODE_ID = 'ozr9fZbU8i7hMdjEjuTS2o';
const DESIRED_STATE_KEY =
  'cns/network/nodes/sri4FZUq2V7S4ik2PrG4pj/contexts/kMqdHs8ZcskdkCvf1VpfSZ/provider/padi.button/connections/geizaJngWyA1AL3Nhn5dzD/properties/sState';

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

// Register this node with the control plane
await client.addSystem();
await client.addNode(NODE_ID, APPNAME, false);
console.log(`Registered as node ${NODE_ID} on Arete control plane`);

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
