import * as WS from "websocket";
import { Cell, MessageFromServer } from "./types";
import { is_gol_subscribe_message } from "./type_guards";
import * as F from "fp-ts/function";
import * as TE from "fp-ts/TaskEither";
import * as O from "fp-ts/Option";
import * as T from "fp-ts/Task";
import { SubscribeMessage } from "./subscribe_message";
import { StartSimMessage } from "./start_sim_message";
// import * as A from "fp-ts/Array";
// import { Ord, fromCompare, contramap } from "fp-ts/Ord";

export type GameOfLife = {
  connect: () => void;
  destroy: () => void;
  start_sim: () => void;
};

type GameOfLifeState = {
  //cells: (Cell & { generation_index: number })[];
  cells: Cell[];
  errors: string[];
  connection: O.Option<WS.connection>;
  generation_index: number;
};

const INITIAL_STATE: GameOfLifeState = {
  cells: [],
  errors: [],
  connection: O.none,
  generation_index: 0,
};

export const GameOfLife = (config: {
  url: string;
  on_gen_complete: (cs: Cell[]) => void;
  on_err: (msg: string) => void;
}): GameOfLife => {
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

    state = {
      ...state,
      cells: [],
    };
  };

  const add_subscription = F.pipe(
    subscribe,
    TE.map((s) =>
      s((msg) => {
        if (msg.generation_index > state.generation_index) {
          drain();
        }

        state = {
          ...state,
          cells: [
            ...state.cells,
            ...msg.cells.map((c) => ({
              ...c,
              generation_index: msg.generation_index,
            })),
          ],
          generation_index: msg.generation_index,
        };
      }, add_err)
    )
  );

  const add_err = (err_msg: string) => {
    config.on_err(err_msg);

    state = { ...state, errors: [...state.errors, err_msg] };
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
      state = { ...state, connection };
    })
  );

  const connect = async () => {
    await establish_connection();
  };

  const destroy = () =>
    F.pipe(
      state.connection,
      O.map((c) => {
        c.removeAllListeners();
      })
    );

  const start_sim = () =>
    F.pipe(
      state.connection,
      O.map((connection) => {
        connection.sendUTF(StartSimMessage().to_string());
      })
    );

  return {
    connect,
    destroy,
    start_sim,
    //get_cells,
  };
};

function connect_to_gol_server(url: string) {
  const client = new WS.client();

  client.connect(url);

  return new Promise<WS.connection>((resolve, reject) => {
    client.on("connectFailed", (err) => {
      reject(err);
    });

    client.on("connect", (connection) => {
      resolve(connection);
    });
  });
}

function subscribe_to_gol_sim(connection: WS.connection) {
  return (
    on_change: (msg: MessageFromServer) => void,
    on_err: (err_msg: string) => void
  ) => {
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

function send_subscribe_message(
  connection: WS.connection,
  on_err: (err_msg: string) => void
) {
  try {
    connection.sendUTF(SubscribeMessage().to_string());
  } catch (err) {
    on_err(`Failed to send subscribe message to GOL Server: ${err}`);
  }
}

// const ord_number_ascending: Ord<number> = fromCompare((x, y) =>
//   x < y ? 1 : x > y ? -1 : 0
// );
//
// const by_gen: Ord<{ generation_index: number }> = contramap(
//   (c: { generation_index: number }) => c.generation_index
// )(ord_number_ascending);
//
// function sort_cells_by_gen_ascending(
//   cells: (Cell & { generation_index: number })[]
// ) {
//   return A.sortBy([by_gen])(cells);
// }
