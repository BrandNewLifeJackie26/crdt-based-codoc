import React, { useState, useEffect } from 'react';
import MonacoEditor, { EditorDidMount, applyEdits } from 'react-monaco-editor';
import * as monaco from 'monaco-editor';
import { MonacoServices } from 'monaco-languageclient';
import { useParams } from 'react-router-dom';
import { Tag, Divider, Button } from 'antd';
import * as wasm from 'wasm-crdt';
var mutex_js = require('./mutex.js');

function Doc() {
    const myRef = React.createRef();
    const [connected, setConnected] = useState('Connected');
    const [code, setCode] = useState('');
    const { doc, id } = useParams();
    let mux = true;

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
        if (connected === 'Disconnect') {
            setConnected('Connect');
        } else {
            setConnected('Disconnect');
            // tentitve as logout
            wasm.sign_out(id);
        }
    }

    function editorDidMount(editor) {
        MonacoServices.install(monaco);
        if (editor && editor.getModel()) {
            const editorModel = editor.getModel();
            if (editorModel) {
                editorModel.setValue(code);
            }
            const interval = setInterval(() => {
                if (mux) {
                    mux = false;
                    wasm.to_string(id).then(function (newCode) {
                        if (newCode !== editor.getModel().getValue()) {
                            // const range = new monaco.Selection(0, 0, 0, 0);
                            // myRef.current.editor
                            //     .getModel()
                            //     .applyEdits([{ range, text: newCode }]);
                            editor.getModel().setValue(newCode);
                            console.log('Fin set Value');
                        }
                        mux = true;
                    });
                }
            }, 1000);
            editorModel.onDidChangeContent((event) => {
                if (mux) {
                    mux = false;
                    event.changes
                        .sort(
                            (change1, change2) =>
                                change2.rangeOffset - change1.rangeOffset
                        )
                        .forEach((change) => {
                            // Operation here
                            if (change.rangeLength === 0) {
                                wasm.insert_update(
                                    id,
                                    change.rangeOffset,
                                    change.text
                                )
                                    .then(function () {
                                        return wasm.to_string(id);
                                    })
                                    .then(function (newCode) {
                                        setCode(newCode);
                                        console.log('Change', change);
                                        mux = true;
                                    });
                            } else if (change.text === '') {
                                wasm.delete_update(
                                    id,
                                    change.rangeOffset,
                                    change.rangeLength
                                )
                                    .then(function () {
                                        return wasm.to_string(id);
                                    })
                                    .then(function (newCode) {
                                        setCode(newCode);
                                        console.log('newCode', newCode);
                                        console.log('Change', change);
                                        mux = true;
                                    });
                            } else {
                                wasm.delete_update(
                                    id,
                                    change.rangeOffset,
                                    change.rangeLength
                                )
                                    .then(
                                        wasm.insert_update(
                                            id,
                                            change.rangeOffset,
                                            change.text
                                        )
                                    )
                                    .then(function () {
                                        return wasm.to_string(id);
                                    })
                                    .then(function (newCode) {
                                        setCode(newCode);
                                        console.log('newCode', newCode);
                                        console.log('Change', change);
                                        mux = true;
                                    });
                            }
                        });
                }
            });
        }
        editor.focus();
    }

    // useEffect(() => {
    //     // console.log('Update code: ', code);
    //     const interval = setInterval(() => {
    //         if (mux) {
    //             mux = false;
    //             wasm.to_string(id).then(function (newCode) {
    //                 console.log(myRef);
    //                 if (newCode !== myRef.current.editor.getModel().getValue()) {
    //                     // const range = new monaco.Selection(0, 0, 0, 0);
    //                     // myRef.current.editor
    //                     //     .getModel()
    //                     //     .applyEdits([{ range, text: newCode }]);
    //                     myRef.current.editor.getModel().setValue(newCode);
    //                     console.log('Fin applyEdits');
    //                 }
    //                 mux = true;
    //             });
    //         }
    //     }, 10000);
    // }, [code]);

    return (
        <div
            style={{
                padding: '50px',
            }}>
            <span
                style={{
                    margin: '10px',
                }}>
                Document Name:
            </span>
            <Tag>{doc}</Tag>
            <br />
            <span
                style={{
                    margin: '10px',
                }}>
                Client ID:
            </span>
            <Tag>{id}</Tag>
            <Divider plain></Divider>
            <MonacoEditor
                ref={myRef}
                width="100%"
                height="80vh"
                language="json"
                theme="vs"
                options={MONACO_OPTIONS}
                editorDidMount={editorDidMount}
            />
            <Button onClick={() => handleClick()}>{connected}</Button>
        </div>
    );
}

export default Doc;
