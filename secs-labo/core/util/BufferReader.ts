class BufferReader {
    private buffer: ArrayBuffer;
    private view: DataView;
    private _offset: number;

    constructor(buffer: ArrayBuffer) {
        this.buffer = buffer;
        this.view = new DataView(buffer);
        this._offset = 0;
    }

    public get offset() {
        return this._offset;
    }

    public get maxOffset() {
        return this.view.byteLength;
    }

    public resetOffset() {
        this._offset = 0;
    }

    public readInt8(): number {
        const data = this.view.getInt8(this._offset);
        this._offset += 1;

        return data;
    }

    public readInt16(): number {
        const data = this.view.getInt16(this._offset);
        this._offset += 2;

        return data;
    }

    public readInt32(): number {
        const data = this.view.getInt32(this._offset);
        this._offset += 4;

        return data;
    }

    public readInt64(): bigint {
        const data = this.view.getBigInt64(this._offset);
        this._offset += 8;

        return data;
    }

    public readUInt8(): number {
        const data = this.view.getUint8(this._offset);
        this._offset += 1;

        return data;
    }

    public readUInt16(): number {
        const data = this.view.getUint16(this._offset);
        this._offset += 2;

        return data;
    }

    public readUInt32(): number {
        const data = this.view.getUint32(this._offset);
        this._offset += 4;

        return data;
    }

    public readUInt64(): bigint {
        const data = this.view.getBigUint64(this._offset);
        this._offset += 8;

        return data;
    }

    public readFloat32(): number {
        const data = this.view.getFloat32(this._offset);
        this._offset += 4;

        return data;
    }

    public readFloat64(): number {
        const data = this.view.getFloat64(this._offset);
        this._offset += 8;

        return data;
    }

    /**
     * 지정된 길이의 문자열을 읽는다.
     * @param length 문자열의 길이
     * @param label 문자열이 해석될 타입
     * @returns 지정된 길이의 문자열
     */
    readString(length: number, label: string = 'utf-8'): string {
        const buffer = this.buffer.slice(this._offset, this._offset + length);
        const decoder = new TextDecoder(label);
        const data = decoder.decode(buffer);
        this._offset += length;

        return data;
    }
}

export {
    BufferReader
}