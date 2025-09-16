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
}
