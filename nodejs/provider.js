// Copyright 2025 Padi, Inc. All Rights Reserved.

export class Provider {
  #client;
  context;
  profile;

  constructor(client, context, profile) {
    this.#client = client;
    this.context = context;
    this.profile = profile;
  }

  put(property, value) {
    const key = `cns/${this.context.node.system.id}/nodes/${this.context.node.id}/contexts/${this.context.id}/provider/${this.profile}/properties/${property}`;
    return this.#client.put(key, value);
  }
}
