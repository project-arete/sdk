import atexit
import ssl
import sys
import time
import RPi.GPIO as GPIO

sys.path.append('../../src/arete_sdk')
from client import Client

GPIO23 = 23

NODE_ID = 'onqXVczGoymQkFc3UN6qcM'
NODE_NAME = 'arete-sdk-01-light'

CONTEXT_ID = 'uRLoYsXEY7nsbs9fRdjM8A'
CONTEXT_NAME = 'Building 23, Office 41-B'

PADI_LIGHT_PROFILE = 'padi.light'

# Configure pin for output
GPIO.setmode(GPIO.BCM)

# Connect to Arete control plane
ssl_context = ssl.SSLContext(protocol=ssl.PROTOCOL_TLS_CLIENT)
ssl_context.check_hostname = False
ssl_context.verify_mode = ssl.CERT_NONE
client = Client.connect('wss://dashboard.test.cns.dev:443', ssl=ssl_context)
client.wait_for_open()
sys.stderr.write('Connected to Arete control plane\n')

# Register with the control plane
system = client.system()
node = system.node(NODE_ID, NODE_NAME)
context = node.context(CONTEXT_ID, CONTEXT_NAME)

# Read initial actual state of the light, and sync it with Arete
GPIO.setup(GPIO23, GPIO.IN)
state = GPIO.input(GPIO23) == 0
consumer = context.consumer(PADI_LIGHT_PROFILE)
consumer.put("cState", '1' if state else '0')
sys.stderr.write('Light is initially {}\n'.format('ON' if state else 'OFF'))


# Detect future changes to desired state, and try to actualize it
def detect_change(event):
    if event['property'] == 'sOut':
        desired_state = event['value'] == '1'
        GPIO.setup(GPIO23, GPIO.OUT)
        GPIO.output(GPIO23, desired_state)
        consumer.put("cState", '1' if desired_state else '0')
        sys.stderr.write('Light is now {}\n'.format('ON' if desired_state else 'OFF'))


consumer.watch(detect_change)


# Register shutdown handling
@atexit.register
def cleanup():
    GPIO.cleanup()


# Startup complete
sys.stderr.write('Light service started\n')
while True:
    time.sleep(0.1)
