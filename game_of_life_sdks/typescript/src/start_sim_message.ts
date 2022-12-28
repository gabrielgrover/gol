import { Message } from "./types";

export type StartSimMessage = Message<"StartSim"> & {
  to_string: () => string;
};

export const StartSimMessage = (): StartSimMessage => {
  const message: Message<"StartSim"> = {
    type: "StartSim",
  };

  const to_string = () => JSON.stringify(message);

  return {
    ...message,
    to_string,
  };
};
