import atexit
import ssl
import sys
import time
import RPi.GPIO as GPIO
from threading import Thread
from time import sleep

sys.path.append('../../src')
from client import Client

GPIO04 = 4

NODE_ID = 'ozr9fZbU8i7hMdjEjuTS2o'
NODE_NAME = 'arete-sdk-01-switch'

CONTEXT_ID = 'uRLoYsXEY7nsbs9fRdjM8A'
CONTEXT_NAME = 'Building 23, Office 41-B'

PADI_LIGHT_PROFILE = 'padi.light'

# Configure pin for input
GPIO.setmode(GPIO.BCM)
GPIO.setup(GPIO04, GPIO.IN)

# Connect to Arete control plane
ssl_context = ssl.SSLContext(protocol=ssl.PROTOCOL_TLS_CLIENT)
ssl_context.check_hostname = False
ssl_context.verify_mode = ssl.CERT_NONE
client = Client.connect('wss://dashboard.test.cns.dev:443', ssl=ssl_context)
client.wait_for_open()
sys.stderr.write('Connected to Arete control plane\n')

# Register this node and its context with the control plane
system = client.system()
node = system.node(NODE_ID, NODE_NAME, False)
sys.stderr.write(f'Registered as node {NODE_ID}\n')
context = node.context(CONTEXT_ID, CONTEXT_NAME)
sys.stderr.write(f'Registered context {CONTEXT_ID} for node {NODE_ID}\n')

# Register as provider of state for the "padi.light" profile
client.add_provider(NODE_ID, CONTEXT_ID, PADI_LIGHT_PROFILE);
sys.stderr.write(f'Registered as provider of state for {PADI_LIGHT_PROFILE} profile for context {CONTEXT_ID}\n')

# Read initial switch state, and sync it with Arete
state = GPIO.input(GPIO04) == 0
client.put_property(NODE_ID, CONTEXT_ID, PADI_LIGHT_PROFILE, "sOut", '1' if state else '0')
sys.stderr.write('Switch is initially {}\n'.format('ON' if state else 'OFF'))

# Detect initial desired state, plus future changes to desired state, and try to actualize it
last_state = state
def detect_change(channel):
    global last_state
    while True:
        state = GPIO.input(GPIO04) == 0
        if state != last_state:
            client.put_property(NODE_ID, CONTEXT_ID, PADI_LIGHT_PROFILE, "sOut", '1' if state else '0')
            sys.stderr.write('Switch is now {}\n'.format('ON' if state else 'OFF'))
            last_state = state
        else:
            sleep(0.1)
change_detector = Thread(target=detect_change, args=(client, ))
change_detector.daemon = True
change_detector.start()

# Register shutdown handling
@atexit.register
def cleanup():
    GPIO.cleanup()

# Startup complete
sys.stderr.write('Switch service started\n')
while True:
    time.sleep(0.1)
