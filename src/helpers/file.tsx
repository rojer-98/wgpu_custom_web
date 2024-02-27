export enum FileType {
  Config = "assets/config.toml",
}

export async function get_file(file_type: FileType): Promise<string> {
  let c: string = "";

  await fetch(file_type)
    .then((r) => r.text())
    .then((text) => {
      c = text;
    });

  return c;
}
