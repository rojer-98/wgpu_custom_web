import init from "./engine/index.js";

export default async function init_wasm() {
  try {
    await init();
  } catch (e) {
    console.log(e);
  }
}
