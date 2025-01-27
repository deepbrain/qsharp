// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import * as vscode from "vscode";
import { getCompilerWorker, log } from "qsharp-lang";
import { isQsharpDocument } from "./common";

const generateQirTimeoutMs = 30000;

let compilerWorkerScriptPath: string;

export class QirGenerationError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "QirGenerationError";
  }
}

export async function getQirForActiveWindow(): Promise<string> {
  let result = "";
  const editor = vscode.window.activeTextEditor;
  if (!editor || !isQsharpDocument(editor.document)) {
    throw new QirGenerationError(
      "The currently active window is not a Q# file"
    );
  }

  // Check that the current target is base profile, and current doc has no errors.
  const targetProfile = vscode.workspace
    .getConfiguration("Q#")
    .get<string>("targetProfile", "full");
  if (targetProfile !== "base") {
    throw new QirGenerationError(
      "Submitting to Azure is only supported when targeting the QIR base profile. " +
        "Please update the QIR target via the status bar selector or extension settings."
    );
  }

  // Get the diagnostics for the current document.
  const diagnostics = await vscode.languages.getDiagnostics(
    editor.document.uri
  );
  if (diagnostics?.length > 0) {
    throw new QirGenerationError(
      "The current program contains errors that must be fixed before submitting to Azure"
    );
  }

  const code = editor.document.getText();

  // Create a temporary worker just to get the QIR, as it may loop/panic during codegen.
  // Let it run for max 10 seconds, then terminate it if not complete.
  const worker = getCompilerWorker(compilerWorkerScriptPath);
  const compilerTimeout = setTimeout(() => {
    worker.terminate();
  }, generateQirTimeoutMs);
  try {
    result = await worker.getQir(code);
    clearTimeout(compilerTimeout);
  } catch (e: any) {
    log.error("Codegen error. ", e.toString());
    throw new QirGenerationError(
      "Code generation failed. Please ensure the code is compatible with the QIR base profile " +
        "by setting the target QIR profile to 'base' and fixing any errors."
    );
  } finally {
    worker.terminate();
  }

  return result;
}

export function initCodegen(context: vscode.ExtensionContext) {
  compilerWorkerScriptPath = vscode.Uri.joinPath(
    context.extensionUri,
    "./out/compilerWorker.js"
  ).toString();

  context.subscriptions.push(
    vscode.commands.registerCommand("qsharp-vscode.getQir", async () => {
      try {
        const qir = await getQirForActiveWindow();
        const qirDoc = await vscode.workspace.openTextDocument({
          language: "llvm",
          content: qir,
        });
        await vscode.window.showTextDocument(qirDoc);
      } catch (e: any) {
        log.error("QIR generation failed. ", e);
        if (e.name === "QirGenerationError") {
          vscode.window.showErrorMessage(e.message);
        }
      }
    })
  );
}
