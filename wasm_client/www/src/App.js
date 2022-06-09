import React from 'react';
import Login from './Login';
import Doc from './Doc';
import { BrowserRouter, Routes, Route, useNavigate } from 'react-router-dom';
import 'antd/dist/antd.css';

function App() {
    return (
        <BrowserRouter>
            <Routes>
                <Route path="/" element={<Login />} />
                <Route path="/Doc/:doc/:id" element={<Doc />} />
            </Routes>
        </BrowserRouter>
    );
}

export default App;
