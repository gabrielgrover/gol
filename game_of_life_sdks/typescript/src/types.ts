export type Message<T extends string> = {
  type: T;
};

export type MessageFromServer = Message<"MessageFromServer"> & {
  cells: Cell[];
  generation_index: number;
};

export type Cell = {
  row: number;
  col: number;
  alive: boolean;
};
