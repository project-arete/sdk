// Copyright 2025 Padi, Inc. All Rights Reserved.

import { Context } from './context.js';

export class Node {
  #client;
  system;
  id;

  constructor(client, system, id) {
    this.#client = client;
    this.system = system;
    this.id = id;
  }

  context(id, name) {
    return new Promise((resolve, reject) => {
      const args = [this.system.id, this.id, id, name];
      this.#client
        .command('contexts', ...args)
        .then((res) => resolve(new Context(this.#client, this, id)))
        .catch(reject);
    });
  }
}
