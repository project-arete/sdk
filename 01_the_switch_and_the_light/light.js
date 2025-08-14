const Gpio = require('onoff').Gpio;
const GPIO23 = 535;

// Configure pin for reads
let pin = new Gpio(GPIO23, 'out');
console.log('Light service started')

// Read initial desired state
let desired_state = Math.random() < 0.5; // TODO read from Arete
console.log('Desired state is initially', desired_state ? 'ON' : 'OFF');

// Set initial desired state
pin.write(desired_state ^ 1, err => {
    if (err) {
        throw err;
    }
});

// Detect future changes in desired state
setTimeout(on_change, 1000); // TODO watch the Arete control plane instead of faking changes with a timer

// Register shutdown handling
process.on('SIGINT', _ => {
    console.log();
    console.log('Light service terminating');
    pin.unexport();
});

let last_desired_state = false; // TODO remove
function on_change() {
    let desired_state = Math.random() < 0.5; // TODO read from Arete
    if (desired_state == last_desired_state) {
        return;
    }
    console.log('New desired state is', desired_state ? 'ON' : 'OFF');
    pin.write(desired_state ^ 1, err => {
        if (err) {
            throw err;
        }
    });
    last_desired_state = desired_state;
}
