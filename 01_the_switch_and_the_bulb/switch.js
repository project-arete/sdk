const Gpio = require('onoff').Gpio;
const GPIO_04 = 516;

// Configure pin for reads
let pin = new Gpio(GPIO_04, 'in', 'both');
console.log('Switch service started')

// Read initial switch state
let state = pin.readSync();
console.log('Switch is initially', state ? 'ON' : 'OFF');

// Sync initial state with Arete
// TODO

// Detect future changes in state, and sync new switch state with Arete
pin.watch((err, state) => {
    if (err) {
        throw err;
    }
    console.log('Switch is now', state ? 'ON' : 'OFF');

    // Sync new state with Arete
    // TODO
});

process.on('SIGINT', _ => {
    console.log();
    console.log('Switch service terminating');
    pin.unexport();
});
