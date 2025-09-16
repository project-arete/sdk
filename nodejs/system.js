// Copyright 2025 Padi, Inc. All Rights Reserved.

import * as fs from 'fs';
import uuidv5 from 'uuidv5';
import { Node } from './node.js';

const LINUX_MODEL_FILENAME = '/sys/firmware/devicetree/base/model';
const LINUX_SERIAL_NUMBER_FILENAME =
  '/sys/firmware/devicetree/base/serial-number';

export class System {
  #client;
  #id;

  constructor(client, id) {
    this.#client = client;
    this.#id = id;
  }

  node(id, name, upstream) {
    return new Promise((resolve, reject) => {
      const args = [this.#id, id, name, upstream, null];
      this.#client
        .command('nodes', ...args)
        .then((res) => resolve(new Node(this.#client, this, id)))
        .catch(reject);
    });
  }
}

export function get_system_id() {
  if (
    fs.existsSync(LINUX_MODEL_FILENAME) &&
    fs.existsSync(LINUX_SERIAL_NUMBER_FILENAME)
  ) {
    return get_system_id_linux();
  }
  throw 'Unable to detect System ID on this platform';
}

function get_system_id_linux() {
  const model = fs.readFileSync(LINUX_MODEL_FILENAME);
  const serialNumber = fs.readFileSync(LINUX_SERIAL_NUMBER_FILENAME);
  const modelPlusSerialNumber = `${model}:${serialNumber}`;

  const id = uuidv5('oid', modelPlusSerialNumber);
  return id;
}
