#!lua name=api_ratelimit

local function ratelimit(keys, args)
    local limit_key = keys[1]
    local limit = tonumber(args[1])
    local ticket_count = tonumber(args[2])
    local ttl = tonumber(args[3])
    local overuse_threshold = tonumber(args[4])
    local overuse_punishment = tonumber(args[5])

    local current_usage = redis.call('hincrby', limit_key, 'usage', ticket_count)
    local reset = redis.call('ttl', limit_key)

    local remaining = limit - current_usage

    if overuse_threshold > 0 and current_usage > overuse_threshold and overuse_punishment > 0 then
		if redis.call('hsetnx', limit_key, 'overused', 1) == 1 then
        	redis.call('expire', limit_key, overuse_punishment)
        	reset = overuse_punishment
		end
    elseif reset < 0 then
        redis.call('expire', limit_key, ttl)
        reset = ttl
	elseif reset > ttl then
		redis.call('hdel', limit_key, 'overused')
		redis.call('expire', limit_key, ttl)
		reset = ttl
	end

    return {remaining, reset}
end

redis.register_function('api_ratelimit', ratelimit)
