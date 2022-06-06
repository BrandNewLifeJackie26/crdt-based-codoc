import React, { useState } from 'react';
import * as wasm from 'wasm-crdt';
import { BrowserRouter, Routes, Route, useNavigate } from 'react-router-dom';

function Login() {
    const [docName, setDocName] = useState('');
    const [clientID, setClientID] = useState('');
    const [clientIP, setClientIP] = useState('');
    const navigation = useNavigate();

    function handleChangeDocName(event) {
        setDocName(event.target.value);
    }
    function handleChangeID(event) {
        setClientID(event.target.value);
    }
    function handleChangeIP(event) {
        setClientIP(event.target.value);
    }

    function handleSubmit(event) {
        event.preventDefault();
        console.log(clientID);
        wasm.register(clientID, clientIP, docName);
        navigation(`/Doc/${clientID}`);
    }
    return (
        <div>
            <form onSubmit={handleSubmit}>
                <label>
                    Document Name:
                    <input
                        type="text"
                        value={docName}
                        onChange={handleChangeDocName}
                    />
                </label>
                <label>
                    Client ID:
                    <input
                        type="text"
                        value={clientID}
                        onChange={handleChangeID}
                    />
                </label>
                <label>
                    Client IP:
                    <input
                        type="text"
                        value={clientIP}
                        onChange={handleChangeIP}
                    />
                </label>
                <input type="submit" value="submit" />
            </form>
        </div>
    );
}

export default Login;
