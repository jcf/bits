(function () {
  "use strict";

  let lastEventId = null;
  let reconnectDelay = 1000;
  const MAX_RECONNECT_DELAY = 30000;

  // ---------------------------------------------------------------------------
  // Connection State

  function reportState(state, detail) {
    console.log(`[bits] ${state}`, detail || "");
    document.dispatchEvent(
      new CustomEvent("bits:connection", { detail: { state } }),
    );
  }

  // ---------------------------------------------------------------------------
  // SSE Connection

  function connect() {
    reportState("connecting");
    const url = window.location.pathname;
    const headers = { Accept: "text/event-stream" };
    if (lastEventId) headers["Last-Event-ID"] = lastEventId;

    fetch(url, { method: "POST", headers, credentials: "same-origin" })
      .then((response) => {
        if (!response.ok) throw new Error(`HTTP ${response.status}`);
        reportState("connected");
        reconnectDelay = 1000;

        const reader = response.body.getReader();
        const decoder = new TextDecoder();
        let buffer = "";

        function read() {
          reader.read().then(({ done, value }) => {
            if (done) {
              reconnect();
              return;
            }
            buffer += decoder.decode(value, { stream: true });
            buffer = processEvents(buffer);
            read();
          });
        }
        read();
      })
      .catch((err) => {
        reportState("error", err.message);
        reconnect();
      });
  }

  function reconnect() {
    reportState("reconnecting", `in ${reconnectDelay}ms`);
    setTimeout(connect, reconnectDelay);
    reconnectDelay = Math.min(reconnectDelay * 2, MAX_RECONNECT_DELAY);
  }

  // ---------------------------------------------------------------------------
  // SSE Event Parsing

  function processEvents(buffer) {
    const events = [];
    let pos = 0;

    while (true) {
      const end = buffer.indexOf("\n\n", pos);
      if (end === -1) break;

      const block = buffer.slice(pos, end);
      const event = parseEvent(block);
      if (event) events.push(event);
      pos = end + 2;
    }

    events.forEach(handleEvent);
    return buffer.slice(pos);
  }

  function parseEvent(block) {
    const event = { type: "message", id: null, data: [] };

    for (const line of block.split("\n")) {
      if (line.startsWith("event:")) {
        event.type = line.slice(6).trim();
      } else if (line.startsWith("id:")) {
        event.id = line.slice(3).trim();
      } else if (line.startsWith("data:")) {
        event.data.push(line.slice(5).trimStart());
      }
    }

    if (event.data.length === 0) return null;
    event.data = event.data.join("\n");
    return event;
  }

  // ---------------------------------------------------------------------------
  // Event Handling

  function handleEvent(event) {
    if (event.id) lastEventId = event.id;

    if (event.type === "morph") {
      const target = document.getElementById("morph");
      if (target && window.Idiomorph) {
        Idiomorph.morph(target, event.data, { morphStyle: "innerHTML" });
      }
    }
  }

  // ---------------------------------------------------------------------------
  // Action Dispatch

  function postAction(action, params) {
    const body = new URLSearchParams({ action, ...params });
    fetch("/action", {
      method: "POST",
      headers: { "Content-Type": "application/x-www-form-urlencoded" },
      body,
      credentials: "same-origin",
    });
  }

  document.addEventListener("click", (e) => {
    const el = e.target.closest("[data-action]");
    if (el) {
      e.preventDefault();
      const form = el.form;
      const params = form ? Object.fromEntries(new FormData(form)) : {};
      postAction(el.dataset.action, params);
    }
  });

  // ---------------------------------------------------------------------------
  // Init

  document.addEventListener("DOMContentLoaded", connect);
})();
