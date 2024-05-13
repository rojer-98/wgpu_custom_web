import { run, user_event_action } from "./engine";

export function test_user_event_action() {
  user_event_action();
}

export default function engine_runner() {
  try {
     run();
  } catch (e) {
    console.log(e);
  }
}
