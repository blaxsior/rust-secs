import { ItemEnum } from "@/core/secs/const";
import { validateFloat, validateInt } from "@/core/util/validator";

export const itemValidator = (key: string) => {
    const floatKeys: string[] = [ItemEnum.F4, ItemEnum.F8];
    const strKeys: string[] = [ItemEnum.A];
    if(floatKeys.includes(key)) return validateFloat;
    if(strKeys.includes(key)) return () => true; // 문자열은 따로 validation 없음
    return validateInt;
};