

function getHexFromString(str: string): string{
    let result = "";
    const hexChars = "0123456789ABCDEFabcdef";

    if(str.charAt(0) === '0' && str.charAt(1).toLowerCase() === 'x'){
        result = "0x";
        for(var i = 2; i < str.length; i++){
            if(hexChars.includes(str.charAt(i))){
                result += str.charAt(i);
            }else{
                break;
            }
        }
    } else {
        result = "0x00";
    }

    return result;
}

export { getHexFromString };