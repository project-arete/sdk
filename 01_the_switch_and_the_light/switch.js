import { Gpio } from 'onoff';
import { Client } from './cns.js';

const GPIO04 = 516;
const DESIRED_STATE_KEY = 'cns/network/nodes/sri4FZUq2V7S4ik2PrG4pj/contexts/kMqdHs8ZcskdkCvf1VpfSZ/provider/padi.button/connections/geizaJngWyA1AL3Nhn5dzD/properties/sState';

// Configure pin for input
let pin = new Gpio(GPIO04, 'in', 'both', {debounceTimeout: 10});

// Connect to Arete control plane
let client = new Client({protocol:'wss:', host:'dashboard.test.cns.dev', port:443});
await client.waitForOpen(5000);
console.log('Connected to Arete control plane');

// Read initial switch state
let state = pin.readSync();
console.log('Switch is initially', state ? 'ON' : 'OFF');

// Sync initial state with Arete
client.put(DESIRED_STATE_KEY, state ? '1' : '0');

// Startup complete
console.log('Switch service started')

// Detect future changes in state, and sync new switch state with Arete
pin.watch((err, state) => {
    if (err) {
        throw err;
    }
    console.log('Switch is now', state ? 'ON' : 'OFF');

    // Sync new state with Arete
    client.put(DESIRED_STATE_KEY, state ? '1' : '0');
});

// Register shutdown handling
process.on('SIGINT', _ => {
    console.log();
    console.log('Switch service terminating');
    client.close();
    pin.unexport();
});
