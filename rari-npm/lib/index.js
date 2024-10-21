import { join } from "node:path";

export const rariBin = join(
  import.meta.dirname,
  `../bin/rari${process.platform === "win32" ? ".exe" : ""}`,
);
