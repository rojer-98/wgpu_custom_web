import { run, user_event_action } from "./engine";
import { get_file } from "../helpers/file";
import config from "../assets/engine/config.toml";

export function test_user_event_action() {
  user_event_action();
}

export default async function engine_runner() {
  const c = await get_file(config);

  try {
    await run(c);
  } catch (e) {
    console.log(e);
  }
}
