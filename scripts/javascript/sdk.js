const axios = require('axios');
const { URL } = require('url');

class CrustSDK {
    constructor() {
        this._base_url = process.env.CRUST_BASE_URL || 'http://localhost/api';
        this._connections = JSON.parse(Buffer.from(process.env.CRUST_CONNECTIONS || 'e30=', 'base64').toString('utf-8'));
    }

    get base_url() {
        return this._base_url;
    }

    async fetch({
        url,
        method = "GET",
        connection = null,
        payload = null,
        headers = {},
        timeout = 5000
    }) {
        try {
            let finalUrl = url;
            let finalHeaders = { ...headers };
            let auth = null;

            if (connection) {
                const conn = this._connections[connection];
                if (!conn) throw new Error(`Connection '${connection}' not found.`);

                switch (conn.kind) {
                    case "bearer_token":
                        finalHeaders["Authorization"] = `Bearer ${conn.value}`;
                        break;
                    case "custom_header":
                        finalHeaders[conn.value.name] = conn.value.value;
                        break;
                    case "api_key":
                        if (method.toUpperCase() === "GET") {
                            const urlObj = new URL(url);
                            urlObj.searchParams.append(conn.value.key_name, conn.value.key_value);
                            finalUrl = urlObj.toString();
                        } else {
                            finalHeaders[conn.value.key_name] = conn.value.key_value;
                        }
                        break;
                    case "basic_auth":
                        auth = {
                            username: conn.value.username,
                            password: conn.value.password
                        };
                        break;
                    default:
                        throw new Error(`Unsupported connection kind '${conn.kind}'`);
                }
            }

            const response = await axios({
                method: method.toLowerCase(),
                url: finalUrl,
                data: payload,
                headers: finalHeaders,
                auth: auth,
                timeout: timeout
            });

            return response.data;
        } catch (error) {
            return {
                error: error.response ? error.response.data : error.message
            };
        }
    }

    module(moduleName) {
        return new ModuleHandler(this, moduleName);
    }
}

class ModuleHandler {
    constructor(sdk, name) {
        this.sdk = sdk;
        this.name = name;
    }

    async get(recordId) {
        return await this.sdk.fetch({
            url: `${this.sdk.base_url}/${this.name}/${recordId}`
        });
    }

    async create(data) {
        return await this.sdk.fetch({
            url: `${this.sdk.base_url}/${this.name}`,
            method: "POST",
            payload: data
        });
    }

    async update(recordId, data) {
        return await this.sdk.fetch({
            url: `${this.sdk.base_url}/${this.name}/${recordId}`,
            method: "PATCH",
            payload: data
        });
    }

    async delete(recordId) {
        const result = await this.sdk.fetch({
            url: `${this.sdk.base_url}/${this.name}/${recordId}`,
            method: "DELETE"
        });
        return result.success || false;
    }

    async list(filters) {
        const result = await this.sdk.fetch({
            url: `${this.sdk.base_url}/${this.name}`,
            method: "GET",
            payload: filters
        });
        return result.data || [];
    }
}

module.exports = { CrustSDK };