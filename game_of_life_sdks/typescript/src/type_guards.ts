import { z } from "zod";
import { MessageFromServer } from "./types";

const cell_validator = z.object({
  row: z.number(),
  col: z.number(),
  alive: z.boolean(),
});

const cell_array_validator = z.array(cell_validator);

const gol_subscribe_message_validator = z.object({
  cells: cell_array_validator,
  generation_index: z.number(),
});

export function is_gol_subscribe_message(
  val: unknown
): val is MessageFromServer {
  const { success } = gol_subscribe_message_validator.safeParse(val);

  return success;
}
