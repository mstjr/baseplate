const { CrustSDK } = require('./sdk.js');
const fs = require('fs');
const vm = require('vm');

async function bootstrap() {
    const sdk = new CrustSDK();
    const event = JSON.parse(Buffer.from(process.env.USER_EVENT, 'base64').toString());
    const userCode = Buffer.from(process.env.USER_CODE, 'base64').toString('utf-8');

    const context = {
        // Essential globals
        global,
        process,
        console,
        Buffer,
        setTimeout,
        setInterval,
        setImmediate,
        clearTimeout,

        // Modules and Exports
        require: require, // Give them the REAL require
        exports: {},
        module: { exports: {} },
    };

    vm.createContext(context);

    try {
        vm.runInContext(userCode, context);

        if (typeof context.exports.handler === 'function') {
            const result = await context.exports.handler(sdk, event);
            await safeOutput(result);
        }
    } catch (err) {
        await safeOutput({ error: err.message });
        process.exit(1);
    }
}

async function safeOutput(result) {
    const outputPath = '/output/result.json';
    try {
        fs.writeFileSync(outputPath, JSON.stringify(result));
        console.log('Output written successfully at ', outputPath);
    } catch (err) {
        console.error('Failed to write output:', err);
    }
}

bootstrap();