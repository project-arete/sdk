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
DESIRED_STATE_KEY = 'cns/network/nodes/sri4FZUq2V7S4ik2PrG4pj/contexts/kMqdHs8ZcskdkCvf1VpfSZ/provider/padi.button/connections/geizaJngWyA1AL3Nhn5dzD/properties/sState'

# Configure pin for input
GPIO.setmode(GPIO.BCM)
GPIO.setup(GPIO04, GPIO.IN)

# Connect to Arete control plane
ssl_context = ssl.SSLContext()
ssl_context.verify_mode = ssl.CERT_NONE
ssl_context.check_hostname = False
client = Client.connect('wss://dashboard.test.cns.dev:443', ssl=ssl_context)
sys.stderr.write('Connected to Arete control plane\n')

# Read initial switch state, and sync it with Arete
state = GPIO.input(GPIO04) == 0
client.put(DESIRED_STATE_KEY, '1' if state else '0')
sys.stderr.write('Switch is initially {}\n'.format('ON' if state else 'OFF'))

# Detect future changes in switch state, and sync it with Arete
last_state = state
def detect_change(channel):
    global last_state
    while True:
        state = GPIO.input(GPIO04) == 0
        if state != last_state:
            client.put(DESIRED_STATE_KEY, '1' if state else '0')
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
