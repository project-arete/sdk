class Provider:
    def __init__(self, client, context, profile):
        self.client = client
        self.context = context
        self.profile = profile

    def put(self, property, value):
        key = f'cns/{self.context.node.system.id}/nodes/{self.context.node.id}/contexts/{self.context.id}/provider/{self.profile}/properties/{property}'
        self.client.put(key, value)
