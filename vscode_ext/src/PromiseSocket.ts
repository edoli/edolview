import * as net from 'net';

function promisify(f: Function, thisArg: any): (...args: any[]) => Promise<void> {
    return function (...args: any[]) {
        return new Promise((resolve, reject) => {
            function callback(err: Error, ...results: any[]) {
                if (err) {
                    reject(err);
                } else {
                    resolve();
                }
            }
      
            args.push(callback);

            f.call(thisArg, ...args);
        });
    };
}

function promisifyResult<T>(f: Function, thisArg: any): (...args: any[]) => Promise<T> {
    return function (...args: any[]) {
        return new Promise((resolve, reject) => {
            function callback(err: Error, ...results: any[]) {
                if (err) {
                    reject(err);
                } else {
                    resolve(results[0]);
                }
            }
      
            args.push(callback);

            f.call(thisArg, ...args);
        });
    };
}

class PromiseSocket extends net.Socket {
    writeBuffer: (buffer: Uint8Array | string) => Promise<void> = promisify(super.write, this);
    writeStr: (str: Uint8Array | string, encoding?: BufferEncoding) => Promise<void> = promisify(super.write, this);

    connectPromise: (port: number, host: string) => Promise<void> = promisify(super.connect, this);
}

export default PromiseSocket;