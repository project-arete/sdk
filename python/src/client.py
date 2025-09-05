from threading import Thread
from websockets.sync.client import connect as websockets_connect
import json
import socket
from system import get_system_id

class Client:
    def __init__(self, websocket):
        self.cache = {}
        self.system_id = get_system_id()
        self.transaction = 1
        self.websocket = websocket

    @classmethod
    def connect(cls, url, ssl=None):
        websocket = websockets_connect(url, ssl=ssl)

        client = cls(websocket)

        # Start a thread to receive messages
        receiver = Thread(target=receive_messages, args=(client, ))
        receiver.daemon = True
        receiver.start()

        return client

    def add_node(self, id, name, upstream=False, token=None):
        args = [id, name, upstream]
        return self.send('json', 'nodes', args)

    def add_system(self, id, name):
        if id is None:
            id = self.system_id
        if name is None:
            name = socket.gethostname()
        args = [id, name]
        return self.send('json', 'systems', args)

    def get(self, key):
        return self.cache['keys'][key]

    def keys(self):
        return self.cache['keys']

    def put(self, key, value):
        pass # TODO

    def send(self, format, cmd, args=[]):
        for arg in args:
            cmd = f'{cmd} "{arg}"'
        self.transaction += 1
        message = json.dumps({
            'transaction': self.transaction,
            'format': format,
            'command': cmd,
        })
        return self.websocket.send(message)

    def stats(self):
        return self.cache['stats']

    def version(self):
        return self.cache['version']

def receive_messages(self):
    for message in self.websocket:
        data = json.loads(message)
        self.cache.update(data)
