(function () {
  "use strict";

  let controller = null;
  let lastEventId = null;
  let channelId = null;
  let retryDelay = 1000;

  // Live form state — tracks which fields have been interacted with
  const _timers = new Map();
  const _used = new Set();
  let _focusoutTimer = null;

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

  function scheduleReconnect(delay = retryDelay) {
    const jitter = delay * 0.5 * Math.random();
    const wait = Math.round(delay + jitter);
    const next = Math.min(30000, delay * 2);
    log.warn("reconnecting in " + wait + "ms");
    setTimeout(() => connect(next), wait);
  }

  function connect(nextDelay = retryDelay) {
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
          scheduleReconnect(nextDelay);
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
      else if (line.startsWith("retry:")) {
        const ms = parseInt(line.slice(6).trim(), 10);
        if (!isNaN(ms)) retryDelay = ms;
      }
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
        // Check if incoming content has a form requesting reset
        const temp = document.createElement("div");
        temp.innerHTML = data;
        const shouldReset = temp.querySelector("form[data-reset]") !== null;

        Idiomorph.morph(target, data, {
          ignoreActiveValue: false,
          morphStyle: "innerHTML",
          callbacks: {
            beforeNodeMorphed: (oldNode, newNode) => {
              // Copy values to new node, allowing class updates while preserving input
              if (
                !shouldReset &&
                oldNode instanceof HTMLInputElement &&
                newNode instanceof HTMLInputElement &&
                oldNode.value
              ) {
                // Preserve password and focused input values
                if (oldNode.type === "password" || oldNode === document.activeElement) {
                  newNode.value = oldNode.value;
                }
              }
              // Preserve select values
              if (
                !shouldReset &&
                oldNode instanceof HTMLSelectElement &&
                newNode instanceof HTMLSelectElement
              ) {
                newNode.value = oldNode.value;
              }
            },
          },
        });
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
    stylesheet: (data) => {
      const links = document.querySelectorAll('link[rel="stylesheet"]');
      if (links.length !== 1) {
        log.warn("Expected exactly one stylesheet. Found:", links);
        return;
      }
      links[0].href = data;
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
    const meta = document.querySelector('meta[name="csrf-cookie"]');
    if (!meta) {
      log.warn("Page is missing required meta[name='csrf-cookie']?!");
      return null;
    }
    const match = document.cookie.match(
      new RegExp(`(^|; )${meta.content}=([^;]+)`),
    );
    return match ? match[2] : null;
  }

  function postAction(action, params) {
    const csrf = getCsrf();
    return fetch("/action", {
      method: "POST",
      headers: { "Content-Type": "application/x-www-form-urlencoded" },
      body: new URLSearchParams({ action, csrf, ...params }),
      credentials: "same-origin",
    }).then((response) => {
      // If we have a Location header in the response, we redirect to that
      // location, trusting that the server will always send us somewhere safe.
      //
      // Where the response is 200 (rather than 204), we morph the response body
      // into the DOM ignoring whatever is in the body.
      const location = response.headers.get("Location");
      if (location) {
        window.location.href = location;
      } else if (response.status === 200) {
        response.text().then((html) => handlers.morph(html));
      }
    });
  }

  document.addEventListener("click", (e) => {
    const el = e.target.closest("[data-action]");
    if (el) {
      e.preventDefault();
      const form = el.form || el.closest("form");
      const params = form ? Object.fromEntries(new FormData(form)) : {};

      if (form) form.inert = true;
      postAction(el.dataset.action, params).finally(() => {
        if (form) form.inert = false;
      });
    }
  });

  document.addEventListener("submit", (e) => {
    const form = e.target;
    // Cancel any pending focusout validation — submit takes precedence
    clearTimeout(_focusoutTimer);

    // `form.action` will return an input with name "action", if present. We
    // want the action attribute on the form element.
    const formAction = form.getAttribute("action");
    if (formAction && formAction.endsWith("/action")) {
      e.preventDefault();
      // Pass e.submitter to include the submit button's name/value in FormData
      const params = Object.fromEntries(new FormData(form, e.submitter));
      const action = params.action;
      delete params.action;
      if (action) {
        postAction(action, params);
      } else {
        log.warn("Form missing required hidden action input:", form);
      }
    }
  });

  // ---------------------------------------------------------------------------
  // Live Form Validation

  function postFormAction(form, target) {
    const raw = new FormData(form);
    const params = {};
    const action = raw.get("action");

    for (const [k, v] of raw.entries()) {
      // Skip control params that get special handling
      if (k === "action" || k === "csrf") continue;
      // Pass submit through unprefixed — it's a control param, not a field
      if (k === "submit") {
        params[k] = v;
        continue;
      }
      // Prefix unused fields so server can distinguish pristine from touched
      params[_used.has(k) ? k : "_unused_" + k] = v;
    }

    params._target = target;
    postAction(action, params);
  }

  document.addEventListener("input", (e) => {
    const form = e.target.closest('form[action="/action"]');
    if (!form || !e.target.name) return;

    _used.add(e.target.name);

    if (e.target.dataset.used === "true") {
      const key = e.target.name;
      clearTimeout(_timers.get(key));
      _timers.set(key, setTimeout(() => postFormAction(form, key), 300));
    }
  });

  document.addEventListener("change", (e) => {
    const form = e.target.closest('form[action="/action"]');
    if (!form || !e.target.name) return;

    _used.add(e.target.name);
    const tag = e.target.tagName;
    const type = e.target.type;

    // Immediate validation for discrete inputs (no debounce needed)
    if (tag === "SELECT" || type === "checkbox" || type === "radio") {
      clearTimeout(_timers.get(e.target.name));
      postFormAction(form, e.target.name);
    }
  });

  document.addEventListener("focusout", (e) => {
    const form = e.target.closest('form[action="/action"]');
    if (!form || !e.target.name) return;

    if (e.relatedTarget?.type === "submit") return;

    if (_used.has(e.target.name)) {
      clearTimeout(_timers.get(e.target.name));
      clearTimeout(_focusoutTimer);
      // Small delay lets submit event fire first and cancel this
      _focusoutTimer = setTimeout(() => postFormAction(form, e.target.name), 10);
    }
  });

  document.addEventListener("animationend", (e) => {
    if (e.target.classList.contains("form-shake")) {
      e.target.classList.remove("form-shake");
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
