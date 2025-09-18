// Copyright 2025 Padi, Inc. All Rights Reserved.

export class Consumer {
  #client;
  context;
  profile;

  constructor(client, context, profile) {
    this.#client = client;
    this.context = context;
    this.profile = profile;
  }

  watch(handler) {
    const keyPrefix = `cns/${this.context.node.system.id}/nodes/${this.context.node.id}/contexts/${this.context.id}/consumer/${this.profile}/`;
    const re = new RegExp(`connections/(\\w+)/properties/(\\w+)$`);
    this.#client.on('update', (event) => {
      for (let [key, value] of Object.entries(event.keys)) {
        if (!key.startsWith(keyPrefix)) {
          continue;
        }
        const captures = key.match(re);
        if (captures.length < 3) {
          continue;
        }
        const connection = captures[1];
        const property = captures[2];
        const changeEvent = {
          connection,
          property,
          value,
        };
        handler(changeEvent);
      }
    });
  }
}
