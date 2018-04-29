/* tslint:disable */
import * as wasm from './simplex_noise_bg';

export function noise(arg0, arg1) {
    return wasm.noise(arg0, arg1);
}

export class Simplex {

                    static __construct(ptr) {
                        return new Simplex(ptr);
                    }

                    constructor(ptr) {
                        this.ptr = ptr;
                    }

                free() {
                    const ptr = this.ptr;
                    this.ptr = 0;
                    wasm.__wbg_simplex_free(ptr);
                }
            }

const TextDecoder = typeof window === 'object' && window.TextDecoder
    ? window.TextDecoder
    : require('util').TextDecoder;

let cachedDecoder = new TextDecoder('utf-8');

let cachedUint8Memory = null;
function getUint8Memory() {
    if (cachedUint8Memory === null ||
        cachedUint8Memory.buffer !== wasm.memory.buffer)
        cachedUint8Memory = new Uint8Array(wasm.memory.buffer);
    return cachedUint8Memory;
}

function getStringFromWasm(ptr, len) {
    return cachedDecoder.decode(getUint8Memory().slice(ptr, ptr + len));
}

export function __wbindgen_throw(ptr, len) {
    throw new Error(getStringFromWasm(ptr, len));
}

