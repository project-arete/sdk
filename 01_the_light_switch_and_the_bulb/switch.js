const Gpio = require('onoff').Gpio;

console.log("Switch service started")

// Read initial switch state
let pin = new Gpio(4, 'in', 'both');
let state = pin.readSync();
console.log('Switch is ' + state ? 'ON' : 'OFF');

// Sync initial state with Arete
// TODO

// Detect future changes in state, and sync new switch state with Arete
pin.watch((err, state) => {
    console.log('Switch is now ' + state ? 'ON' : 'OFF');

    // Sync new state with Arete
    // TODO
});

console.log("Switch service terminating");
