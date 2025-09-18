import re

class Consumer:
    def __init__(self, client, context, profile):
        self.client = client
        self.context = context
        self.profile = profile

    def watch(self, fn):
        key_prefix = f'cns/{self.context.node.system.id}/nodes/{self.context.node.id}/contexts/{self.context.id}/consumer/{self.profile}/'
        def on_update(event):
            for key, value in event['keys'].items():
                if not key.startswith(key_prefix):
                    continue
                captures = re.search('connections/(\w+)/properties/(\w+)$', key)
                connection = captures.group(1)
                property = captures.group(2)
                change_event = {
                    'connection': connection,
                    'property': property,
                    'value': value,
                }
                fn(change_event)
        self.client.on_update(on_update)
