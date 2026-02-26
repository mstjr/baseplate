const vm = require('vm');
const { CrustSDK } = require('./sdk.js');

async function bootstrap() {
    const sdk = new CrustSDK();
    const event = JSON.parse(Buffer.from(process.env.USER_EVENT, 'base64').toString());
    const userCode = Buffer.from(process.env.USER_CODE, 'base64').toString('utf-8');

    const context = {
        console,
        Buffer,
        process,
        exports: {},
        require: (moduleName) => {
            return require(moduleName);
        }
    };

    vm.createContext(context);

    try {
        vm.runInContext(userCode, context);

        if (typeof context.exports.handler === 'function') {
            const result = await context.exports.handler(sdk, event);
            console.log(JSON.stringify(result));
        }
    } catch (err) {
        console.log(JSON.stringify({ error: err.message, kind: err.constructor.name }));
        process.exit(1);
    }
}

bootstrap();