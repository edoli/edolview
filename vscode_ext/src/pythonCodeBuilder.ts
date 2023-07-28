
const pythonCode = `
import socket
import json
from struct import pack
import importlib.util
import numpy as np
import zlib

class EdolView:
    def __init__(self, host='127.0.0.1', port=21734):
        self.host = host
        self.port = port

    def send_image(self, name:str, image, extra={}):
        # convert torch to numpy array
        if type(image) != np.ndarray:
            torch_spec = importlib.util.find_spec('torch')

            if torch_spec is not None:
                import torch

                if type(image) == torch.Tensor:
                    if hasattr(image, 'detach'):
                        image = image.detach()

                    if hasattr(image, 'cpu'):
                        image = image.cpu()
                        
                    if hasattr(image, 'numpy'):
                        image = image.numpy()
                        
        if type(image) != np.ndarray:
            raise Exception('image should be np.ndarray')
        
        initial_shape = image.shape
        dtype = image.dtype
        
        # Try to convert (B, C, H, W) or (C, H, W) to (H, W, C)
        if len(image.shape) == 4:
            image = image[0, ...]

        if image.shape[-1] > 4:
            image = image.transpose(1, 2, 0)        
            
        if image.shape[-1] > 4:
            raise Exception('image dimension not valid shape: ' + str(initial_shape))

        # Convert to PNG if cv2 is  installed and image is integer. Otherwise use zlib compress
        cv2_spec = importlib.util.find_spec('cv2')
            
        if np.issubdtype(dtype, np.integer) and cv2_spec is not None:
            import cv2

            retval, buf = cv2.imencode('.png', image[:, :, ::-1])
            buf_bytes = buf.tobytes()

            extra['compression'] = 'png'
        else:
            if not image.data.c_contiguous:
                image = image.copy()

            buf_bytes = zlib.compress(image.data)

            extra['compression'] = 'zlib'

        extra['nbytes'] = image.nbytes
        extra['shape'] = image.shape
        extra['dtype'] = image.dtype.name

        extra_str = json.dumps(extra)

        # Encode string and send to socket
        name_bytes = name.encode('utf-8')
        extra_bytes = extra_str.encode('utf-8')

        with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
            s.connect((self.host, self.port))

            s.send(pack('!i', len(name_bytes)))
            s.send(pack('!i', len(extra_bytes)))
            s.send(pack('!i', len(buf_bytes)))
            s.send(name_bytes)
            s.send(extra_bytes)
            s.sendall(buf_bytes)
            s.close()
`;

const pythonCodeBuilder = (evaluateName: string, host: string, port: number) => {
    const evaluateNameEscape = evaluateName.replaceAll('\'', '\\\'').replaceAll('\"', '\\\"');
    return `
${pythonCode}

EdolView(host='${host}', port=${port}).send_image('${evaluateNameEscape}', ${evaluateName})
    `;
};

export { pythonCodeBuilder };