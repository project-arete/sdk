from context import Context

class Node:
    def __init__(self, client, system, id):
        self.client = client
        self.system = system
        self.id = id

    def context(self, id, name):
        args = [self.system.id, self.id, id, name]
        transaction = self.client.send('json', 'contexts', args)
        self.client.wait_for_response(transaction)
        return Context(self, id)

