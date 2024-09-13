export type ObjId = {id: string};

export function isObjId(value: any): value is ObjId {
    return typeof value === 'object' && value !== null && 'id' in value;
}