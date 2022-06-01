import React from 'react';
import Login from './Login';
import Doc from './Doc';
import { BrowserRouter, Routes, Route, useNavigate } from 'react-router-dom';

function App() {
    return (
        <BrowserRouter>
            <Routes>
                <Route path="/" element={<Login />} />
                <Route path="/Doc/:id" element={<Doc />} />
            </Routes>
        </BrowserRouter>
    );
}

export default App;
