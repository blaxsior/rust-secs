class BufferWriter {
  private _buffer: ArrayBuffer;
  private view: DataView;
  private _offset: number;
  private static BUF_DEFAULT_SIZE = 256;
  private isExternalBuffer: boolean;

  constructor(arg: ArrayBuffer | number = BufferWriter.BUF_DEFAULT_SIZE) {
    if (arg instanceof ArrayBuffer) {
      this._buffer = arg;
      this.isExternalBuffer = true;
    } else {
      this._buffer = new ArrayBuffer(arg);
      this.isExternalBuffer = false;
    }

    this.view = new DataView(this._buffer);
    this._offset = 0;
  }
  
  /**
   * 버퍼의 크기를 늘린다. 내부 버퍼 이용 시에만 가능하다.
   * @param size
   */
  public resize(size: number) {
    if (this.isExternalBuffer)
      throw new Error("buffer writer use external buffer now.");
    if (size <= 0) throw new Error("size must be bigger than 0");

    const newBuffer = new Uint8Array(size);
    const oldBuffer = new Uint8Array(this._buffer);

    newBuffer.set(oldBuffer);
    this._buffer = newBuffer.buffer;
    this.view = new DataView(this._buffer);
  }

  resizeBufferIfNeeded(dataLength: number) {
    let newSize = this.maxOffset;

    while (this.offset + dataLength > newSize) {
      newSize *= 2;
    }
    
    if (newSize !== this.maxOffset) {
      this.resize(newSize);
    }
  }

  public get offset() {
    return this._offset;
  }

  public get buffer() {
    return this._buffer.slice(0, this._offset);
  }

  public get maxOffset() {
    return this.view.byteLength;
  }

  public resetOffset() {
    this._offset = 0;
  }

  public writeInt8(item: number) {
    this.resizeBufferIfNeeded(1);
    this.view.setInt8(this._offset, item);
    this._offset += 1;
  }

  public writeInt16(item: number) {
    this.resizeBufferIfNeeded(2);
    this.view.setInt16(this._offset, item);
    this._offset += 2;
  }

  public writeInt32(item: number) {
    this.resizeBufferIfNeeded(4);
    this.view.setInt32(this._offset, item);
    this._offset += 4;
  }

  public writeInt64(item: bigint) {
    this.resizeBufferIfNeeded(8);
    this.view.setBigInt64(this._offset, item);
    this._offset += 8;
  }

  public writeUInt8(item: number) {
    this.resizeBufferIfNeeded(1);
    this.view.setUint8(this._offset, item);
    this._offset += 1;
  }

  public writeUInt16(item: number) {
    this.resizeBufferIfNeeded(2);
    this.view.setUint16(this._offset, item);
    this._offset += 2;
  }

  public writeUInt32(item: number) {
    this.resizeBufferIfNeeded(4);
    this.view.setUint32(this._offset, item);
    this._offset += 4;
  }

  public writeUInt64(item: bigint) {
    this.resizeBufferIfNeeded(8);
    this.view.setBigUint64(this._offset, item);
    this._offset += 8;
  }

  public writeFloat32(item: number) {
    this.resizeBufferIfNeeded(4);
    this.view.setFloat32(this._offset, item);
    this._offset += 4;
  }

  public writeFloat64(item: number) {
    this.resizeBufferIfNeeded(8);
    this.view.setFloat64(this._offset, item);
    this._offset += 8;
  }

  /**
   * 문자열을 쓴다
   * @returns 작성한 문자열의 길이
   */
  writeString(item: string): number {
    const encoder = new TextEncoder();
    const data = encoder.encode(item);
    this.resizeBufferIfNeeded(data.byteLength);
    const uint8View = new Uint8Array(this._buffer);

    // 데이터 쓰기
    uint8View.set(data, this._offset);

    // 오프셋 갱신
    this._offset += data.length;

    // 문자열 길이 반환
    return data.length;
  }
}

export { BufferWriter };
