import { run } from "./engine";
import { FileType, get_file } from "../helpers/file";

export default async function runner() {
  const config_file = await get_file(FileType.Config);

  run(config_file);
}
