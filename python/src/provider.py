class Provider:
    def __init__(self, client, context, profile):
        self.client = client
        self.context = context
        self.profile = profile

    def put(self, property, value):
        system_id = self.context.node.system.id
        node_id = self.context.node.id
        context_id = self.context.id
        profile = self.profile
        key = f'cns/{system_id}/nodes/{node_id}/contexts/{context_id}/provider/{profile}/properties/{property}'
        self.client.put(key, value)
