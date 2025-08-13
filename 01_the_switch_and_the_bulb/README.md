# The Switch and The Bulb

![PCB's](readme_intro.png)

This example consists of an IoT system containing 2 components: a switch, and a light bulb.

* A Switch Service reads the state of the GPIO4 pin, and conveys it to the Arete control plane as a "desired state".
* A Bulb Service reads the desired state from the Arete control plane, and tries to realize it as "actual state"
  by setting a GPIO pin on or off.

## Building

```shell
$ npm install
```

## Running

### Run the switch service

The switch service needs to run as root in order to access GPIO edge triggers.

```shell
$ sudo node switch.js 
Switch service started
Switch is initially ON
Switch is now OFF
Switch is now ON
...
```
