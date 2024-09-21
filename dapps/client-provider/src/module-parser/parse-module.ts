/**
 * CLI command to generate IDL from a Move contract file.
 */

import * as fs from 'fs';
import * as path from 'path';
import { ModuleParser } from './module-parser';

const IDL_DIR_PATH = "src/idl";

const args = process.argv.slice(2);

if (args.length < 1) {
    console.error('Please provide the path to the Move contract file.');
    process.exit(1);
}

const moveFilePath = path.join(process.cwd(), args[0]);

if (!fs.existsSync(moveFilePath)) {
    console.error(`File not found: ${moveFilePath}`);
    process.exit(1);
}

// Loading the Move contract module
const moveFileContent = fs.readFileSync(moveFilePath, 'utf8');

// Parse the module details
const moduleDetails = ModuleParser.parseModuleDetails(moveFileContent);
const parsedFunctions = ModuleParser.parseMethods(moveFileContent);

// Generate IDL
const idl = ModuleParser.generateIdl(parsedFunctions, moduleDetails);

// Save the generated IDL
const outputPath = path.join(process.cwd(), IDL_DIR_PATH, `${moduleDetails.moduleIdlName}.ts`);
fs.writeFileSync(outputPath, idl);

console.log(`IDL generated at ${outputPath}`);
