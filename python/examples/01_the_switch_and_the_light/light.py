import atexit
import ssl
import sys
import time
import RPi.GPIO as GPIO

sys.path.append('../../src')
from client import Client

APPNAME = "arete-sdk-01-light"
GPIO23 = 23
NODE_ID = 'onqXVczGoymQkFc3UN6qcM'

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

# Register this node with the control plane
client.add_system()
client.add_node(NODE_ID, APPNAME)
sys.stderr.write(f'Registered as node {NODE_ID} on Arete control plane\n')

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
