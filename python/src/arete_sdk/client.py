from threading import Thread
from websockets.sync.client import connect as websockets_connect
from websockets.protocol import State
import json
import socket
import time
from system import get_system_id, System


class Client:
    def __init__(self, websocket):
        self.cache = {}
        self.requests = {}
        self.subscribers = []
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

    def get(self, key):
        return self.cache['keys'][key]

    def keys(self):
        return self.cache['keys']

    def on_update(self, fn):
        self.subscribers.append(fn)

    def put(self, key, value):
        args = [key, value]
        return self.send('json', 'put', args)

    def send(self, format, cmd, args=[]):
        for arg in args:
            cmd = f'{cmd} "{arg}"'
        transaction = self.transaction
        self.transaction += 1
        message = json.dumps({
            'transaction': transaction,
            'format': format,
            'command': cmd,
        })
        self.websocket.send(message)
        self.requests[transaction] = None
        return transaction

    def stats(self):
        return self.cache.get('stats', None)

    def system(self):
        id = self.system_id
        name = socket.gethostname()
        args = [id, name]
        transaction = self.send('json', 'systems', args)
        self.wait_for_response(transaction)
        return System(self, id)

    def version(self):
        return self.cache.get('version', None)

    def wait_for_open(self, timeout_secs=5):
        start_time = time.time()
        while time.time() - start_time < timeout_secs:
            if self.websocket.state == State.OPEN and self.version() is not None:
                return
            time.sleep(0.1)

    def wait_for_response(self, transaction, timeout_secs=5):
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

        # Merge with cache
        if 'stats' in data:
            if 'stats' not in self.cache:
                self.cache['stats'] = {}
            self.cache['stats'].update(data['stats'])
        if 'keys' in data:
            if 'keys' not in self.cache:
                self.cache['keys'] = {}
            self.cache['keys'].update(data['keys'])

        # Notify watches
        if 'keys' in data:
            for fn in self.subscribers:
                fn(data)
