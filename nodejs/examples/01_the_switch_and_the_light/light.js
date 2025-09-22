import { Gpio } from 'onoff';
import { Client } from '../../index.js';

const GPIO23 = 535;

const CONTEXT_ID = 'uRLoYsXEY7nsbs9fRdjM8A';
const CONTEXT_NAME = 'Building 23, Office 41-B';

const NODE_ID = 'onqXVczGoymQkFc3UN6qcM';
const NODE_NAME = 'arete-sdk-01-light';

const PADI_LIGHT_PROFILE = 'padi.light';

// Connect to Arete control plane
let client = new Client({
  protocol: 'wss:',
  host: 'dashboard.test.cns.dev',
  port: 443,
});
await client.waitForOpen(5000);
console.log('Connected to Arete control plane');

// Register with the control plane
let system = await client.system();
let node = await system.node(NODE_ID, NODE_NAME, false);
let context = await node.context(CONTEXT_ID, CONTEXT_NAME);

// Read initial actual state of the light, and sync it with Arete
let pin = new Gpio(GPIO23, 'in', 'both', { debounceTimeout: 10 });
let state = pin.readSync();
let consumer = await context.consumer(PADI_LIGHT_PROFILE);
consumer.put('cState', state ? '1' : '0');
console.log('Light is initially', state ? 'ON' : 'OFF');

// Detect initial desired state, plus future changes to desired state, and try to actualize it
consumer.watch((event) => {
  if (event.property == 'sOut') {
    const desiredState = event.value == '1';
    let pin = new Gpio(GPIO23, 'out');
    pin.writeSync(desiredState ? Gpio.HIGH : Gpio.LOW, (err) => {
      if (err) {
        throw err;
      }
    });
    consumer.put('cState', desiredState ? '1' : '0');
    console.log('Light is now', desiredState ? 'ON' : 'OFF');
  }
});

// Register shutdown handling
process.on('SIGINT', (_) => {
  console.log();
  console.log('Light service terminating');
  pin.unexport();
});

// Startup complete
console.log('Light service started');
