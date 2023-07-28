import * as vscode from 'vscode';
import { getHexFromString } from "./utils";

class CvVariable {
    flags: number = 0;
    dims: number = 0;
    cols: number = 0;
    rows: number = 0;
    datastart: string = "";
    dataend: string = "";

    nbytes: number = 0;
    channels: number = 0;
    cvType: number = 0;
    dtype: number = 0;
    
    static parseCvVariable(variables: Array<Variable>): CvVariable | null {

        let cvVariable = new CvVariable();

        variables.forEach((element: any) => {
            const name = element.name;
            switch (name) {
                case 'flags':
                    cvVariable.flags = parseInt(element.value);
                    break;
                case 'dims':
                    cvVariable.dims = parseInt(element.value);
                    break;
                case 'rows':
                    cvVariable.rows = parseInt(element.value);
                    break;
                case 'cols':
                    cvVariable.cols = parseInt(element.value);
                    break;
                case 'datastart':
                    cvVariable.datastart = getHexFromString(element.value);
                    break;
                case 'dataend':
                    cvVariable.dataend = getHexFromString(element.value);
                    break;
                default:
                    break;
            }
        });

        if (cvVariable.dims === 0 || cvVariable.cols === 0 || cvVariable.rows === 0 ||
            cvVariable.flags === 0 || cvVariable.datastart === "" || cvVariable.dataend === "") {
            return null;
        }

        const type = cvVariable.flags & 0xFFF;

        cvVariable.cvType = type;
        cvVariable.channels = (type >> 3) + 1;
        cvVariable.dtype = type & 0x7;

        cvVariable.nbytes = parseInt(cvVariable.dataend) - parseInt(cvVariable.datastart);

        return cvVariable;
    }

    async readData(session: vscode.DebugSession): Promise<Buffer> {
        
        const response = await session.customRequest('readMemory', {memoryReference: this.datastart, offset: 0, count: this.nbytes});
        const data: string = response.data;
        const buf = Buffer.from(data, 'base64');
        return buf;
    }
}

export default CvVariable;