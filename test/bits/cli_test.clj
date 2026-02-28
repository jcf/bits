(ns bits.cli-test
  (:require
   [bits.cli :as sut]
   [clojure.test :refer [are deftest]]))

(deftest result->exit
  (are [k out] (= out (sut/result->exit {:bits.cli.exit/code k}))
    nil                              0
    :bits.cli.exit/ok                0
    :bits.cli.exit/base              64
    :bits.cli.exit/usage             64
    :bits.cli.exit/data-error        65
    :bits.cli.exit/no-input          66
    :bits.cli.exit/no-user           67
    :bits.cli.exit/no-host           68
    :bits.cli.exit/unavailable       69
    :bits.cli.exit/software-error    70
    :bits.cli.exit/os-error          71
    :bits.cli.exit/os-file-missing   72
    :bits.cli.exit/cannot-create     73
    :bits.cli.exit/io-error          74
    :bits.cli.exit/temp-failure      75
    :bits.cli.exit/protocol-error    76
    :bits.cli.exit/permission-denied 77
    :bits.cli.exit/config-error      78))
