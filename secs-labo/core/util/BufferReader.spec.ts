import { describe, expect, it } from "vitest";
import { BufferReader } from "./BufferReader";

describe("BufferReader", () => {
    it("offset: 현재 오프셋을 반환", () => {
        const buffer = new ArrayBuffer(8);
        const view = new DataView(buffer);
        view.setUint32(0, 100);
        view.setUint8(4, 255);

        const reader = new BufferReader(buffer);


        const data1 = reader.readInt32(); // offset 4
        const data2 = reader.readInt8(); // offset 5
        const offset = reader.offset;

        expect(data1).toEqual(100);
        expect(data2).toEqual(-1); // 0xFF는 Int8에서 -1
        expect(offset).toEqual(5);
    });

    it("readInt8: int8 데이터를 읽고 오프셋 1 이동", () => {
        const buffer = new ArrayBuffer(1);
        const view = new DataView(buffer);
        view.setUint8(0, 255);
        const reader = new BufferReader(buffer);

        const data = reader.readInt8();

        expect(data).toEqual(-1); // 0xFF = -1 (signed byte)
        expect(reader.offset).toEqual(1);
    });

    it("readUInt8: unsigned int8 데이터를 읽고 오프셋 1 이동", () => {
        const buffer = new ArrayBuffer(1);
        const view = new DataView(buffer);
        view.setUint8(0, 255);
        const reader = new BufferReader(buffer);

        const data = reader.readUInt8();

        expect(data).toEqual(255); // 0xFF = 255 (unsigned byte)
        expect(reader.offset).toEqual(1);
    });

    it("readInt16: int16 데이터를 읽고 오프셋 2 이동", () => {
        const buffer = new ArrayBuffer(2);
        const view = new DataView(buffer);
        view.setUint16(0, 0xFFFF);
        const reader = new BufferReader(buffer);

        const data = reader.readInt16();

        expect(data).toEqual(-1); // 0xFF FF = -1 (signed byte)
        expect(reader.offset).toEqual(2);
    });

    it("readUInt16: unsingned int16 데이터를 읽고 오프셋 2 이동", () => {
        const buffer = new ArrayBuffer(2);
        const view = new DataView(buffer);
        view.setUint16(0, 0xFFFF);
        const reader = new BufferReader(buffer);

        const data = reader.readUInt16();

        expect(data).toEqual(65535); // 0xFF FF = 65535 (unsigned byte)
        expect(reader.offset).toEqual(2);
    });

    it("readInt32: signed int32 데이터 읽고 오프셋 4 이동", () => {
        const buffer = new ArrayBuffer(4);
        const view = new DataView(buffer);
        view.setUint32(0, 0xFFFFFFFF);
        const reader = new BufferReader(buffer);

        const data = reader.readInt32();

        expect(data).toEqual(-1); // 0xFFFFFFFF = -1 (signed byte)
        expect(reader.offset).toEqual(4);
    });

    it("readUInt32: unsigned int32 데이터 읽고 오프셋 4 이동", () => {
        const buffer = new ArrayBuffer(4);
        const view = new DataView(buffer);
        view.setUint32(0, 0xFFFFFFFF);
        const reader = new BufferReader(buffer);

        const data = reader.readUInt32();

        expect(data).toEqual(0xFFFFFFFF); // 0xFFFFFFFF (unsigned byte)
        expect(reader.offset).toEqual(4);
    });

    it("readInt64: signed int64 데이터 읽고 오프셋 8 이동", () => {
        const ui64max = 0xFFFF_FFFF_FFFF_FFFFn;

        const buffer = new ArrayBuffer(8);
        const view = new DataView(buffer);
        view.setBigUint64(0, ui64max);

        const reader = new BufferReader(buffer);

        const data = reader.readInt64();

        expect(data).toEqual(-1n); // 0xFFFF FFFF FFFF FFFF = -1n (signed byte)
        expect(reader.offset).toEqual(8);
    });

    it("readUInt64: unsigned int64 데이터 읽고 오프셋 8 이동", () => {
        const ui64max = 0xFFFF_FFFF_FFFF_FFFFn;

        const buffer = new ArrayBuffer(8);
        const view = new DataView(buffer);
        view.setBigUint64(0, ui64max);

        const reader = new BufferReader(buffer);

        const data = reader.readUInt64();

        expect(data).toEqual(ui64max); // 0xFFFF FFFF FFFF FFFF = -1n (signed byte)
        expect(reader.offset).toEqual(8);
    });

    it("readFloat32: float32 데이터 읽고 오프셋 4 이동", () => {

        const buffer = new ArrayBuffer(4);
        const view = new DataView(buffer);
        view.setFloat32(0, 0.123456789);

        const reader = new BufferReader(buffer);

        const data = reader.readFloat32();

        expect(data).toEqual(0.12345679104328156); // 정확도 이슈로 0.123456789 그대로 저장 안됨
        expect(reader.offset).toEqual(4);
    });

    it("readFloat64: float64 데이터 읽고 오프셋 8 이동", () => {
        const buffer = new ArrayBuffer(8);
        const view = new DataView(buffer);
        view.setFloat64(0, 0.123456789);

        const reader = new BufferReader(buffer);

        const data = reader.readFloat64();

        expect(data).toEqual(0.123456789);
        expect(reader.offset).toEqual(8);
    });

    it("readString: string 데이터 읽고, 지정된 길이만큼 오프셋 이동", () => {
        const length = 3;
        const buffer = new ArrayBuffer(length);
        const view = new DataView(buffer);
        view.setUint8(0,0b01000001);
        view.setUint8(1,0b01000010);
        view.setUint8(2,0b01000011);

        const reader = new BufferReader(buffer);

        const data = reader.readString(length);

        expect(data).toEqual('ABC');
        expect(reader.offset).toEqual(3);
    });

    it("maxOffset: 버퍼의 최대 바이트 길이 반환", () => {
        const length = 3;
        const buffer = new ArrayBuffer(length); // 바이트 길이 3
        const reader = new BufferReader(buffer);
        
        expect(reader.maxOffset).toEqual(length);
    })
});