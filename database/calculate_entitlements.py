#
# This isnt a script but rather a workflow that is used to calculate entitlements for a user.
# This does not actually run but is used to demonstrate the logic of the entitlements calculation.
#

def query(sql, *args) -> any:
    pass

def compute(expression, product_item) -> any:
    pass

def evaluate_condition(condition, product_item) -> bool:
    if condition['type'] == 'and':
        return all([evaluate_condition(c, product_item) for c in condition['conditions']])
    elif condition['type'] == 'or':
        return any([evaluate_condition(c, product_item) for c in condition['conditions']])
    elif condition['type'] == 'not':
        return not evaluate_condition(condition['condition'], product_item)
    elif condition['type'] == 'eq' or condition['type'] == 'neq' or condition['type'] == 'gt' or condition['type'] == 'gte' or condition['type'] == 'lt' or condition['type'] == 'lte':
        lhs = compute(condition['lhs'], product_item)
        rhs = compute(condition['rhs'], product_item)

        if condition['type'] == 'eq':
            return lhs == rhs
        elif condition['type'] == 'neq':
            return lhs != rhs
        elif condition['type'] == 'gt':
            return lhs > rhs
        elif condition['type'] == 'gte':
            return lhs >= rhs
        elif condition['type'] == 'lt':
            return lhs < rhs
        elif condition['type'] == 'lte':
            return lhs <= rhs
    else:
        raise ValueError(f"Unknown condition type: {condition['type']}")

def cache(key, value, ttl):
    pass

def sadd(key, value):
    pass

def jitter(min, max) -> int:
    pass

def get_entitlements(user_id):   
    assigned_role_ids = query("SELECT role_id FROM user_roles WHERE user_id = '$1'", user_id)

    products = query("SELECT p.* FROM product_purchases pp INNER JOIN products ON p.id = pp.product_id WHERE pp.recipient_id = '$1' AND pp.status = 'COMPLETED'", user_id)
    subscriptions = query("SELECT * FROM product_subscriptions WHERE user_id = $1 AND status = 'ACTIVE'", user_id)

    product_items = {
        product['id']: {
            'product': product,
            'subscription': next((s for s in subscriptions if s['product_id'] == product['id']), None)
        } for product in products
    }

    entitlements = {
        'role_ids': set(assigned_role_ids),
        'badge_ids': set(),
        'paint_ids': set(),
        'emote_set_ids': set(),
    }

    cache_invalidate_keys = {
        'roles': set(),
        'products': set()
    }

    for product in products:
        cache_invalidate_keys['products'].append(product['id'])
        for entitlement_group in product['data']['entitlement_groups']:
            condition = entitlement_group['condition']
            if condition is None or evaluate_condition(condition, product_items[product['id']]):
                for entitlement in entitlement_group['entitlements']:
                    if entitlement['type'] == 'role':
                        entitlements['role_ids'].append(entitlement['id'])
                    elif entitlement['type'] == 'badge':
                        entitlements['badge_ids'].append(entitlement['id'])
                    elif entitlement['type'] == 'paint':
                        entitlements['paint_ids'].append(entitlement['id'])
                    elif entitlement['type'] == 'emote_set':
                        entitlements['emote_set_ids'].append(entitlement['id'])

    role_badges = query("SELECT * FROM role_badges WHERE role_id IN ($1)", entitlements['role_ids'])
    for role_badge in role_badges:
        entitlements['badge_ids'].append(role_badge['badge_id'])
        cache_invalidate_keys['roles'].append(role_badge['role_id'])

    role_paints = query("SELECT * FROM role_paints WHERE role_id IN ($1)", entitlements['role_ids'])
    for role_paint in role_paints:
        entitlements['paint_ids'].append(role_paint['paint_id'])
        cache_invalidate_keys['roles'].append(role_paint['role_id'])

    role_emote_sets = query("SELECT * FROM role_emote_sets WHERE role_id IN ($1)", entitlements['role_ids'])
    for role_emote_set in role_emote_sets:
        entitlements['emote_set_ids'].append(role_emote_set['emote_set_id'])
        cache_invalidate_keys['roles'].append(role_emote_set['role_id'])

    cache(f'entitlements:cached:{user_id}', entitlements, 24 * 60 * 60 + jitter(-60 * 60, 60 * 60))

    for (key, values) in cache_invalidate_keys.items():
        for value in values:
            sadd(f'entitlements:invalidate:{key}:{value}', user_id)

    return entitlements

print(get_entitlements('some-user-id'))
