const Gpio = require('onoff').Gpio;
const CnsClient = require('./cns').Client;
const GPIO04 = 516;

// Configure pin for input
let pin = new Gpio(GPIO04, 'in', 'both', {debounceTimeout: 10});

// Read initial switch state
let state = pin.readSync();
console.log('Switch is initially', state ? 'ON' : 'OFF');

// Sync initial state with Arete
// TODO

// Startup complete
console.log('Switch service started')

// Detect future changes in state, and sync new switch state with Arete
pin.watch((err, state) => {
    if (err) {
        throw err;
    }
    console.log('Switch is now', state ? 'ON' : 'OFF');

    // Sync new state with Arete
    // TODO
});

// Register shutdown handling
process.on('SIGINT', _ => {
    console.log();
    console.log('Switch service terminating');
    pin.unexport();
});
