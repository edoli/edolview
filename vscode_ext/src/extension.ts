import * as vscode from "vscode";
import { EdolViewImageHandler } from "./edolViewService";



export function activate(context: vscode.ExtensionContext) {
  try {	
		let imagesHandler = new EdolViewImageHandler();

    let variableImageViewFile = vscode.commands.registerCommand('edolview.image.file', async (file: vscode.Uri) =>{

      // Variable is clicked
      const path = file.fsPath;
      
      imagesHandler.addImageFile(path);
    });

    let variableImageViewPython = vscode.commands.registerCommand('edolview.image.python', async (variableObject) =>{

      // Variable is clicked
      const variable = variableObject.variable;
      
      imagesHandler.addImagePython(variable);
    });

    let variableImageViewCpp = vscode.commands.registerCommand('edolview.image.cpp', async (variableObject) =>{

      // Variable is clicked
      const varName = variableObject.variable.name; 
      const varref = variableObject.variable.variablesReference;
      
      imagesHandler.addImageCpp(varName, varref);
    });
  
    context.subscriptions.push(variableImageViewFile);
    context.subscriptions.push(variableImageViewPython);
    context.subscriptions.push(variableImageViewCpp);
  } catch {
		vscode.window.showErrorMessage("Couldnt activate Extension");
	}
}

// This method is called when your extension is deactivated
export function deactivate() {}