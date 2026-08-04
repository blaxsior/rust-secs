import { describe, expect, it } from "vitest";
import { SecsItemToSMLSerializer } from "./serializer";
import { Secs2Item } from "@/types/secs2";

describe("SecsItemSerializer Test", () => {
    describe("serialize", () => {
        it("아이템이 없다면 개수를 명시하지 않음", () => {
            const item: Secs2Item = {
                format: 'list',
                value: []
            };
            const expectedPattern = /<\s*L\s*>\s+\./;

            const serializer = new SecsItemToSMLSerializer();
            const result = serializer.serialize(item);
            expect(result).toMatch(expectedPattern);
        });

        it("리스트를 제외한 아이템들은 아이템이 2개 이상일 때 개수를 명시", () => {
            const item: Secs2Item = {
                format: 'int4',
                value: [2, 65535]
            };
            const expectedPattern = /<\s*I4\s+\[2\]\s+2\s+65535\s*>\s+\./;
            
            const serializer = new SecsItemToSMLSerializer();
            const result = serializer.serialize(item);
            expect(result).toMatch(expectedPattern);
        });

        
        it("리스트를 제외한 아이템들은 아이템이 1개 이하라면 개수 명시 X", () => {
            const item: Secs2Item = {
                format: 'int2',
                value: [2]
            };
            const expectedPattern = /<\s*I2\s+2\s*>\s+\./;
            
            const serializer = new SecsItemToSMLSerializer();
            const result = serializer.serialize(item);
            expect(result).toMatch(expectedPattern);
        });

        it("바이너리 타입은 값을 hex 표기법으로 표현", () => {
            const item: Secs2Item = {
                format: 'binary',
                value: [253, 10] // 0xFD, 0x0A
            };
            const expectedPattern = /<\s*B\s+\[2\]\s+0xFD\s+0x0A\s*>\s+\./;
            
            const serializer = new SecsItemToSMLSerializer();
            const result = serializer.serialize(item);
            expect(result).toMatch(expectedPattern);
        });
    });
});