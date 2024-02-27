export async function get_file(file: string): Promise<string> {
  let c: string = "";

  await fetch(file)
    .then((r) => r.text())
    .then((text) => {
      c = text;
    });

  return c;
}
