from arete_sdk.consumer import Consumer
from arete_sdk.provider import Provider


class Context:
    def __init__(self, client, node, id):
        self.client = client
        self.node = node
        self.id = id

    def consumer(self, profile):
        args = [self.node.system.id, self.node.id, self.id, profile]
        transaction = self.client.send('json', 'consumers', args)
        self.client.wait_for_response(transaction)
        return Consumer(self.client, self, profile)

    def provider(self, profile):
        args = [self.node.system.id, self.node.id, self.id, profile]
        transaction = self.client.send('json', 'providers', args)
        self.client.wait_for_response(transaction)
        return Provider(self.client, self, profile)
