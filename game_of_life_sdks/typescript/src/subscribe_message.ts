import { Message } from "./types";

export type SubscribeMessage = Message<"Subscribe"> & {
  to_string: () => string;
};

export const SubscribeMessage = (): SubscribeMessage => {
  const message: Message<"Subscribe"> = {
    type: "Subscribe",
  };

  const to_string = () => JSON.stringify(message);

  return {
    ...message,
    to_string,
  };
};
