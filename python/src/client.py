from threading import Thread
from websockets.sync.client import connect as websockets_connect
from websockets.protocol import State
import json
import socket
import time
from system import get_system_id

class Client:
    def __init__(self, websocket):
        self.cache = {}
        self.requests = {}
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

    def add_context(self, node_id, id, name):
        args = [self.system_id, node_id, id, name]
        transaction = self.send('json', 'contexts', args)
        self.wait_for_response(transaction)

    def add_node(self, id, name, upstream=False, token=None):
        args = [self.system_id, id, name, upstream]
        transaction = self.send('json', 'nodes', args)
        self.wait_for_response(transaction)

    def add_system(self, id=None, name=None):
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
        args = [f'"{key}"', value]
        return self.send('json', 'put', args)

    def send(self, format, cmd, args=[]):
        for arg in args:
            cmd = f'{cmd} "{arg}"'
        self.transaction += 1
        message = json.dumps({
            'transaction': self.transaction,
            'format': format,
            'command': cmd,
        })
        self.websocket.send(message)
        self.requests[self.transaction] = None
        return self.transaction

    def stats(self):
        return self.cache['stats']

    def version(self):
        return self.cache['version']

    def wait_for_open(self, timeout_secs = 5):
        start_time = time.time()
        while time.time() - start_time < timeout_secs:
            if self.websocket.state == State.OPEN:
                return
            time.sleep(0.1)

    def wait_for_response(self, transaction, timeout_secs = 5):
        start_time = time.time()
        while time.time() - start_time < timeout_secs:
            if transaction not in self.requests:
                raise Exception('No such transaction')
            response = self.requests[transaction]
            if response is not None:
                if 'error' in response:
                    error = response['error']
                    if error is not None:
                        raise Exception(error)
                return
            time.sleep(0.1)
        raise Exception('Timed out waiting for response')

def receive_messages(self):
    for message in self.websocket:
        data = json.loads(message)
        if 'transaction' in data:
            transaction = int(data['transaction'])
            if 'response' in data:
                response = data['response']
                if response == '':
                    self.requests[transaction] = {'error': None}
        self.cache.update(data)
