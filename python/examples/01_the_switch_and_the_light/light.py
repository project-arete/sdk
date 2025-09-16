import atexit
import ssl
import sys
import time
import RPi.GPIO as GPIO

sys.path.append('../../src')
from client import Client

GPIO23 = 23

NODE_ID = 'onqXVczGoymQkFc3UN6qcM'
NODE_NAME = 'arete-sdk-01-light'

CONTEXT_ID = 'uRLoYsXEY7nsbs9fRdjM8A'
CONTEXT_NAME = 'Building 23, Office 41-B'

PADI_LIGHT_PROFILE = 'padi.light'

# Configure pin for output
GPIO.setmode(GPIO.BCM)
GPIO.setup(GPIO23, GPIO.OUT)

# Connect to Arete control plane
ssl_context = ssl.SSLContext(protocol=ssl.PROTOCOL_TLS_CLIENT)
ssl_context.check_hostname = False
ssl_context.verify_mode = ssl.CERT_NONE
client = Client.connect('wss://dashboard.test.cns.dev:443', ssl=ssl_context)
client.wait_for_open()
sys.stderr.write('Connected to Arete control plane\n')

# Register this node and its context with the control plane
system = client.system()
node = system.node(NODE_ID, NODE_NAME)
sys.stderr.write(f'Registered as node {NODE_ID}\n')
client.add_context(NODE_ID, CONTEXT_ID, CONTEXT_NAME)
sys.stderr.write(f'Registered context {CONTEXT_ID} for node {NODE_ID}\n')

# Register as a consumer of state for the "padi.light" profile
client.add_consumer(NODE_ID, CONTEXT_ID, PADI_LIGHT_PROFILE);
sys.stderr.write(f'Registered as consumer of state for {PADI_LIGHT_PROFILE} profile for context {CONTEXT_ID}\n')

# Detect initial desired state, plus future changes to desired state, and try to actualize it
# TODO(https://github.com/project-arete/sdk/issues/57)

# Register shutdown handling
@atexit.register
def cleanup():
    GPIO.cleanup()

# Startup complete
sys.stderr.write('Light service started\n')
while True:
    time.sleep(0.1)
