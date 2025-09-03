import atexit
import sys
import time
import RPi.GPIO as GPIO

sys.path.append('../../src')
from client import Client

GPIO23 = 23

# Configure pin for output
GPIO.setmode(GPIO.BCM)
GPIO.setup(GPIO23, GPIO.OUT)

# Connect to Arete control plane
client = Client.connect('wss://dashboard.test.cns.dev:443')
sys.stderr.write('Connected to Arete control plane\n')

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
