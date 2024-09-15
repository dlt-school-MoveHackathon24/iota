// =============================================================================
// Imports
// =============================================================================

import * as fs from 'fs';
import * as path from 'path';

// =============================================================================


// =============================================================================
// Types
// =============================================================================

// Interface for the parsed Move function
interface ParsedFunction {
    name: string;
    parameters: { [key: string]: string };  // Object with key-value pairs
}

// =============================================================================


// =============================================================================
// Functions
// =============================================================================

// Function to parse the module name from the Move file
function parseModuleName(moveFileContent: string): string {
    const moduleRegex = /module\s+(\w+::\w+)/;
    const match = moduleRegex.exec(moveFileContent);
    if (match) {
        const moduleName = match[1];
        const pascalCaseName = moduleName
            .split('::')
            .map(part => part.charAt(0).toUpperCase() + part.slice(1).toLowerCase())
            .join('');
        return `${pascalCaseName}Interface`;
    }
    throw new Error("Module name not found in the Move file");
}

// Function to parse the Move file and extract function signatures
function parseMoveContract(moveFileContent: string): ParsedFunction[] {
    const functions: ParsedFunction[] = [];
    const regex = /fun\s+(\w+)\s*\(([^)]*)\)\s*\{?/g;
    let match;

    while ((match = regex.exec(moveFileContent)) !== null) {
        const functionName = match[1];
        const parameters = match[2]
            .split(',')
            .map(param => param.trim())
            .filter(Boolean)
            .filter(param => !param.includes('TxContext'))
            .reduce((acc, param) => {
                const [name, type] = param.split(':').map(s => s.trim());
                let tsType: string = 'ObjId';

                if(type === 'u64') {
                    tsType = 'number';
                }

                acc[name] = tsType;
                return acc;
            }, {} as { [key: string]: string }); // Initialize as an empty object

        if (functionName !== 'init') {  // Exclude init function
            functions.push({ name: functionName, parameters });
        }
    }

    return functions;
}

// Function to generate TypeScript type
function generateType(functions: ParsedFunction[], moduleName: string): string {
    let typeString = 'import {ObjId} from "./types" \n\n export type ' + moduleName + ' = {\n';

    functions.forEach(fn => {
        const paramsObject = Object.entries(fn.parameters)
            .map(([name, type]) => `${name}: ${type}`)
            .join(', ');

        typeString += `  ${fn.name}: { ${paramsObject} },\n`;
    });

    typeString += '};\n';
    return typeString;
}

// =============================================================================


// =============================================================================
// Main
// =============================================================================

const args = process.argv.slice(2);
if (args.length < 1) {
    console.error('Please provide the path to the Move contract file.');
    process.exit(1);
}

// Move contract file
const moveFilePath = args[0];
if (!fs.existsSync(moveFilePath)) {
    console.error(`File not found: ${moveFilePath}`);
    process.exit(1);
}

const moveFileContent = fs.readFileSync(moveFilePath, 'utf8');

// Parse the module name and the Move file content
const moduleName = parseModuleName(moveFileContent);
const parsedFunctions = parseMoveContract(moveFileContent);

// Generate the TypeScript type
const typeContent = generateType(parsedFunctions, moduleName);

// Save the type to a file based on the module name
const outputPath = path.join(__dirname, `${moduleName}.ts`);
fs.writeFileSync(outputPath, typeContent);

console.log(`Contract type generated: ${outputPath}`);

// =============================================================================