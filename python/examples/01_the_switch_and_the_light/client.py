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
        return Client(websocket)
