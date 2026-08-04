/**
 * 최소 / 최대 범위로 주어진 값을 잘라 반환
 * 
 */
function saturate(num: number, min: number, max: number) {
    if(min > max) throw new Error("saturate: must min <= max");

    if(num < min) return min;
    else if(num > max) return max;
    return num;
}

export {
    saturate
}