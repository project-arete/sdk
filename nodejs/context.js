// Copyright 2025 Padi, Inc. All Rights Reserved.

export class Context {
  #client;
  #node;
  #id;

  constructor(client, node, id) {
    this.#client = client;
    this.#node = node;
    this.#id = id;
  }
}
