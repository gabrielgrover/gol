declare type Message<T extends string> = {
    type: T;
};
declare type MessageFromServer = Message<"MessageFromServer"> & {
    cells: Cell[];
    generation_index: number;
};
declare type Cell = {
    row: number;
    col: number;
    alive: boolean;
};

declare type GameOfLife = {
    connect: () => void;
    destroy: () => void;
    start_sim: () => void;
};
declare const GameOfLife: (config: {
    url: string;
    on_gen_complete: (cs: Cell[]) => void;
    on_err: (msg: string) => void;
}) => GameOfLife;

export { Cell, GameOfLife, Message, MessageFromServer };
