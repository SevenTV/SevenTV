#!lua name=api_transaction_mutex

local function lock(keys, args)
    local key = keys[1]
    local tx_id = args[1]
    local duration = tonumber(args[2])
    
    local value = redis.call('get', key)
    if value == tx_id then
        redis.call('expire', key, duration)
        return 1
    end

    if value then
        return 0
    end

    redis.call('set', key, tx_id, 'EX', duration)
    return 1
end

local function free(keys, args)
    local key = keys[1]
    local tx_id = args[1]
    
    local value = redis.call('get', key)
    if value == tx_id then
        redis.call('del', key)
        return 1
    end

    return 0
end

redis.register_function('api_mutex_lock', lock)
redis.register_function('api_mutex_free', free)
