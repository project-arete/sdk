import { Gpio } from 'onoff';
import { Client } from './cns.js';

const GPIO23 = 535;

// Configure pin for output
let pin = new Gpio(GPIO23, 'out');

// Connect to Arete control plane
let client = new Client({protocol:'wss:', host:'dashboard.test.cns.dev', port:443});
await client.waitForOpen(5000);
console.log('Connected to Arete control plane');

// Read initial desired state
let desired_state = Math.random() < 0.5; // TODO read from Arete
console.log('Desired state is initially', desired_state ? 'ON' : 'OFF');

// Actualize initial desired state
pin.write(desired_state ? Gpio.HIGH : Gpio.LOW, err => {
    if (err) {
        throw err;
    }
});

// Startup complete
console.log('Light service started')

// Detect future changes in desired state
setInterval(on_change, 1000); // TODO watch the Arete control plane instead of faking changes with a timer

// Register shutdown handling
process.on('SIGINT', _ => {
    console.log();
    console.log('Light service terminating');
    client.close();
    pin.unexport();
});

let last_desired_state = false; // TODO remove
function on_change() {
    let desired_state = Math.random() < 0.5; // TODO read from Arete
    if (desired_state == last_desired_state) {
        return;
    }
    console.log('New desired state is', desired_state ? 'ON' : 'OFF');

    // Actualize new desired state
    pin.writeSync(desired_state ? Gpio.HIGH : Gpio.LOW, err => {
        if (err) {
            throw err;
        }
    });
    last_desired_state = desired_state;
}
