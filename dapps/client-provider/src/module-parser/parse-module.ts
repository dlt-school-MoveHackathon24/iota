import * as fs from 'fs';
import * as path from 'path';
import { ModuleParser } from './module-parser';

const args = process.argv.slice(2);

if (args.length < 1) {
    console.error('Please provide the path to the Move contract file.');
    process.exit(1);
}

// Move contract file relative to the current working directory
const moveFilePath = path.join(process.cwd(), args[0]);

if (!fs.existsSync(moveFilePath)) {
    console.error(`File not found: ${moveFilePath}`);
    process.exit(1);
}

const moveFileContent = fs.readFileSync(moveFilePath, 'utf8');

// Parse the module details and the Move file content
const moduleDetails = ModuleParser.parseModuleDetails(moveFileContent);
const parsedFunctions = ModuleParser.parseMethods(moveFileContent);

// Generate the TypeScript type
const idl = ModuleParser.generateIdl(parsedFunctions, moduleDetails);

// Save the type to a file based on the module name in the current working directory
const outputPath = path.join(process.cwd(), `${moduleDetails.moduleIdlName}.ts`);
fs.writeFileSync(outputPath, idl);

console.log(`IDL generated at ${outputPath}`);
