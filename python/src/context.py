from provider import Provider

class Context:
    def __init__(self, client, node, id):
        self.client = client
        self.node = node
        self.id = id

    def provider(self, profile):
        args = [self.node.system.id, self.node.id, self.id, profile]
        transaction = self.client.send('json', 'providers', args)
        self.client.wait_for_response(transaction)
        return Provider(self.client, self, profile)
