import atexit
import sys
import time
import RPi.GPIO as GPIO

GPIO04 = 4

# Configure pin for input
GPIO.setmode(GPIO.BCM)
GPIO.setup(GPIO04, GPIO.IN)

# Connect to Arete control plane
# TODO
# sys.stderr.write('Connected to Arete control plane\n')

# Read initial switch state, and sync it with Arete
state = GPIO.input(GPIO04) == 0
#client.put(DESIRED_STATE_KEY, state ? '1' : '0');
sys.stderr.write('Switch is initially {}\n'.format('ON' if state else 'OFF'))

# Detect future changes in switch state, and sync it with Arete
def on_change(channel):
    state = GPIO.input(GPIO04) == 0
    #client.put(DESIRED_STATE_KEY, state ? '1' : '0');
    sys.stderr.write('Switch is now {}\n'.format('ON' if state else 'OFF'))
GPIO.add_event_detect(GPIO04, GPIO.BOTH, callback=on_change)

# Register shutdown handling
@atexit.register
def cleanup():
    GPIO.cleanup()

# Startup complete
sys.stderr.write('Switch service started\n')
while True:
    time.sleep(0.1)
