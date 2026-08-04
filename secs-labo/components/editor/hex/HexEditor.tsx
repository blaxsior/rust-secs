import { useId, useRef, useState } from "react";
import { cn } from '@/lib/utils';
import { saturate } from "@/lib/math";
import { ClipboardPasteIcon, CopyIcon, Trash2Icon } from 'lucide-react';
import { copyFromClipboard, copyToClipboard } from "@/lib/clipboard";
import type { ByteEditorProps } from "./hooks/useEditor";
import { Button } from "@/components/ui/button";
import { toast } from "@/components/ui/toast";

type HexEditorProps = {
  /**
   * editor이 readonly인지?
   */
  readonly?: boolean;
  /**
   * 슬롯 당 문자 개수 ex) hex = 2
   */
  charPerSlot: number;
  /**
   * 1줄 당 보여줄 슬롯 개수
   */
  slotPerLine?: number;
  /**
 * 슬롯에 부여할 이름
 */
  name?: string;
  validator: RegExp;
  /**
   * 들어온 숫자를 어떻게 보여줄 것인지
   * @param num 숫자
   * @returns 보여주고자 하는 내용
   */
  displayFunc: (num: number) => string;
  /**
   * 문자를 어떻게 숫자로 변환할 것인지
   * @param str 문자
   * @returns 변환된 숫자
   */
  parseFunc: (str: string) => number;
  className?: string;

} & ByteEditorProps & React.HTMLAttributes<HTMLDivElement>;

function HexEditor({ name, bytes, setBytes, updateItem: updateItemHandler, deleteItem: deleteItemHandler, focusItem: focusItemHandler, clearItem: clearItemHandler, displayFunc, parseFunc, charPerSlot: charPerItem, validator, selectedIdx, className, slotPerLine: itemPerLine = 8, readonly = false, ...props }: HexEditorProps) {
  const inputId = useId();
  const divRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLInputElement>(null);

  const inputStrRef = useRef<string>("");
  const [displayInput, setDisplayInput] = useState<string>("");

  const onCopy = async () => {
    const data = divRef.current?.textContent?.replace('+', '');
    if (!data) return;

    await copyToClipboard(data);

    toast.add({
      title: "Copy Complete",
      description: `success copy to clipboard`,
      type: "success"
    });
  };

  const onPaste = async () => {
    const strFromClipboard = await copyFromClipboard();
    const hasInvalidChar = [...strFromClipboard].some((char) => !validator.test(char));

    if (strFromClipboard.length === 0 || strFromClipboard.length % charPerItem !== 0 || hasInvalidChar) {
      toast.add({
        title: "Paste Failed",
        description: `invalid data: ${strFromClipboard}`,
        // timeout: 10000,
        type: "error"
      });
      return;
    }

    const chunks = strFromClipboard.match(new RegExp(`.{${charPerItem}}`, "g")) ?? [];
    const pastedBytes = chunks.map((chunk) => parseFunc(chunk));
    const hasInvalid = pastedBytes.some((v) => Number.isNaN(v));

    if (hasInvalid) {
      toast.add({
        title: "Paste Failed",
        description: "invalid data entered",
        type: "error"
      });
      return;
    }

    const insertIdx = Math.min(selectedIdx, bytes.length);

    setBytes([
      ...bytes.slice(0, insertIdx),
      ...pastedBytes,
      ...bytes.slice(insertIdx),
    ]);

    focusItemHandler(insertIdx + pastedBytes.length);
    toast.add({
      title: "Paste Complete",
      description: `${pastedBytes.length} item(s) inserted`,
      type: "success"
    });
  };

  const onClick = () => {
    inputRef.current?.focus();
  }

  const onBlur = () => {
    inputRef.current?.blur();
    inputStrRef.current = "";
    setDisplayInput(inputStrRef.current);
  }

  const isWriting = () => inputStrRef.current.length;

  const focusElement = (idx: number, addidx = 0) => {
    // 0 = 첫번째 엘리먼트 / bytes.length + 1 = 엘리먼트 추가 시
    const targetIdx = saturate(idx, 0, bytes.length + addidx);

    if (targetIdx === selectedIdx) return;
    if (isWriting()) updateItem(selectedIdx);
    focusItemHandler(targetIdx);
  }

  const clearItem = () => {
    clearItemHandler();
    // focusItemHandler(0);
    toast.add({
      title: "Clear Complete",
      description: `success to clear data`,
      type: "success"
    });
  }


  const updateItem = (idx: number) => {
    let number = parseFunc(inputStrRef.current);
    if (!isNaN(number)) updateItemHandler(idx, number);

    inputStrRef.current = "";
    setDisplayInput(inputStrRef.current);
  }

  const deleteItem = (idx: number) => {
    if (idx === bytes.length) idx -= 1;
    deleteItemHandler(idx);
  }

  const handleInput = (key: string) => {
    if (key.length > 1 || !validator.test(key)) return;

    inputStrRef.current += key;
    setDisplayInput(inputStrRef.current);

    if (inputStrRef.current.length < charPerItem) return;

    updateItem(selectedIdx);
    focusElement(selectedIdx + 1, 1);
  };

  const keyHandler = (e: React.KeyboardEvent<HTMLDivElement>) => {
    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "v") {
      if (readonly) return;
      e.preventDefault();
      void onPaste();
      return;
    }

    const key = e.key;

    switch (key) {
      case 'ArrowLeft':
        focusElement(selectedIdx - 1);
        break;
      case 'ArrowRight':
        const addIdx = isWriting() ? 1 : 0;
        focusElement(selectedIdx + 1, addIdx);
        break;
      case 'ArrowUp':
        focusElement(selectedIdx - itemPerLine);
        break;
      case 'ArrowDown':
        focusElement(selectedIdx + itemPerLine);
        break;
      case 'Backspace':
        if (isWriting()) {
          inputStrRef.current = inputStrRef.current.slice(0, inputStrRef.current.length - 1);
          setDisplayInput(inputStrRef.current);
        } else {
          deleteItem(selectedIdx);
          focusElement(selectedIdx - 1);
        }
        break;
      default:
        if(readonly) {
          // 정의된 액션이 아니면 값 입력으로 취급 -> readonly면 입력 무시
          return;
        }
        handleInput(key);
        break;
    }
  }

  return (
    <>
      <div className={cn("space-y-2 rounded-lg border border-border bg-background p-3 shadow-sm transition-colors focus-within:border-ring focus-within:ring-2 focus-within:ring-ring/30", className)}>
        <div className="flex flex-row justify-end items-center gap-2">
          {
            name && <h1 className="flex-1">{name}</h1>
          }
          <Button
            type="button"
            variant="outline"
            size="icon"
            onClick={onCopy}
            aria-label="copy data"
            title="copy data"
          >
            <CopyIcon className="size-4 text-muted-foreground" />
          </Button>
          {!readonly &&

            <><Button
              type="button"
              variant="outline"
              size="icon"
              onClick={onPaste}
              aria-label="paste data"
              title="paste data"
            >
              <ClipboardPasteIcon className="size-4 text-muted-foreground" />
            </Button>
              <Button
                type="button"
                variant="outline"
                size="icon"
                onClick={clearItem}
                aria-label="clear data"
                title="clear data"
              >
                <Trash2Icon className="size-4 text-muted-foreground" />
              </Button>
            </>
          }
        </div>
        <hr />
        <div
          tabIndex={0}
          ref={divRef}
          onFocus={onClick}
          onBlur={onBlur}
          onClick={onClick}
          className={cn('grid font-mono justify-items-center items-stretch outline-none', className)}
          // onKeyDown={keyHandler}
          style={{
            gridTemplateColumns: `repeat(${itemPerLine}, 1fr)`
          }}
          {...props}
        >
          {bytes.map((it, idx) => (
            <div
              aria-label={`${props["aria-label"] ?? ""}_item-${idx}`}
              key={idx}
              onClick={() => focusElement(idx)}
              className={cn(`p-1 text-center`,
                selectedIdx === idx && "bg-yellow-300")}
              style={{ width: `${charPerItem}em` }}
            >
              {selectedIdx === idx && isWriting() ? (
                displayInput
              ) : (
                displayFunc(it)
              )}
            </div>
          ))}
          <div
            aria-label={`${props["aria-label"] ?? ""}_item-add`}
            onClick={() => focusElement(bytes.length)}
            className={cn(`p-1 text-center border border-gray-400`,
              selectedIdx === bytes.length && "bg-yellow-300")}
            style={{ width: `${charPerItem}em` }}
          >
            {selectedIdx === bytes.length && isWriting() ? (
              displayInput
            ) : (
              "+"
            )}</div>
        </div>
        <input className="w-0 h-0 outline-0 opacity-0"
          ref={inputRef}
          value={""}
          onKeyDown={keyHandler}
          onChange={(e) => { e.preventDefault(); }}
          id={inputId}
        />
      </div>

    </>
  )
};

export default HexEditor;
