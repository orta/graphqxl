import test from "ava";

import { graphqxlToSdl } from "../index.js";

test("graphqxlToSdl", (t) => {
  t.is(graphqxlToSdl("input", 2, "_"), "expected output");
});
