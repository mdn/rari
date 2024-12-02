import { compile, compileFromFile } from 'json-schema-to-typescript'
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import fs from 'node:fs';

const __dirname = dirname(fileURLToPath(import.meta.url));
const schemaPath = join(__dirname, '..', 'schema.json');
const typesPath = join(__dirname, 'rari-types.d.ts');

compileFromFile(schemaPath)
  .then(ts => fs.writeFileSync(typesPath, ts))

