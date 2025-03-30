const express = require('express');
const { exec } = require('child_process');
const path = require('path');

const app = express();
const port = 3000;

const rustExecutablePath = path.join(__dirname);
    console.log(rustExecutablePath)

app.get('/run-rust', (req, res) => {
    // Construct the path to the Rust executable
    const rustExecutablePath = path.join(__dirname, 'rust', 'target', 'release', 'rust');
    console.log(rustExecutablePath)
    exec(rustExecutablePath, (error, stdout, stderr) => {
        if (error) {
            console.error(`Error executing Rust script: ${error}`);
            return res.status(500).send(`Error executing Rust script: ${error}`);
        }
        console.log(`Rust script output:\n${stdout}`);
        res.send(`Rust script executed successfully. Output:\n<pre>${stdout}</pre>`);
    });
});

app.listen(port, () => {
    console.log(`Node.js API listening at http://localhost:${port}`);
});