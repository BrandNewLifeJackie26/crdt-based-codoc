import React, { useState, useEffect } from 'react';
import MonacoEditor, { EditorDidMount } from 'react-monaco-editor';
import * as monaco from 'monaco-editor';
import { MonacoServices } from 'monaco-languageclient';
import { useParams } from 'react-router-dom';
import * as wasm from 'wasm-crdt';

function Doc() {
    const myRef = React.createRef();
    const [connected, setConnected] = useState('Connected');
    const [code, setCode] = useState('');
    let { id } = useParams();

    const MONACO_OPTIONS = {
        autoIndent: 'full',
        automaticLayout: true,
        contextmenu: true,
        fontFamily: 'monospace',
        fontSize: 13,
        lineHeight: 24,
        hideCursorInOverviewRuler: true,
        matchBrackets: 'always',
        minimap: {
            enabled: false,
        },
        readOnly: false,
        scrollbar: {
            horizontalSliderSize: 4,
            verticalSliderSize: 18,
        },
    };

    function handleClick() {
        if (this.state.connected === 'Disconnect') {
            setConnected('Connect');
        } else {
            setConnected('Disconnect');
        }
    }

    function editorDidMount(editor) {
        MonacoServices.install(monaco);
        if (editor && editor.getModel()) {
            const editorModel = editor.getModel();
            if (editorModel) {
                editorModel.setValue(code);
            }
            editorModel.onDidChangeContent((event) => {
                event.changes
                    .sort(
                        (change1, change2) =>
                            change2.rangeOffset - change1.rangeOffset
                    )
                    .forEach((change) => {
                        // Operation here
                        wasm.delete_update(id, change.rangeOffset, change.rangeLength);
                        wasm.insert_update(id, change.rangeOffset, change.text);
                        setCode(wasm.to_string(id));
                        console.log('CHANGE: ', change);
                    });
            });
        }
        editor.focus();
    }

    useEffect(() => {
        const interval = setInterval(() => {
            console.log('INTERVAL');
            // setCode(wasm.to_string());
        }, 1000);
    }, []);

    return (
        <div>
            <MonacoEditor
                ref={myRef}
                width="100%"
                height="80vh"
                language="json"
                theme="vs"
                options={MONACO_OPTIONS}
                editorDidMount={editorDidMount}
            />
            <button onClick={() => handleClick()}>{connected}</button>
        </div>
    );
}

export default Doc;
