import { Gpio } from 'onoff';
import { Client } from '../../index.js';

const APPNAME = 'arete-sdk-01-light';
const GPIO23 = 535;
const NODE_ID = 'onqXVczGoymQkFc3UN6qcM';
const DESIRED_STATE_KEY =
  'cns/network/nodes/sri4FZUq2V7S4ik2PrG4pj/contexts/kMqdHs8ZcskdkCvf1VpfSZ/provider/padi.button/connections/geizaJngWyA1AL3Nhn5dzD/properties/sState';

// Configure pin for output
let pin = new Gpio(GPIO23, 'out');

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

// Detect future changes in desired state, and actualize it
client.on('update', (event) => {
  let value = event.keys[DESIRED_STATE_KEY];
  if (!value) {
    return;
  }
  const desired_state = value == '1';
  pin.writeSync(desired_state ? Gpio.HIGH : Gpio.LOW, (err) => {
    if (err) {
      throw err;
    }
  });
  console.log('New desired state is', desired_state ? 'ON' : 'OFF');
});

// Register shutdown handling
process.on('SIGINT', (_) => {
  console.log();
  console.log('Light service terminating');
  pin.unexport();
});

// Startup complete
console.log('Light service started');
