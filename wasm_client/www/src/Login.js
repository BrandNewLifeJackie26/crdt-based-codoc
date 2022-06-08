import React, { useState } from 'react';
import * as wasm from 'wasm-crdt';
import { BrowserRouter, Routes, Route, useNavigate } from 'react-router-dom';
import { Button, Checkbox, Form, Input } from 'antd';
import 'antd/dist/antd.css';

function Login() {
    const [docName, setDocName] = useState('');
    const [clientID, setClientID] = useState('');
    const [clientIP, setClientIP] = useState('');
    const navigation = useNavigate();

    function handleChangeDocName(event) {
        setDocName(event.target.value);
        console.log('CHange!');
    }
    function handleChangeID(event) {
        setClientID(event.target.value);
    }
    function handleChangeIP(event) {
        setClientIP(event.target.value);
    }

    function handleSubmit(event) {
        console.log(clientID);
        wasm.register(clientID, clientIP, docName);
        navigation(`/Doc/${docName}/${clientID}`);
    }
    return (
        <div
            style={{
                padding: '40px',
                width: '50%',
                margin: '0 auto',
            }}>
            <Form
                name="basic"
                labelCol={{
                    span: 8,
                }}
                wrapperCol={{
                    span: 12,
                }}
                initialValues={{ remember: true }}
                onFinish={handleSubmit}
                autoComplete="off">
                <Form.Item
                    label="Document Name"
                    name="docName"
                    rules={[
                        {
                            required: true,
                            message: 'Please input your document name!',
                        },
                    ]}
                    onChange={handleChangeDocName}>
                    <Input />
                </Form.Item>

                <Form.Item
                    label="Client ID"
                    name="clientID"
                    rules={[
                        {
                            required: true,
                            message: 'Please input your client ID!',
                        },
                    ]}
                    onChange={handleChangeID}>
                    <Input />
                </Form.Item>

                <Form.Item
                    label="IP"
                    name="IP"
                    rules={[
                        {
                            required: true,
                            message: 'Please input your IP!',
                        },
                    ]}
                    onChange={handleChangeIP}>
                    <Input />
                </Form.Item>

                <Form.Item wrapperCol={{ offset: 8, span: 16 }}>
                    <Button type="primary" htmlType="submit">
                        Submit
                    </Button>
                </Form.Item>
            </Form>
        </div>
    );
}

export default Login;
