from threading import Thread
from websockets.sync.client import connect
import json
import ssl
from system_id import get_system_id

class Client:
    websocket = None

    def __init__(self, websocket):
        self.system_id = get_system_id()
        self.websocket = websocket
        self.cache = {}

    @staticmethod
    def connect(url):
        ssl_context = ssl.SSLContext()
        ssl_context.verify_mode = ssl.CERT_NONE
        ssl_context.check_hostname = False
        websocket = connect(url, ssl=ssl_context)
        self = Client(websocket)

        # Start a thread to receive messages
        receiver = Thread(target=receive_messages, args=(self, ))
        receiver.daemon = True
        receiver.start()

        return self

    @classmethod
    def get(self, key):
        return self.cache['keys'][key]

    @classmethod
    def keys(self):
        return self.cache['keys']

    @classmethod
    def put(self, key, value):
        pass # TODO

    @classmethod
    def stats(self):
        return self.cache['stats']

    @classmethod
    def version(self):
        return self.cache['version']

def receive_messages(self):
    for message in self.websocket:
        data = json.loads(message)
        self.cache.update(data)
