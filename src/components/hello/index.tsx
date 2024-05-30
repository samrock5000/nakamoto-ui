import {
  $,
  component$,
  useSignal,
  useStore,
  useVisibleTask$,
} from "@builder.io/qwik";
import { invoke } from "@tauri-apps/api/tauri";
import { listen } from "@tauri-apps/api/event";

type MSG = {
  timeStamp: number;
  message: string;
};

export default component$(() => {
  const msg = useSignal("");
  const currHeight = useSignal(0);
  const eventCount = useSignal(0);
  const peers = useSignal("");
  const connectReq = $(async (message: string) => {
    invoke("ui_request", { message });
  });

  const getNextMessage = $(async (message: string) => {
    peers.value = await invoke("ui_request", { message });
  });
  useVisibleTask$(async () => {
    await listen("net_event", (event) => {
      eventCount.value += 1;
      msg.value = event.payload;
    });
    console.log("net event payload ", msg.value);
  });

  return (
    <div>
      <button onClick$={() => connectReq("get-block-2")}>Send Req</button>
      <p>{peers.value}</p>
      <p>{"Events"}</p>
      <p>{msg.value}</p>
      <p>{"Events Count"}</p>
      <p>{eventCount.value}</p>
    </div>
  );
});
