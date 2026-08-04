import {cn} from '@/lib/utils';
import { saturate } from "@/lib/math";
import { useRef } from 'react';
import { copyToClipboard } from '@/lib/clipboard';
import { CopyIcon } from 'lucide-react';

type HexViewerProps = {
    bytes: number[],
    focusItemHandler: (idx: number) => void;
    displayFunc: (num: number) => string;
    charPerItem: number;
    selectedIdx: number;
    className?: string;
    itemPerLine?: number;
} & React.HTMLAttributes<HTMLDivElement>;

function HexViewer({ bytes, focusItemHandler, displayFunc, charPerItem, selectedIdx, className, itemPerLine = 8, ...props }: HexViewerProps) {
    const divRef = useRef<HTMLDivElement>(null);

    const onCopy = async () => {
        const data = divRef.current?.textContent;
        if (!data) return;

        await copyToClipboard(data);
    };

    const focusElement = (idx: number) => {
        // + 가 없어서 byte끼리만 이동 가능 = length - 1
        const targetIdx = saturate(idx, 0, bytes.length - 1);

        if (targetIdx === selectedIdx) return;
        focusItemHandler(targetIdx);
    };


    const keyHandler = (e: React.KeyboardEvent<HTMLDivElement>) => {
        const key = e.key;

        switch (key) {
            case 'ArrowLeft':
                focusElement(selectedIdx - 1);
                break;
            case 'ArrowRight':
                focusElement(selectedIdx + 1);
                break;
            case 'ArrowUp':
                focusElement(selectedIdx - itemPerLine);
                break;
            case 'ArrowDown':
                focusElement(selectedIdx + itemPerLine);
                break;
        }
    }

    return (
        <div className="space-y-2">
            <div className="flex flex-row justify-end">
                <button
                    className="p-1 border rounded-md border-gray-300 active:border-gray-400"
                    onClick={onCopy}
                    aria-label='copy data'
                    title="copy data"
                >
                    <CopyIcon color="gray" width={20} height={20} />
                </button>
            </div>
            <hr />
            <div
                ref={divRef}
                tabIndex={0}
                className={cn('grid font-mono justify-items-center items-stretch gap-1', className)}
                onKeyDown={keyHandler}
                style={{
                    gridTemplateColumns: `repeat(${itemPerLine}, 1fr)`,
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
                        style={{ width: `${charPerItem}rem` }}
                    >{displayFunc(it)}</div>
                ))}
            </div>
        </div>
    )
};

export default HexViewer;
