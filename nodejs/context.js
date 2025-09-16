// Copyright 2025 Padi, Inc. All Rights Reserved.

import { Consumer } from './consumer.js';
import { Provider } from './provider.js';

export class Context {
  #client;
  node;
  id;

  constructor(client, node, id) {
    this.#client = client;
    this.node = node;
    this.id = id;
  }

  consumer(profile) {
    return new Promise((resolve, reject) => {
      const args = [this.node.system.id, this.node.id, this.id, profile];
      this.#client
        .command('consumers', ...args)
        .then((res) => resolve(new Consumer(this.#client, this, profile)))
        .catch(reject);
    });
  }

  provider(profile) {
    return new Promise((resolve, reject) => {
      const args = [this.node.system.id, this.node.id, this.id, profile];
      this.#client
        .command('providers', ...args)
        .then((res) => resolve(new Provider(this.#client, this, profile)))
        .catch(reject);
    });
  }
}
