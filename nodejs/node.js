// Copyright 2025 Padi, Inc. All Rights Reserved.

export class Node {
  #client;
  #system;
  #id;

  constructor(client, system, id) {
    this.#client = client;
    this.#system = system;
    this.#id = id;
  }
}
