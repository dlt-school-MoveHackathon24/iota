import {ObjId} from "./src/types" 

 export type CtfLuckynumberIdl = {
  get_flag: { user_counter: ObjId, lucky_num: number },
};

export const moduleName = "luckynumber";
