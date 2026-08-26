local function require_env(name)
    local val = os.getenv(name)
    if not val or val == "" then
        error("FATAL: Environment variable " .. name .. " is required")
    end
    return val
end

local function optional_env(name, default)
    return os.getenv(name) or default
end

return {
    secrets = {
        jwt_secret     = require_env("YSH_JWT_SECRET"),
        db_password    = require_env("YSH_DB_PASSWORD"),
        encryption_key = require_env("YSH_ENCRYPTION_KEY"),
    },

    encryption = {
        algorithm = "aes-256-gcm",
        nonce_strategy = "counter",
        key_rotation_days = 30,
    },

    password = {
        algorithm = "argon2id",
        memory_cost = 19456,
        time_cost = 2,
        parallelism = 1,
    },

    tls = {
        min_version = "1.3",
        cert_path = require_env("YSH_TLS_CERT"),
        key_path  = require_env("YSH_TLS_KEY"),
    },

    jwt = {
        expiry_hours = 24,
        refresh_expiry_days = 30,
    },

    supervision = {
        root           = { strategy = "one_for_one",  max_restarts = 5,  max_seconds = 10 },
        infrastructure = { strategy = "rest_for_one",  max_restarts = 3,  max_seconds = 30 },
        services       = { strategy = "one_for_all",   max_restarts = 2,  max_seconds = 60 },
        security       = { strategy = "one_for_one",   max_restarts = 3,  max_seconds = 10 },
    },

    server = {
        host = "0.0.0.0",
        port = tonumber(optional_env("YSH_PORT", "8080")),
        workers = 4,
        shutdown_timeout_secs = 30,
    },

    database = {
        url = optional_env("YSH_DATABASE_URL", "sqlite://ysh.db"),
        max_connections = 10,
        connect_timeout_secs = 5,
        query_timeout_secs = 30,
    },

    backpressure = {
        server_channel_size = 1024,
        database_channel_size = 256,
        webrtc_channel_size = 512,
    },

    rate_limit = {
        requests_per_second = 100,
        burst_size = 200,
    },
}
