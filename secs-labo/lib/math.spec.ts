import { describe, expect, it } from "vitest";
import { saturate } from "./math";

describe("math test", () => {
    it("saturate: 값을 최대 / 최소 범위에서 자른다.", () => {
        const min = 0;
        const max = 255;
        
        const target1 = -1; //최소보다 작음
        const result1 = saturate(target1, min, max);
        expect(result1).toEqual(min);

        const target2 = 30; // 정상 범위
        const result2 = saturate(target2, min, max);
        expect(result2).toEqual(target2);

        const target3 = 256; // 최대보다 큼
        const result3 = saturate(target3, min, max);
        expect(result3).toEqual(max);
    });
});