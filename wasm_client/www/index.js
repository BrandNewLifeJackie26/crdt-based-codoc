import * as wasm from "wasm-crdt";

const submit_button = document.getElementById("submit");

const test = () => {
    var doc_name = document.getElementById('doc_name').value; 
    var client_id = document.getElementById('client_id').value;
    var client_ip = document.getElementById('client_ip').value;
    var res = wasm.register(client_id, client_ip, doc_name);
    res.then(function () {
        console.log("I'm invoked");
    });
};

submit_button.addEventListener("click", event => {
    test();
  });
  