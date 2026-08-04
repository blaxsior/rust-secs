import { Secs2Variant } from "@/types/secs2";

export const ItemSMLS =
    ['L', 'B', 'BOOLEAN', 'A',
        // 'J' |
        'I8', 'I1', 'I2', 'I4',
        'F8', 'F4',
        'U8', 'U1', 'U2', 'U4'] as const;

export const SMLMapping: Record<Secs2Variant['format'], string> = {
    'list': "L",
    'binary' : "B",
    'boolean' : "BOOLEAN",
'ascii' : "A",
    'int8' : "I8",
    'int1' : "I1",
    'int2' : "I2",
    'int4' : "I4",
    'float8' : "F8",
    'float4' : "F4",
    'uint8' : "U8",
    'uint1' : "U1",
    'uint2' : "U2",
    'uint4' : "U4"
};

