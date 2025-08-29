import atexit
import sys
import time
import RPi.GPIO as GPIO

GPIO23 = 23

# Configure pin for output
GPIO.setmode(GPIO.BCM)
GPIO.setup(GPIO23, GPIO.OUT)

# Connect to Arete control plane
# TODO

# Detect future changes in desired state, and actualize it
# TODO

# Register shutdown handling
@atexit.register
def cleanup():
    GPIO.cleanup()

# Startup complete
sys.stderr.write('Light service started\n')
while True:
    time.sleep(0.1)
