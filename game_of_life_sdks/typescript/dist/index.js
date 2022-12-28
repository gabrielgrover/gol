"use strict";
var __create = Object.create;
var __defProp = Object.defineProperty;
var __defProps = Object.defineProperties;
var __getOwnPropDesc = Object.getOwnPropertyDescriptor;
var __getOwnPropDescs = Object.getOwnPropertyDescriptors;
var __getOwnPropNames = Object.getOwnPropertyNames;
var __getOwnPropSymbols = Object.getOwnPropertySymbols;
var __getProtoOf = Object.getPrototypeOf;
var __hasOwnProp = Object.prototype.hasOwnProperty;
var __propIsEnum = Object.prototype.propertyIsEnumerable;
var __defNormalProp = (obj, key, value) => key in obj ? __defProp(obj, key, { enumerable: true, configurable: true, writable: true, value }) : obj[key] = value;
var __spreadValues = (a, b) => {
  for (var prop in b || (b = {}))
    if (__hasOwnProp.call(b, prop))
      __defNormalProp(a, prop, b[prop]);
  if (__getOwnPropSymbols)
    for (var prop of __getOwnPropSymbols(b)) {
      if (__propIsEnum.call(b, prop))
        __defNormalProp(a, prop, b[prop]);
    }
  return a;
};
var __spreadProps = (a, b) => __defProps(a, __getOwnPropDescs(b));
var __export = (target, all) => {
  for (var name in all)
    __defProp(target, name, { get: all[name], enumerable: true });
};
var __copyProps = (to, from, except, desc) => {
  if (from && typeof from === "object" || typeof from === "function") {
    for (let key of __getOwnPropNames(from))
      if (!__hasOwnProp.call(to, key) && key !== except)
        __defProp(to, key, { get: () => from[key], enumerable: !(desc = __getOwnPropDesc(from, key)) || desc.enumerable });
  }
  return to;
};
var __toESM = (mod, isNodeMode, target) => (target = mod != null ? __create(__getProtoOf(mod)) : {}, __copyProps(
  isNodeMode || !mod || !mod.__esModule ? __defProp(target, "default", { value: mod, enumerable: true }) : target,
  mod
));
var __toCommonJS = (mod) => __copyProps(__defProp({}, "__esModule", { value: true }), mod);
var __async = (__this, __arguments, generator) => {
  return new Promise((resolve, reject) => {
    var fulfilled = (value) => {
      try {
        step(generator.next(value));
      } catch (e) {
        reject(e);
      }
    };
    var rejected = (value) => {
      try {
        step(generator.throw(value));
      } catch (e) {
        reject(e);
      }
    };
    var step = (x) => x.done ? resolve(x.value) : Promise.resolve(x.value).then(fulfilled, rejected);
    step((generator = generator.apply(__this, __arguments)).next());
  });
};

// src/index.ts
var src_exports = {};
__export(src_exports, {
  GameOfLife: () => GameOfLife
});
module.exports = __toCommonJS(src_exports);

// src/GameOfLife.ts
var WS = __toESM(require("websocket"));

// src/type_guards.ts
var import_zod = require("zod");
var cell_validator = import_zod.z.object({
  row: import_zod.z.number(),
  col: import_zod.z.number(),
  alive: import_zod.z.boolean()
});
var cell_array_validator = import_zod.z.array(cell_validator);
var gol_subscribe_message_validator = import_zod.z.object({
  cells: cell_array_validator,
  generation_index: import_zod.z.number()
});
function is_gol_subscribe_message(val) {
  const { success } = gol_subscribe_message_validator.safeParse(val);
  return success;
}

// src/GameOfLife.ts
var F = __toESM(require("fp-ts/function"));
var TE = __toESM(require("fp-ts/TaskEither"));
var O = __toESM(require("fp-ts/Option"));
var T = __toESM(require("fp-ts/Task"));

// src/subscribe_message.ts
var SubscribeMessage = () => {
  const message = {
    type: "Subscribe"
  };
  const to_string = () => JSON.stringify(message);
  return __spreadProps(__spreadValues({}, message), {
    to_string
  });
};

// src/start_sim_message.ts
var StartSimMessage = () => {
  const message = {
    type: "StartSim"
  };
  const to_string = () => JSON.stringify(message);
  return __spreadProps(__spreadValues({}, message), {
    to_string
  });
};

// src/GameOfLife.ts
var INITIAL_STATE = {
  cells: [],
  errors: [],
  connection: O.none,
  generation_index: 0
};
var GameOfLife = (config) => {
  let state = INITIAL_STATE;
  const subscribe = F.pipe(
    TE.tryCatch(
      () => connect_to_gol_server(config.url),
      (err) => `Websocket connection failed:  ${err}`
    ),
    TE.map(subscribe_to_gol_sim)
  );
  const drain = () => {
    config.on_gen_complete(state.cells);
    state = __spreadProps(__spreadValues({}, state), {
      cells: []
    });
  };
  const add_subscription = F.pipe(
    subscribe,
    TE.map(
      (s) => s((msg) => {
        if (msg.generation_index > state.generation_index) {
          drain();
        }
        state = __spreadProps(__spreadValues({}, state), {
          cells: [
            ...state.cells,
            ...msg.cells.map((c) => __spreadProps(__spreadValues({}, c), {
              generation_index: msg.generation_index
            }))
          ],
          generation_index: msg.generation_index
        });
      }, add_err)
    )
  );
  const add_err = (err_msg) => {
    config.on_err(err_msg);
    state = __spreadProps(__spreadValues({}, state), { errors: [...state.errors, err_msg] });
  };
  const establish_connection = F.pipe(
    add_subscription,
    TE.fold(
      (err_msg) => {
        add_err(err_msg);
        return T.of(O.none);
      },
      (connection) => T.of(O.some(connection))
    ),
    T.map((connection) => {
      state = __spreadProps(__spreadValues({}, state), { connection });
    })
  );
  const connect = () => __async(void 0, null, function* () {
    yield establish_connection();
  });
  const destroy = () => F.pipe(
    state.connection,
    O.map((c) => {
      c.removeAllListeners();
    })
  );
  const start_sim = () => F.pipe(
    state.connection,
    O.map((connection) => {
      connection.sendUTF(StartSimMessage().to_string());
    })
  );
  return {
    connect,
    destroy,
    start_sim
  };
};
function connect_to_gol_server(url) {
  const client2 = new WS.client();
  client2.connect(url);
  return new Promise((resolve, reject) => {
    client2.on("connectFailed", (err) => {
      reject(err);
    });
    client2.on("connect", (connection) => {
      resolve(connection);
    });
  });
}
function subscribe_to_gol_sim(connection) {
  return (on_change, on_err) => {
    send_subscribe_message(connection, on_err);
    connection.on("message", (data) => {
      if (data.type === "utf8") {
        const message = JSON.parse(data.utf8Data);
        if (is_gol_subscribe_message(message)) {
          on_change(message);
        }
      }
    });
    connection.on("error", (err) => {
      on_err(`Subscription error: ${err}`);
    });
    return connection;
  };
}
function send_subscribe_message(connection, on_err) {
  try {
    connection.sendUTF(SubscribeMessage().to_string());
  } catch (err) {
    on_err(`Failed to send subscribe message to GOL Server: ${err}`);
  }
}
// Annotate the CommonJS export names for ESM import in node:
0 && (module.exports = {
  GameOfLife
});
