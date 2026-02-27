const axios = require('axios');

async function handler(sdk, event) {
    console.log("Received event:", event);
    let count = 0;
    while (true) {
        console.log("Worker is alive...");
        await new Promise(resolve => setTimeout(resolve, 5000));
        count++;
        if (count >= 3) {
            console.log("Simulating a long-running task...");
            await new Promise(resolve => setTimeout(resolve, 1000));
            break;
        }
    }
    const response = await appeler("4388635456");
    console.log("Received response:", response);
    return { message: "Hello from the worker!", data: response };
}

async function appeler(num) {
    console.log(`Appel en cours vers le numéro ${num}...`);
    return await axios.get("https://jsonplaceholder.typicode.com/posts").then((response) => {
        return response.data
    });
}

exports.handler = handler;