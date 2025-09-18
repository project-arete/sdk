import { Gpio } from 'onoff';
import { Client } from '../../index.js';

const GPIO23 = 535;

const CONTEXT_ID = 'uRLoYsXEY7nsbs9fRdjM8A';
const CONTEXT_NAME = 'Building 23, Office 41-B';

const NODE_ID = 'onqXVczGoymQkFc3UN6qcM';
const NODE_NAME = 'arete-sdk-01-light';

const PADI_LIGHT_PROFILE = 'padi.light';

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

// Register this node and its context with the control plane
let system = await client.system();
let node = await system.node(NODE_ID, NODE_NAME, false);
console.log(`Registered as node ${NODE_ID}`);
let context = await node.context(CONTEXT_ID, CONTEXT_NAME);
console.log(`Registered context ${CONTEXT_ID} for node ${NODE_ID}`);

// Register as a consumer of state for the "padi.light" profile
let consumer = await context.consumer(PADI_LIGHT_PROFILE);
console.log(
  `Registered as a consumer of state for ${PADI_LIGHT_PROFILE} profile for context ${CONTEXT_ID}`,
);

// Detect initial desired state, plus future changes to desired state, and try to actualize it
consumer.watch((event) => {
  const desiredState = event.value == '1';
  pin.writeSync(desiredState ? Gpio.HIGH : Gpio.LOW, (err) => {
    if (err) {
      throw err;
    }
  });
  console.log('Light is now', desiredState ? 'ON' : 'OFF');
});

// Register shutdown handling
process.on('SIGINT', (_) => {
  console.log();
  console.log('Light service terminating');
  pin.unexport();
});

// Startup complete
console.log('Light service started');
