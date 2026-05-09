const PROXY_CONFIG = {
  "/api": {
    "target": `http://localhost:${process.env.VORTEX_PORT || 3200}`,
    "secure": false
  }
};

module.exports = PROXY_CONFIG;
