import { LanguageClient, LanguageClientOptions, ServerOptions, TransportKind } from 'vscode-languageclient/node';
import * as vscode from 'vscode';
import * as path from 'path';


import {
    StreamInfo,
} from 'vscode-languageclient/node';

import * as net from 'net';


let client: LanguageClient | undefined;

export async function activate(context: vscode.ExtensionContext) {
    const serverCommand = "microcad-lsp";
    /*
        const serverOptions: ServerOptions = {
            run: {
                command: serverCommand,
                args: ["-l", "microcad-lsp.log"],
                transport: TransportKind.socket
            },
            debug: {
                command: serverCommand,
                args: ["-l", "microcad-lsp-debug.log"],
                transport: TransportKind.socket
            }
        };

    // With these server options you can connect to an already running LSP, which makes development more convenient.
    const serverOptions: ServerOptions = () => {
        return new Promise<StreamInfo>((resolve, reject) => {
            const port = 5007;
            const socket = net.connect(port, '127.0.0.1', () => {
                console.log(`Connected to language server on port ${port}`);
                resolve({
                    reader: socket,
                    writer: socket,
                });
            });

            socket.on('error', (err) => {
                console.error('Failed to connect to language server:', err);
                reject(err);
            });
        });
    };
    */

    // 1. Encapsulate startup logic
    function startLanguageServer() {
        const clientOptions: LanguageClientOptions = {
            documentSelector: [{ scheme: 'file', language: 'microcad' }],
            synchronize: {
                fileEvents: vscode.workspace.createFileSystemWatcher('**/*.mu')
            }
        };

        const serverOptions: ServerOptions = {
            run: {
                command: serverCommand,
                args: ["-l", "microcad-lsp.log"],
                transport: TransportKind.stdio // Changed to stdio for reliable process lifecycle management
            },
            debug: {
                command: serverCommand,
                args: ["-l", "microcad-lsp-debug.log"],
                transport: TransportKind.stdio
            }
        };

        client = new LanguageClient(
            'microcadLSP',
            'Microcad Language Server',
            serverOptions,
            clientOptions
        );
        context.subscriptions.push(client);

        // Start the client. This also launches the networking/process loop
        client.start();
    }

    const showPreviewCmd = vscode.commands.registerCommand(
        "microcad.showPreview",
        async () => {
            if (!client) { return; }

            const editor = vscode.window.activeTextEditor;
            if (!editor) {
                vscode.window.showErrorMessage("No active editor");
                return;
            }
            const uri = editor.document.uri;
            try {
                const result = await client.sendRequest("workspace/executeCommand", {
                    command: "microcad.showPreview",
                    arguments: [{ uri }]
                });

                vscode.window.showInformationMessage("Preview requested.");
            } catch (err) {
                vscode.window.showErrorMessage("Show Preview failed: " + err);
            }
        }
    );

    const minimizePreviewCmd = vscode.commands.registerCommand(
        "microcad.minimizePreview",
        async () => {
            if (!client) { return; }
            try {
                const result = await client.sendRequest("workspace/executeCommand", {
                    command: "microcad.minimizePreview",
                    arguments: []
                });

                vscode.window.showInformationMessage("Minimize Preview requested.");
            } catch (err) {
                vscode.window.showErrorMessage("Minimize Preview failed: " + err);
            }
        }
    );

    // 2. Register the Restart Command
    const restartCommand = vscode.commands.registerCommand('microcad.restartServer', async () => {
        if (!client) {
            vscode.window.showWarningMessage('Microcad server is not running.');
            startLanguageServer();
            return;
        }

        vscode.window.showInformationMessage('Restarting Microcad Language Server...');

        try {
            // Stop the current active process cleanly
            await client.stop();
            client = undefined;

            // Fire up a brand new process instance
            startLanguageServer();
            vscode.window.showInformationMessage('Microcad Language Server restarted successfully!');
        } catch (error) {
            vscode.window.showErrorMessage(`Failed to restart server: ${error}`);
        }
    });

    // Fire initial server initialization
    startLanguageServer();

    context.subscriptions.push(showPreviewCmd);
    context.subscriptions.push(minimizePreviewCmd);
    context.subscriptions.push(restartCommand);

    vscode.window.onDidChangeActiveTextEditor((editor) => {
        if (editor && client) {
            const uri = editor.document.uri.toString();
            client.sendNotification("custom/activeFileChanged", { uri });
        }
    });
}

export async function deactivate() {
    if (client) {
        await client.stop();
    }
}
