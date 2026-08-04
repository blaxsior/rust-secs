"use client";

import { SMLMapping } from "@/core/secs/const";
import { itemValidator as validate } from "./util";
import { Secs2Format, Secs2Variant } from "@/types/secs2";

type Secs2ComponentProps = {
    item: Secs2Variant;
    onChange: (newValue: Secs2Variant) => void;
    onRemove?: () => void;
    root?: boolean;
}

const SECS2_FORMATS: Secs2Format[] = [
    "list",
    "binary",
    "boolean",
    "ascii",
    "int8",
    "int1",
    "int2",
    "int4",
    "float8",
    "float4",
    "uint8",
    "uint1",
    "uint2",
    "uint4",
];

function createEmptyVariant(format: Secs2Format): Secs2Variant {
    switch (format) {
        case "list":
            return { format, value: [] };
        case "ascii":
            return { format, value: "" };
        case "binary":
        case "boolean":
        case "int8":
        case "int1":
        case "int2":
        case "int4":
        case "float8":
        case "float4":
        case "uint8":
        case "uint1":
        case "uint2":
        case "uint4":
            return { format, value: [] } as Secs2Variant;
    }
}

function Secs2Component({ item, onChange, onRemove, root = false }: Secs2ComponentProps) {
    const changeFormat = (format: Secs2Format) => {
        onChange(createEmptyVariant(format));
    };

    const changeScalarValue = (value: string, index: number) => {
        if (item.format === "list") return;

        if (item.format === "ascii") {
            onChange({ format: item.format, value });
            return;
        }

        if (value !== "" && !validate(SMLMapping[item.format])(value)) return;

        const nextValue = [...item.value];
        nextValue[index] = value === "" ? 0 : Number(value);
        onChange({ format: item.format, value: nextValue } as Secs2Variant);
    };

    const addValue = () => {
        if (item.format === "list") return;
        if (item.format === "ascii") return;

        onChange({ format: item.format, value: [...item.value, 0] } as Secs2Variant);
    };

    const removeValue = () => {
        if (item.format === "list") return;
        if (item.format === "ascii") return;

        onChange({ format: item.format, value: item.value.slice(0, -1) } as Secs2Variant);
    };

    const addChild = () => {
        if (item.format !== "list") return;
        onChange({
            format: "list",
            value: [...item.value, createEmptyVariant("ascii")],
        });
    };

    const updateChild = (index: number, newChild: Secs2Variant) => {
        if (item.format !== "list") return;

        const nextValue = [...item.value];
        nextValue[index] = newChild;
        onChange({ format: "list", value: nextValue });
    };

    const removeChild = (index: number) => {
        if (item.format !== "list") return;

        onChange({
            format: "list",
            value: item.value.filter((_, itemIndex) => itemIndex !== index),
        });
    };

    return (
        <div className="space-y-2 rounded-xl border border-slate-200 bg-white/80 p-3">
            <div className="flex flex-wrap items-center gap-2">
                <span className="font-mono text-sm text-slate-500">&lt;</span>
                <select
                    value={item.format}
                    onChange={(event) => changeFormat(event.target.value as Secs2Format)}
                    className="rounded-md border border-slate-300 bg-white px-2 py-1 text-sm"
                >
                    {SECS2_FORMATS.map((format) => (
                        <option key={format} value={format}>
                            {SMLMapping[format]}
                        </option>
                    ))}
                </select>

                <span className="font-mono text-xs text-slate-500">
                    [{item.format === "ascii" ? item.value.length : item.value.length}]
                </span>

                {item.format === "list" ? (
                    <button
                        type="button"
                        onClick={addChild}
                        className="rounded-md border border-slate-300 px-2 py-1 text-xs font-medium text-slate-700 active:bg-slate-100"
                    >
                        add
                    </button>
                ) : item.format === "ascii" ? (
                    <input
                        value={item.value}
                        onChange={(event) => changeScalarValue(event.target.value, 0)}
                        className="min-w-48 flex-1 rounded-md border border-slate-300 px-2 py-1 font-mono text-sm"
                        placeholder="ASCII text"
                    />
                ) : (
                    <>
                        {item.value.map((value, index) => (
                            <input
                                key={index}
                                value={value}
                                onChange={(event) => changeScalarValue(event.target.value, index)}
                                className="w-20 rounded-md border border-slate-300 px-2 py-1 font-mono text-sm"
                            />
                        ))}
                        <button
                            type="button"
                            onClick={addValue}
                            className="rounded-md border border-slate-300 px-2 py-1 text-xs font-medium text-slate-700 active:bg-slate-100"
                        >
                            +
                        </button>
                        <button
                            type="button"
                            onClick={removeValue}
                            className="rounded-md border border-slate-300 px-2 py-1 text-xs font-medium text-slate-700 active:bg-slate-100"
                        >
                            -
                        </button>
                    </>
                )}

                {!root && onRemove ? (
                    <button
                        type="button"
                        onClick={onRemove}
                        className="rounded-md border border-red-300 px-2 py-1 text-xs font-medium text-red-700 active:bg-red-50"
                    >
                        remove
                    </button>
                ) : null}
                <span className="font-mono text-sm text-slate-500">&gt;</span>
            </div>

            {item.format === "list" && item.value.length > 0 ? (
                <div className="ml-4 space-y-2 border-l border-slate-200 pl-3">
                    {item.value.map((child, index) => (
                        <Secs2Component
                            key={index}
                            item={child}
                            onChange={(newChild) => updateChild(index, newChild)}
                            onRemove={() => removeChild(index)}
                        />
                    ))}
                </div>
            ) : null}
        </div>
    );
}

export default Secs2Component;
