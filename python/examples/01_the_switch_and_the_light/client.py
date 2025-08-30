from threading import Thread
from websockets.sync.client import connect
import ssl

class Client:
    websocket = None

    def __init__(self, websocket):
        self.websocket = websocket

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

def receive_messages(self):
    for message in self.websocket:
        pass #print(message)
