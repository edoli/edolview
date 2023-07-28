import * as vscode from 'vscode';
import * as zlib from 'zlib';
import * as net from 'net';
import * as path from 'path';
import * as fs from 'fs/promises';
import { pythonCodeBuilder } from './pythonCodeBuilder';
import CvVariable from './CvVariable';
import PromiseSocket from './PromiseSocket';


export class EdolViewImageHandler{

    constructor() {
    }
    async addImageFile(filePath: string) {

        const host: string = vscode.workspace.getConfiguration().get("edolview.host") ?? "127.0.0.1";
        const port: number = vscode.workspace.getConfiguration().get("edolview.port") ?? 21734;

        const data = await fs.readFile(filePath);

        const netSocket = new net.Socket();
        
        const socket = new PromiseSocket(netSocket);
        await socket.connectPromise(port, host);

        const name = path.basename(filePath);
        const extra = JSON.stringify({
            compression: 'cv'
        });

        const lengthBuffer = await Buffer.allocUnsafe(12);
        let offset = 0;
        offset = lengthBuffer.writeInt32BE(name.length, offset);
        offset = lengthBuffer.writeInt32BE(extra.length, offset);
        offset = lengthBuffer.writeInt32BE(data.length, offset);

        await socket.writeBuffer(lengthBuffer);
        
        await socket.writeStr(name);
        await socket.writeStr(extra);
        await socket.writeBuffer(data);

        await socket.end();
    }

    async addImagePython(variable: Variable) {
        const session = vscode.debug.activeDebugSession;

        if(session) {            
            // const variables: Array<Variable> = (await session.customRequest('variables', {variablesReference: varRef})).variables;

            // const response = await session.customRequest('variables', {variablesReference: varRef});
            
            // let spVariable = response.variables.find((v: Variable) => v.name === 'special variables');
            // const spResponse = await session.customRequest('variables', {variablesReference: spVariable.variablesReference});
            
            // let internalVariable = spResponse.variables.find((v: Variable) => v.name === '__internals__');
            // const internalResponse = await session.customRequest('variables', {variablesReference: internalVariable.variablesReference});

            // let dataVariable = internalResponse.variables.find((v: Variable) => v.name === '\'data\'');
            // let ctypesVariable = internalResponse.variables.find((v: Variable) => v.name === '\'ctypes\'');
            
            // const dataResponse = await session.customRequest('variables', {variablesReference: dataVariable.variablesReference});
            // const ctypesResponse = await session.customRequest('variables', {variablesReference: ctypesVariable.variablesReference});

            // const memoryAddr = parseInt(ctypesResponse.variables.find((v: Variable) => v.name === 'data').value);
            // const nBytes = parseInt(dataResponse.variables.find((v: Variable) => v.name === 'nbytes').value);

            const threadRes = await session.customRequest('threads', {});
            const threads = threadRes.threads;

            const stackTraceRes = await session.customRequest('stackTrace', { threadId: threads[0].id });
            const stacks = stackTraceRes.stackFrames;

            const callStack = stacks[0].id;

            const host: string = vscode.workspace.getConfiguration().get("edolview.host") ?? "127.0.0.1";
            const port: number = vscode.workspace.getConfiguration().get("edolview.port") ?? 21734;
            
            const pythonCode = pythonCodeBuilder(variable.evaluateName, host, port);

            await session.customRequest("evaluate", { expression: pythonCode, frameId: callStack, context: 'repl' });
        }
    }

    async addImageCpp(varName: string, varRef: number) {
        const session = vscode.debug.activeDebugSession;

        if(session) {
            const variables: Array<Variable> = (await session.customRequest('variables', {variablesReference: varRef})).variables;

            const cvVariable = CvVariable.parseCvVariable(variables);
            if (cvVariable !== null) {
                const buf = await cvVariable.readData(session);
            
                const compressedBuf = zlib.deflateSync(buf);
            
                const host: string = vscode.workspace.getConfiguration().get("edolview.host") ?? "127.0.0.1";
                const port: number = vscode.workspace.getConfiguration().get("edolview.port") ?? 21734;
    
                const socket = net.connect({
                    host: host,
                    port: port
                });
                
                socket.write('');
                // TODO
            }
        }
    }
}