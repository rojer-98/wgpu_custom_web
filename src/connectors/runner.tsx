import { run } from "./engine";
import { get_file } from "../helpers/file";
import config from "../assets/engine/config.toml";

export default async function runner() {
  const c = await get_file(config);
  run(c);
}
