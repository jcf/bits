(function () {
  "use strict";

  let controller = null;
  let lastEventId = null;
  let channelId = null;

  const log = {
    info: (msg) =>
      console.log("%c[bits]%c " + msg, "color: #0af; font-weight: bold", ""),
    warn: (msg) =>
      console.log("%c[bits]%c " + msg, "color: #fa0; font-weight: bold", ""),
    error: (msg) =>
      console.log("%c[bits]%c " + msg, "color: #f55; font-weight: bold", ""),
  };

  // ---------------------------------------------------------------------------
  // SSE Connection

  function scheduleReconnect(delay = 1000) {
    const jitter = delay * 0.5 * Math.random();
    const wait = Math.round(delay + jitter);
    const next = Math.min(30000, delay * 2);
    log.warn("reconnecting in " + wait + "ms");
    setTimeout(() => connect(next), wait);
  }

  function connect(retryDelay = 1000) {
    controller?.abort();
    controller = new AbortController();

    const nonce = crypto.randomUUID().slice(0, 8);
    const search = window.location.search;
    const url =
      window.location.pathname +
      (search ? search + "&nonce=" + nonce : "?nonce=" + nonce);

    const headers = { Accept: "text/event-stream" };
    if (lastEventId) headers["Last-Event-ID"] = lastEventId;

    fetch(url, {
      method: "POST",
      headers,
      credentials: "same-origin",
      signal: controller.signal,
      cache: "no-store",
    })
      .then((response) => {
        if (!response.ok) throw new Error(`HTTP ${response.status}`);
        log.info("connected 🚀");

        const reader = response.body.getReader();
        const decoder = new TextDecoder();
        let buffer = "";

        function read() {
          reader
            .read()
            .then(({ done, value }) => {
              if (done) {
                log.warn("stream ended");
                return scheduleReconnect();
              }
              buffer += decoder.decode(value, { stream: true });
              buffer = processEvents(buffer);
              read();
            })
            .catch((err) => {
              if (err.name !== "AbortError") {
                log.error("stream error: " + err.message);
                scheduleReconnect();
              }
            });
        }
        read();
      })
      .catch((err) => {
        if (err.name !== "AbortError") {
          log.error("fetch failed: " + err.message);
          scheduleReconnect(retryDelay);
        }
      });
  }

  // ---------------------------------------------------------------------------
  // SSE Event Parsing

  function processEvents(buffer) {
    let pos = 0;
    while (true) {
      const end = buffer.indexOf("\n\n", pos);
      if (end === -1) break;
      handleEvent(parseEvent(buffer.slice(pos, end)));
      pos = end + 2;
    }
    return buffer.slice(pos);
  }

  function parseEvent(block) {
    const event = { type: "message", data: [] };
    for (const line of block.split("\n")) {
      if (line.startsWith("event:")) event.type = line.slice(6).trim();
      else if (line.startsWith("id:")) event.id = line.slice(3).trim();
      else if (line.startsWith("data:"))
        event.data.push(line.slice(5).trimStart());
    }
    event.data = event.data.join("\n");
    return event;
  }

  // ---------------------------------------------------------------------------
  // Event Handling

  const handlers = {
    morph: (data) => {
      const target = document.getElementById("morph");
      if (target && window.Idiomorph) {
        Idiomorph.morph(target, data, { morphStyle: "innerHTML" });
        initMouseTracking();
      }
    },
    channel: (data) => {
      channelId = data;
      log.info("channel " + data.slice(0, 6));
    },
    title: (data) => {
      document.title = data;
    },
    redirect: (data) => {
      window.location.href = data;
    },
    reload: () => {
      window.location.reload();
    },
    "push-url": (data) => {
      history.pushState(null, "", data);
    },
    "replace-url": (data) => {
      history.replaceState(null, "", data);
    },
  };

  function handleEvent(event) {
    if (event.id) lastEventId = event.id;
    const handler = handlers[event.type];
    if (handler) handler(event.data);
  }

  // ---------------------------------------------------------------------------
  // Action Dispatch

  function getCsrf() {
    const match = document.cookie.match(/(^|; )__Host-bits-csrf=([^;]+)/);
    return match ? match[2] : null;
  }

  function postAction(action, params) {
    const csrf = getCsrf();
    fetch("/action", {
      method: "POST",
      headers: { "Content-Type": "application/x-www-form-urlencoded" },
      body: new URLSearchParams({ action, csrf, ...params }),
      credentials: "same-origin",
    });
  }

  document.addEventListener("click", (e) => {
    const el = e.target.closest("[data-action]");
    if (el) {
      e.preventDefault();
      const params = el.form ? Object.fromEntries(new FormData(el.form)) : {};
      postAction(el.dataset.action, params);
    }
  });

  // ---------------------------------------------------------------------------
  // Declarative Event Tracking

  function initMouseTracking() {
    document.querySelectorAll("[data-track-mouse]").forEach((el) => {
      if (el._mouseTracked) return;
      el._mouseTracked = true;

      const action = el.dataset.trackMouse;
      let timeout = null;

      el.addEventListener("mousemove", (e) => {
        if (timeout || !channelId) return;
        timeout = setTimeout(() => {
          timeout = null;
        }, 50);
        const rect = el.getBoundingClientRect();
        postAction(action, {
          channel: channelId,
          x: Math.round(e.clientX - rect.left),
          y: Math.round(e.clientY - rect.top),
        });
      });
    });
  }

  // ---------------------------------------------------------------------------
  // Init

  document.addEventListener("DOMContentLoaded", () => {
    connect();
    initMouseTracking();
  });
})();
