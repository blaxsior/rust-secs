import { describe, expect, it } from "vitest";
import { BufferWriter } from "./BufferWriter";

describe("BufferWriter", () => {
    it("offset: 현재 오프셋을 반환", () => {
        const buffer = new ArrayBuffer(8);
        const writer = new BufferWriter(buffer);

        writer.writeInt32(0); // offset 4
        writer.writeInt8(0); // offset 5
        const offset = writer.offset;

        expect(offset).toEqual(5);
    });

    it("writeInt8: int8 데이터를 쓰고 오프셋 1 이동", () => {
        const buffer = new ArrayBuffer(1);
        const view = new DataView(buffer);
        const writer = new BufferWriter(buffer);
        const item = 0xFF;

        writer.writeInt8(item);
        const data = view.getInt8(0);

        expect(data).toEqual(-1); // 0xFF = -1 (signed byte)
        expect(writer.offset).toEqual(1);
    });

    it("writeUInt8: unsigned int8 데이터를 쓰고 오프셋 1 이동", () => {
        const buffer = new ArrayBuffer(1);
        const view = new DataView(buffer);
        const writer = new BufferWriter(buffer);
        const item = 0xFF;

        writer.writeInt8(item);
        const data = view.getUint8(0);

        expect(data).toEqual(0xFF); // 0xFF = 255 (unsigned byte)
        expect(writer.offset).toEqual(1);
    });

    it("writeInt16: int16 데이터를 쓰고 오프셋 2 이동", () => {
        const buffer = new ArrayBuffer(2);
        const view = new DataView(buffer);
        const writer = new BufferWriter(buffer);
        const item = 0xFFFF

        writer.writeInt16(item);
        const data = view.getInt16(0);

        expect(data).toEqual(-1); // 0xFF = -1 (signed byte)
        expect(writer.offset).toEqual(2);
    });

    it("writeUInt16: unsigned int16 데이터를 쓰고 오프셋 2 이동", () => {
        const buffer = new ArrayBuffer(2);
        const view = new DataView(buffer);
        const writer = new BufferWriter(buffer);
        const item = 0xFFFF;

        writer.writeUInt16(item);
        const data = view.getUint16(0);

        expect(data).toEqual(item); // 0xFFFF (unsigned byte)
        expect(writer.offset).toEqual(2);
    });

    it("writeInt32: int32 데이터를 쓰고 오프셋 4 이동", () => {
        const buffer = new ArrayBuffer(4);
        const view = new DataView(buffer);
        const writer = new BufferWriter(buffer);
        const item = 0xFFFFFFFF;

        writer.writeInt32(item);
        const data = view.getInt32(0);

        expect(data).toEqual(-1); // 0xFF = -1 (signed byte)
        expect(writer.offset).toEqual(4);
    });

    it("writeUInt32: unsigned int32 데이터를 쓰고 오프셋 4 이동", () => {
        const buffer = new ArrayBuffer(4);
        const view = new DataView(buffer);
        const writer = new BufferWriter(buffer);
        const item = 0xFFFFFFFF;

        writer.writeUInt32(item);
        const data = view.getUint32(0);

        expect(data).toEqual(item);
        expect(writer.offset).toEqual(4);
    });

    it("writeInt64: int64 데이터를 쓰고 오프셋 8 이동", () => {

        const buffer = new ArrayBuffer(8);
        const view = new DataView(buffer);
        const writer = new BufferWriter(buffer);
        const item = 0xFFFF_FFFF_FFFF_FFFFn;


        writer.writeInt64(item);
        const data = view.getBigInt64(0);

        expect(data).toEqual(-1n);
        expect(writer.offset).toEqual(8);
    });

    
    it("writeUInt64: unsigned int64 데이터를 쓰고 오프셋 8 이동", () => {

        const buffer = new ArrayBuffer(8);
        const view = new DataView(buffer);
        const writer = new BufferWriter(buffer);
        const item = 0xFFFF_FFFF_FFFF_FFFFn;


        writer.writeUInt64(item);
        const data = view.getBigUint64(0);

        expect(data).toEqual(item);
        expect(writer.offset).toEqual(8);
    });


    it("writeFloat32: float32 데이터 쓰고 오프셋 4 이동", () => {

        const buffer = new ArrayBuffer(4);
        const view = new DataView(buffer);
        const writer = new BufferWriter(buffer);
        
        writer.writeFloat32(0.123456789);

        const data = view.getFloat32(0);

        expect(data).toEqual(0.12345679104328156); // 정확도 이슈로 0.123456789 그대로 저장 안됨
        expect(writer.offset).toEqual(4);
    });

    it("readFloat64: float64 데이터 읽고 오프셋 8 이동", () => {
        const buffer = new ArrayBuffer(8);
        const view = new DataView(buffer);
        const reader = new BufferWriter(buffer);
        reader.writeFloat64(0.123456789);

        const data = view.getFloat64(0);

        expect(data).toEqual(0.123456789);
        expect(reader.offset).toEqual(8);
    });

    it("writeString: string 데이터 읽고, 지정된 길이만큼 오프셋 이동", () => {
        const buffer = new ArrayBuffer(3);
        const view = new DataView(buffer);
       
        const reader = new BufferWriter(buffer);
        const length = reader.writeString('ABC');
        expect(length).toEqual(3);

        expect(view.getUint8(0)).toEqual('ABC'.charCodeAt(0));
        expect(view.getUint8(1)).toEqual('ABC'.charCodeAt(1));
        expect(view.getUint8(2)).toEqual('ABC'.charCodeAt(2));
        expect(reader.offset).toEqual(3);
    });

    it("maxOffset: 버퍼의 최대 바이트 길이 반환", () => {
        const length = 3;
        const buffer = new ArrayBuffer(length); // 바이트 길이 3
        const reader = new BufferWriter(buffer);
        
        expect(reader.maxOffset).toEqual(length);
    });

    describe("resize", ()=> {
        it("내부 버퍼 모드에서 버퍼 크기 재설정", () => {
            const writer = new BufferWriter(10);
            writer.resize(20);
    
            expect(writer.maxOffset).toEqual(20);
        });

        it("외부 버퍼 사용 시 버퍼 크기 증가 X", () => {
            const writer = new BufferWriter(new ArrayBuffer(10));

            expect(() => {
                writer.resize(10);
            }).toThrow("external buffer");
        });

        it("buffer size는 양수여야 함(크기 감소는 불가능)", () => {
            const writer = new BufferWriter(10);

            expect(() => {
                writer.resize(-5);
            }).toThrow("size must be bigger");
        });
    });

    it("resizeBufferIfNeeded: 버퍼 크기가 부족한 경우 증가", () => {
        const writer = new BufferWriter(8);
        writer.writeInt32(10);
        writer.writeInt16(10);
        // 6byte 사용
        writer.resizeBufferIfNeeded(30);
        // 30byte 데이터 들어와야 함 -> 총 36byte
        // 8 16 32 64 로 64까지 증가
        expect(writer.maxOffset).toEqual(64);
    });
});