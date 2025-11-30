(ns bits.response
  "Plain text responses, preferably only used in internal or machine-to-machine
  flows.")

(def ^:private text-plain
  "text/plain; charset=utf-8")

(def ok-response
  {:status  200
   :headers {"Content-Type" text-plain}
   :body    "OK!\n"})

(def done-response
  {:status  200
   :headers {"Content-Type" text-plain}
   :body    "Done! 🚀\n"})

(def created-response
  {:status  201
   :headers {"Content-Type" text-plain}
   :body    "OK!\n"})

(def bad-request-response
  {:status  400
   :headers {"Content-Type" text-plain}
   :body    "Bad request.\n"})

(def forbidden-response
  {:status  403
   :headers {"Content-Type" text-plain}
   :body    "Forbidden.\n"})

(def not-found-response
  {:status  404
   :headers {"Content-Type" text-plain}
   :body    "Not found.\n"})

(def unsupported-event-response
  {:status  422
   :headers {"Content-Type" text-plain}
   :body    "Unsupported event.\n"})

(def internal-server-error-response
  {:status  500
   :headers {"Content-Type" text-plain}
   :body    "Internal server error.\n"})
