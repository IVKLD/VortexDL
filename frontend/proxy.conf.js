const PROXY_CONFIG = {
    "/api": {
        "target": `http://127.0.0.1:${process.env.VORTEX_PORT || 3200}`,
        "secure": false,
        "changeOrigin": true,
        "logLevel": "info"
    }
};

module.exports = PROXY_CONFIG;
