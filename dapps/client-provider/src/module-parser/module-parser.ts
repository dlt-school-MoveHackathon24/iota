interface ParsedFunction {
    name: string;
    parameters: { [key: string]: string };
}

type ModuleDetails = { 
    moduleName: string, 
    moduleIdlName: string 
}

const WARNING_AUTOGENERATED = "// AUTOGENERATED FILE. DO NOT EDIT.\n\n";

/**
 * ModuleParser class
 * 
 * This class provides functionality to parse a Move contract and extract
 * module and function details, which are then used to generate an Interface Definition Language (IDL)
 * file in TypeScript. This IDL file facilitates type-safe client-side 
 * interactions with the contract.
 */
export class ModuleParser {
    /**
     * Parses the module details from the provided Move file content.
     * 
     * @param moveFileContent - The content of the Move file as a string.
     * @returns An object containing the module name and the generated IDL name.
     * @throws Will throw an error if the module name is not found in the file.
     */
    static parseModuleDetails(moveFileContent: string): ModuleDetails {
        const moduleRegex = /module\s+(\w+)::(\w+)/;
        const match = moduleRegex.exec(moveFileContent);

        if (!match) throw new Error("Module name not found in the Move file");

        const moduleName = match[2];
        const pascalCaseName = match[1].charAt(0).toUpperCase() + match[1].slice(1).toLowerCase() +
            match[2].charAt(0).toUpperCase() + match[2].slice(1).toLowerCase();

        return {
            moduleName,
            moduleIdlName: `${pascalCaseName}Idl`
        }
    }

    /**
     * Parses the functions from the provided Move file content, excluding the `init` function.
     * Extracts function names and their parameters, converting them into TypeScript types.
     * 
     * @param moveFileContent - The content of the Move file as a string.
     * @returns An array of parsed functions with their names and parameters.
     */
    static parseMethods(moveFileContent: string): ParsedFunction[] {
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

                    if (type === 'u64') {
                        tsType = 'number';
                    }

                    acc[name] = tsType;
                    return acc;
                }, {} as { [key: string]: string });

            if (functionName !== 'init') {  // Exclude init function
                functions.push({ name: functionName, parameters });
            }
        }

        return functions;
    }

    /**
     * Generates the TypeScript IDL (Interface Definition Language) file content
     * from the parsed functions and module details. The IDL specifies the structure
     * of the contract interface to enable type-safe client-side interactions.
     * 
     * @param functions - An array of parsed functions with their names and parameters.
     * @param moduleDetails - Details of the module including its name and IDL name.
     * @returns A string representing the TypeScript IDL file content.
     */
    static generateIdl(functions: ParsedFunction[], moduleDetails: ModuleDetails): string {
        const { moduleName, moduleIdlName } = moduleDetails;

        let typeString = WARNING_AUTOGENERATED;

        typeString += 'import {ObjId} from "../types" \n\n export type ' + moduleIdlName + ' = {\n';

        functions.forEach(fn => {
            const paramsObject = Object.entries(fn.parameters)
                .map(([name, type]) => `${name}: ${type}`)
                .join(', ');

            typeString += `  ${fn.name}: { ${paramsObject} },\n`;
        });

        typeString += '};\n\n';

        typeString += `export const moduleName = "${moduleName}";\n`;
        return typeString;
    }
}
