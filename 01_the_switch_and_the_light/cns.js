// cns.js - CNS Web SDK
// Copyright 2025 Padi, Inc. All Rights Reserved.

/**
 * CNS Web SDK namespace.
 */
const cns = (function() {

/**
 * Socket open error.
 * @private
 * @constant {string}
 */
const E_SOCKET = 'Socket not open';

/**
 * Socket request error.
 * @private
 * @constant {string}
 */
const E_REQUEST = 'Socket request failed';

/**
 * Retry wait time in ms.
 * @private
 * @constant {number}
 */
const RETRY = 5000;

/**
 * Class representing an event emitter.
 * @private
 * @class
 */
class Emitter {

  /**
   * Event handler bindings.
   * @private
   */
  #handlers = {};

  /**
   * Installs an event handler.
   * @method
   * @param {string} event The name of the event.
   * @param {function} handler The handler of the event.
   * @returns {this} Returns this object.
   */
  on(event, handler) {
    const handlers = this.#handlers[event];

    if (handlers === undefined)
      this.#handlers[event] = [handler];
    else handlers.push(handler);

    return this;
  }

  /**
   * Installs an event handler once.
   * @method
   * @param {string} event The name of the event.
   * @param {function} handler The handler of the event.
   * @returns {this} Returns this object.
   */
  once(event, handler) {
    const fn = (...args) => {
      off(event, fn);
      handler(...args);
    };

    return this.on(event, fn);
  }

  /**
   * Removes an event handler.
   * @method
   * @param {string} event The name of the event.
   * @param {function} handler The handler of the event.
   * @returns {this} Returns this object.
   */
  off(event, handler) {
    // Remove all handlers?
    if (event === undefined)
      this.#handlers = {};
    else {
      // Remove all event handlers?
      if (handler === undefined)
        delete this.#handlers[event];
      else {
        // Remove specific handler
        const handlers = this.#handlers[event];

        if (handlers !== undefined) {
          const fns = [];

          for (const fn of handlers)
            if (fn !== handler) fns.push(fn);

          if (fns.length === 0)
            delete this.#handlers[event];
          else this.#handlers[event] = fns;
        }
      }
    }
    return this;
  }

  /**
   * Emits an event.
   * @method
   * @param {string} event The name of the event.
   * @param {...*} args The arguments to the handler.
   * @returns {this} Returns this object.
   */
  emit(event, ...args) {
    const handlers = this.#handlers[event];

    if (handlers !== undefined) {
      const fns = [...handlers];
      for (const fn of fns) fn(...args);
    }
    return this;
  }
}

/**
 * Class representing a CNS client.
 * @class
 * @emits open
 * @emits update
 * @emits close
 * @emits error
 */
class Client extends Emitter {

  /**
   * Options passed to constructor.
   * @private
   */
  #options;

  /**
   * Socket used for communication.
   * @private
   */
  #socket;

  /**
   * Next transaction id.
   * @private
   */
  #transaction;

  /**
   * Outstanding requests.
   * @private
   */
  #requests;

  /**
   * Total updates received.
   * @private
   */
  #updates;

  /**
   * Current client cache.
   * @private
   */
  #cache;

  /**
   * Creates a new CNS client.
   * @constructor
   * @param {object} options The optional host connection options.
   * @param {string} options.protocol The connection protocol (ws or wss).
   * @param {string} options.host The connection host name.
   * @param {string} options.port The connection port number.
   */
  constructor(options = {}) {
    super();

    this.#options = {
      protocol: options.protocol || ((location.protocol === 'https:')?'wss:':'ws:'),
      host: options.host || location.hostname,
      port: options.port || location.port
//      username: options.username || '',
//      password: options.password || ''
    };

    this.open();
  }

  /**
   * Open communication channel.
   * @method
   */
  open() {
    if (this.#socket === undefined) {
      this.#reset();

      const prot = this.#options.protocol;
      const host = this.#options.host;
      const port = this.#options.port;
//      const username = this.#options.username;
//      const password = this.#options.password;

      const uri = prot + '//' + host + (port?(':' + port):'');
      this.#socket = new WebSocket(uri);

      this.#socket.onopen = this.#onopen.bind(this);
      this.#socket.onmessage = this.#onmessage.bind(this);
      this.#socket.onclose = this.#onclose.bind(this);
      this.#socket.onerror = this.#onerror.bind(this);
    }
  }

  /**
   * Is communication channel open?
   * @method
   * @returns {boolean} Returns true if open.
   */
  isOpen() {
    return (this.#socket !== undefined &&
      this.#socket.readyState === WebSocket.OPEN);
  }

  /**
   * Get CNS version.
   * @type {string}
   */
  get version() {
    return this.#cache.version;
  }

  /**
   * Get CNS stats object.
   * @type {object}
   */
  get stats() {
    return this.#cache.stats;
  }

  /**
   * Get CNS keys object.
   * @type {object}
   */
  get keys() {
    return this.#cache.keys;
  }

  /**
   * Get CNS key value.
   * @method
   * @param {string} key The key to get.
   * @param {string} def The default value if key not present.
   * @returns {object} Returns CNS key value or default or null.
   */
  get(key, def = null) {
    const value = this.keys[key];
    return (value === undefined)?def:value;
  }

  /**
   * Put CNS key value.
   * @method
   * @param {string} key The key to put.
   * @param {string} value The value to put.
   * @returns {Promise} Returns a promise.
   * @fulfil {string} - The response from the host.
   * @reject {Error} - The request failed.
   */
  put(key, value) {
    return this.command('put', key, value);
  }

  /**
   * Select matching keys.
   * @method
   * @param {string} filter The key filter.
   * @param {string} keys The optional keys list.
   * @returns {object} Returns CNS keys object.
   */
  select(filter, keys) {
    if (keys === undefined) keys = this.keys;

    const filters = filter.split('/');
    const result = {};

    for (const key in keys) {
      if (compare(key, filters))
        result[key] = keys[key];
    }
    return result;
  }

  /**
   * Execute console command.
   * @method
   * @param {string} cmd The command to execute.
   * @param {string} args The command arguments.
   * @returns {Promise} Returns a promise.
   * @fulfil {string} - The response from the host.
   * @reject {Error} - The request failed.
   */
  execute(cmd, ...args) {
    return this.#send(undefined, cmd, ...args);
  }

  /**
   * Send command to host.
   * @method
   * @param {string} cmd The command to send.
   * @param {string} args The command arguments.
   * @returns {Promise} Returns a promise.
   * @fulfil {string} - The response from the host.
   * @reject {Error} - The request failed.
   */
  command(cmd, ...args) {
    return this.#send('json', cmd, ...args);
  }

  /**
   * Close communication channel.
   * @method
   */
  close() {
    if (this.#socket !== undefined)
      this.#socket.close();
  }

  /**
   * Handle socket open event.
   * @private
   * @method
   * @param {object} e The event object.
   */
  #onopen(e) {
    //this.emit('open', e);
  }

  /**
   * Handle socket message event.
   * @private
   * @method
   * @param {object} e The event object.
   */
  #onmessage(e) {
    try {
      const data = JSON.parse(e.data);

      const transaction = data.transaction;
      const response = data.response;

      if (transaction !== undefined) {
        const request = this.#requests[transaction];

        if (request !== undefined) {
          delete this.#requests[transaction];
          request.resolve(data);
        }
        return;
      }

      merge(this.#cache, data);

      if (this.#updates++ === 0)
        this.emit('open', e);

      this.emit('update', data);
    } catch(e) {
      this.emit('error', e);
    }
  }

  /**
   * Handle socket close event.
   * @private
   * @method
   * @param {object} e The event object.
   */
  #onclose(e) {
    this.#reset();

    if (e !== undefined && e.wasClean) return;

    if (this.#socket !== undefined) {
      this.#socket = undefined;
      this.emit('close', e);
    }

    setTimeout(() => {
      this.open();
    }, RETRY);
  }

  /**
   * Handle socket error event.
   * @private
   * @method
   * @param {object} e The event object.
   */
  #onerror(e) {
    this.emit('error', new Error(E_SOCKET));
    this.close();
  }

  /**
   * Reset client cache.
   * @private
   * @method
   */
  #reset() {
    for (const request in this.#requests)
      this.#requests[request].reject(new Error(E_REQUEST + ': ' + request));

    this.#transaction = 1;
    this.#requests = {};

    this.#updates = 0;
    this.#cache = {
      version: '',
      stats: {},
      keys: {}
    };
  }

  /**
   * Send command to host.
   * @private
   * @method
   * @param {string} format The response format to use.
   * @param {string} cmd The command to send.
   * @param {string} args The command arguments.
   * @returns {Promise} Returns the previous format.
   * @fulfil {string} - The response from the host.
   * @reject {Error} - The request failed.
   */
  #send(format, cmd, ...args) {
    const self = this;

    return new Promise((resolve, reject) => {
      if (!self.isOpen())
        return reject(new Error(E_SOCKET));

      for (const arg of args)
        cmd += ' "' + arg + '"';

      const transaction = self.#transaction++;

      self.#requests[transaction] = {
        resolve: resolve,
        reject: reject
      };

      self.#socket.send(JSON.stringify({
        transaction: transaction,
        format: format,
        command: cmd
      }));
    })
  }
}

/**
 * Get value type.
 * @private
 * @function
 * @param {*} value The value to get type of.
 * @returns {string} Returns fully qualified JS type.
 */
function getType(value) {
  return Object.prototype.toString.call(value);
}

/**
 * Merge source object with target object.
 * @private
 * @function
 * @param {object} target The target object.
 * @param {object} source The source object to merge.
 */
function merge(target, source) {
  for (const key in source) {
    const value = source[key];
    const type = getType(value);

    switch (type) {
      case '[object Null]':
        delete target[key];
        break;
      case '[object Object]':
        if (getType(target[key]) !== type ||
          Object.keys(value).length === 0)
          target[key] = {};

        merge(target[key], value);
        break;
      default:
        target[key] = value;
        break;
    }
  }
}

/**
 * Compare key with filters.
 * @private
 * @function
 * @param {string} key The key to compare.
 * @param {array} filters The filters array to match.
 * @returns {boolean} Returns true if key matches filters.
 */
function compare(key, filters) {
  const parts = key.split('/');

  if (parts.length === filters.length) {
    for (var n = 0; n < parts.length; n++)
      if (!match(parts[n], filters[n])) return false;
    return true;
  }
  return false;
}

/**
 * Wildcard string match.
 * @private
 * @function
 * @param {string} str The string to compare.
 * @param {array} filter The wildcard string to match.
 * @returns {boolean} Returns true if str matches filter.
 */
function match(str, filter) {
  const esc = (s) => s.replace(/([.*+?^=!:${}()|\[\]\/\\])/g, '\\$1');
  return new RegExp('^' + filter.split('*').map(esc).join('.*') + '$', 'i').test(str);
}

// Exports

return {
  Client: Client
};

}());
