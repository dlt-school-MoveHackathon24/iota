interface ParsedFunction {
    name: string;
    parameters: { [key: string]: string };  // Object with key-value pairs
}

type ModuleDetails = { 
    moduleName: string, 
    moduleIdlName: string 
}

/**
 * ModuleParser class
 * TODO: describe the class
 */
export class ModuleParser {
    // Method to parse the module name from the Move file
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
     * TODO: describe the method
     * @param moveFileContent 
     * @returns 
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
     * TODO: describe the method
     * @param functions 
     * @param moduleDetails 
     * @returns 
     */
    static generateIdl(functions: ParsedFunction[], moduleDetails: ModuleDetails): string {
        const { moduleName, moduleIdlName } = moduleDetails;
        let typeString = 'import {ObjId} from "./src/types" \n\n export type ' + moduleIdlName + ' = {\n';

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
