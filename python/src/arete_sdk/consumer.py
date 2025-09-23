import re


class Consumer:
    def __init__(self, client, context, profile):
        self.client = client
        self.context = context
        self.profile = profile

    def profile_key_prefix(self):
        system_id = self.context.node.system.id
        node_id = self.context.node.id
        context_id = self.context.id
        profile = self.profile
        return f'cns/{system_id}/nodes/{node_id}/contexts/{context_id}/consumer/{profile}/'

    def property_key(self, property):
        profile_key_prefix = self.profile_key_prefix()
        return f'{profile_key_prefix}properties/{property}'

    def get(self, property):
        key = self.property_key(property)
        self.client.get(key)

    def put(self, property, value):
        key = self.property_key(property)
        self.client.put(key, value)

    def watch(self, fn):
        key_prefix = self.profile_key_prefix()

        # Start by notifying of existing cached properties
        for key, value in self.client.keys():
            if not key.startswith(key_prefix):
                continue
            captures = re.search('connections/(\\w+)/properties/(\\w+)$', key)
            connection = captures.group(1)
            property = captures.group(2)
            change_event = {
                'connection': connection,
                'property': property,
                'value': value,
            }
            fn(change_event)

        # Watch for future property changes
        def on_update(event):
            for key, value in event['keys'].items():
                if not key.startswith(key_prefix):
                    continue
                captures = re.search('connections/(\\w+)/properties/(\\w+)$', key)
                if captures is None:
                    continue
                connection = captures.group(1)
                property = captures.group(2)
                change_event = {
                    'connection': connection,
                    'property': property,
                    'value': value,
                }
                fn(change_event)
        self.client.on_update(on_update)
