// Copyright 2025 Padi, Inc. All Rights Reserved.

export class Consumer {
  #client;
  context;
  id;

  constructor(client, context, id) {
    this.#client = client;
    this.context = context;
    this.id = id;
  }
}
